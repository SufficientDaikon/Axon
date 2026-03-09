// manifest.rs — Axon.toml parser and manifest types

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ---- Manifest types ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub features: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub lint: LintConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub edition: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(DetailedDep),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDep {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub opt_level: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintConfig {
    #[serde(default)]
    pub warn: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub allow: Vec<String>,
}

// ---- Simple TOML parser ----

#[derive(Debug, Clone)]
enum TomlValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<TomlValue>),
    Table(HashMap<String, TomlValue>),
}

impl TomlValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            TomlValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            TomlValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    fn as_str_array(&self) -> Option<Vec<String>> {
        match self {
            TomlValue::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.push(item.as_str()?.to_string());
                }
                Some(result)
            }
            _ => None,
        }
    }
}

fn is_balanced(s: &str, open: char, close: char) -> bool {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut prev = '\0';
    for ch in s.chars() {
        if ch == '"' && prev != '\\' {
            in_string = !in_string;
        } else if !in_string {
            if ch == open { depth += 1; }
            if ch == close { depth -= 1; }
        }
        prev = ch;
    }
    depth == 0
}

fn strip_comment(s: &str) -> &str {
    let mut in_string = false;
    let mut prev = '\0';
    for (i, ch) in s.char_indices() {
        if ch == '"' && prev != '\\' {
            in_string = !in_string;
        } else if ch == '#' && !in_string {
            return s[..i].trim_end();
        }
        prev = ch;
    }
    s
}

fn split_top_level(s: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut depth_bracket = 0i32;
    let mut depth_brace = 0i32;
    let mut prev = '\0';

    for ch in s.chars() {
        if ch == '"' && prev != '\\' {
            in_string = !in_string;
            current.push(ch);
        } else if in_string {
            current.push(ch);
        } else if ch == '[' {
            depth_bracket += 1;
            current.push(ch);
        } else if ch == ']' {
            depth_bracket -= 1;
            current.push(ch);
        } else if ch == '{' {
            depth_brace += 1;
            current.push(ch);
        } else if ch == '}' {
            depth_brace -= 1;
            current.push(ch);
        } else if ch == delimiter && depth_bracket == 0 && depth_brace == 0 {
            parts.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(ch);
        }
        prev = ch;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        parts.push(trimmed);
    }

    parts
}

fn parse_toml_string(s: &str) -> Result<TomlValue, String> {
    if !s.starts_with('"') {
        return Err(format!("expected string, got: {}", s));
    }
    let chars: Vec<char> = s.chars().collect();
    let mut i = 1;
    let mut result = String::new();
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                _ => {
                    result.push('\\');
                    result.push(chars[i + 1]);
                }
            }
            i += 2;
        } else if chars[i] == '"' {
            return Ok(TomlValue::String(result));
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    Err("unterminated string".to_string())
}

fn parse_toml_value(s: &str) -> Result<TomlValue, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty value".to_string());
    }
    if s.starts_with('"') {
        parse_toml_string(s)
    } else if s.starts_with('[') {
        parse_toml_array(s)
    } else if s.starts_with('{') {
        parse_toml_inline_table(s)
    } else if s == "true" {
        Ok(TomlValue::Boolean(true))
    } else if s == "false" {
        Ok(TomlValue::Boolean(false))
    } else if let Ok(n) = s.parse::<i64>() {
        Ok(TomlValue::Integer(n))
    } else {
        Err(format!("invalid TOML value: {}", s))
    }
}

fn parse_toml_array(s: &str) -> Result<TomlValue, String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(format!("expected array, got: {}", s));
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Ok(TomlValue::Array(vec![]));
    }
    let parts = split_top_level(inner, ',');
    let mut values = Vec::new();
    for part in parts {
        let trimmed = part.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        values.push(parse_toml_value(&trimmed)?);
    }
    Ok(TomlValue::Array(values))
}

fn parse_toml_inline_table(s: &str) -> Result<TomlValue, String> {
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return Err(format!("expected inline table, got: {}", s));
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Ok(TomlValue::Table(HashMap::new()));
    }
    let parts = split_top_level(inner, ',');
    let mut table = HashMap::new();
    for part in parts {
        let trimmed = part.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let value = parse_toml_value(trimmed[eq_pos + 1..].trim())?;
            table.insert(key, value);
        } else {
            return Err(format!("expected key = value in inline table, got: {}", trimmed));
        }
    }
    Ok(TomlValue::Table(table))
}

fn parse_toml(input: &str) -> Result<HashMap<String, HashMap<String, TomlValue>>, String> {
    let mut sections: HashMap<String, HashMap<String, TomlValue>> = HashMap::new();
    let mut current_section = String::new();
    sections.insert(current_section.clone(), HashMap::new());

    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = strip_comment(line).trim();
        if trimmed.is_empty() {
            continue;
        }

        // Section header [name] (skip [[array_of_tables]])
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            if let Some(end) = trimmed.find(']') {
                current_section = trimmed[1..end].trim().to_string();
                sections.entry(current_section.clone()).or_insert_with(HashMap::new);
                continue;
            }
        }

        // Key = value
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let mut value_str = trimmed[eq_pos + 1..].trim().to_string();

            // Handle multi-line arrays
            if value_str.starts_with('[') && !is_balanced(&value_str, '[', ']') {
                while let Some(next_line) = lines.next() {
                    let next_trimmed = strip_comment(next_line).trim().to_string();
                    value_str.push(' ');
                    value_str.push_str(&next_trimmed);
                    if is_balanced(&value_str, '[', ']') {
                        break;
                    }
                }
            }

            // Handle multi-line inline tables
            if value_str.starts_with('{') && !is_balanced(&value_str, '{', '}') {
                while let Some(next_line) = lines.next() {
                    let next_trimmed = strip_comment(next_line).trim().to_string();
                    value_str.push(' ');
                    value_str.push_str(&next_trimmed);
                    if is_balanced(&value_str, '{', '}') {
                        break;
                    }
                }
            }

            let value = parse_toml_value(&value_str)?;
            sections
                .get_mut(&current_section)
                .unwrap()
                .insert(key, value);
        }
    }

    Ok(sections)
}

fn parse_dep_section(section: Option<&HashMap<String, TomlValue>>) -> HashMap<String, Dependency> {
    let mut deps = HashMap::new();
    if let Some(section) = section {
        for (key, value) in section {
            match value {
                TomlValue::String(v) => {
                    deps.insert(key.clone(), Dependency::Simple(v.clone()));
                }
                TomlValue::Table(t) => {
                    let detailed = DetailedDep {
                        version: t.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        git: t.get("git").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        branch: t.get("branch").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        path: t.get("path").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        features: t.get("features").and_then(|v| v.as_str_array()).unwrap_or_default(),
                    };
                    deps.insert(key.clone(), Dependency::Detailed(detailed));
                }
                _ => {}
            }
        }
    }
    deps
}

fn write_dep(out: &mut String, name: &str, dep: &Dependency) {
    match dep {
        Dependency::Simple(v) => {
            out.push_str(&format!("{} = \"{}\"\n", name, v));
        }
        Dependency::Detailed(d) => {
            let mut parts = Vec::new();
            if let Some(ref v) = d.version {
                parts.push(format!("version = \"{}\"", v));
            }
            if let Some(ref g) = d.git {
                parts.push(format!("git = \"{}\"", g));
            }
            if let Some(ref b) = d.branch {
                parts.push(format!("branch = \"{}\"", b));
            }
            if let Some(ref p) = d.path {
                parts.push(format!("path = \"{}\"", p));
            }
            if !d.features.is_empty() {
                let feats: Vec<String> = d.features.iter().map(|f| format!("\"{}\"", f)).collect();
                parts.push(format!("features = [{}]", feats.join(", ")));
            }
            out.push_str(&format!("{} = {{ {} }}\n", name, parts.join(", ")));
        }
    }
}

// ---- Manifest impl ----

impl Manifest {
    pub fn from_str(content: &str) -> Result<Self, String> {
        let sections = parse_toml(content)?;

        let pkg_section = sections
            .get("package")
            .ok_or_else(|| "missing [package] section".to_string())?;

        let package = PackageInfo {
            name: pkg_section
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing package.name".to_string())?
                .to_string(),
            version: pkg_section
                .get("version")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing package.version".to_string())?
                .to_string(),
            authors: pkg_section
                .get("authors")
                .and_then(|v| v.as_str_array())
                .unwrap_or_default(),
            edition: pkg_section
                .get("edition")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: pkg_section
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            license: pkg_section
                .get("license")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: pkg_section
                .get("repository")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        let dependencies = parse_dep_section(sections.get("dependencies"));
        let dev_dependencies = parse_dep_section(sections.get("dev_dependencies"));

        let build = if let Some(build_section) = sections.get("build") {
            BuildConfig {
                target: build_section
                    .get("target")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                opt_level: build_section
                    .get("opt_level")
                    .and_then(|v| v.as_i64())
                    .map(|n| n as u8),
            }
        } else {
            BuildConfig::default()
        };

        let features = if let Some(feat_section) = sections.get("features") {
            let mut map = HashMap::new();
            for (key, value) in feat_section {
                map.insert(key.clone(), value.as_str_array().unwrap_or_default());
            }
            map
        } else {
            HashMap::new()
        };

        let lint = if let Some(lint_section) = sections.get("lint") {
            LintConfig {
                warn: lint_section
                    .get("warn")
                    .and_then(|v| v.as_str_array())
                    .unwrap_or_default(),
                deny: lint_section
                    .get("deny")
                    .and_then(|v| v.as_str_array())
                    .unwrap_or_default(),
                allow: lint_section
                    .get("allow")
                    .and_then(|v| v.as_str_array())
                    .unwrap_or_default(),
            }
        } else {
            LintConfig::default()
        };

        Ok(Manifest {
            package,
            dependencies,
            dev_dependencies,
            build,
            features,
            lint,
        })
    }

    pub fn to_string_pretty(&self) -> Result<String, String> {
        let mut out = String::new();

        out.push_str("[package]\n");
        out.push_str(&format!("name = \"{}\"\n", self.package.name));
        out.push_str(&format!("version = \"{}\"\n", self.package.version));
        if !self.package.authors.is_empty() {
            let authors: Vec<String> = self.package.authors.iter().map(|a| format!("\"{}\"", a)).collect();
            out.push_str(&format!("authors = [{}]\n", authors.join(", ")));
        }
        if let Some(ref ed) = self.package.edition {
            out.push_str(&format!("edition = \"{}\"\n", ed));
        }
        if let Some(ref desc) = self.package.description {
            out.push_str(&format!("description = \"{}\"\n", desc));
        }
        if let Some(ref lic) = self.package.license {
            out.push_str(&format!("license = \"{}\"\n", lic));
        }
        if let Some(ref repo) = self.package.repository {
            out.push_str(&format!("repository = \"{}\"\n", repo));
        }

        if !self.dependencies.is_empty() {
            out.push_str("\n[dependencies]\n");
            for (name, dep) in &self.dependencies {
                write_dep(&mut out, name, dep);
            }
        }

        if !self.dev_dependencies.is_empty() {
            out.push_str("\n[dev_dependencies]\n");
            for (name, dep) in &self.dev_dependencies {
                write_dep(&mut out, name, dep);
            }
        }

        if self.build.target.is_some() || self.build.opt_level.is_some() {
            out.push_str("\n[build]\n");
            if let Some(ref target) = self.build.target {
                out.push_str(&format!("target = \"{}\"\n", target));
            }
            if let Some(level) = self.build.opt_level {
                out.push_str(&format!("opt_level = {}\n", level));
            }
        }

        if !self.features.is_empty() {
            out.push_str("\n[features]\n");
            for (name, values) in &self.features {
                let vals: Vec<String> = values.iter().map(|v| format!("\"{}\"", v)).collect();
                out.push_str(&format!("{} = [{}]\n", name, vals.join(", ")));
            }
        }

        if !self.lint.warn.is_empty() || !self.lint.deny.is_empty() || !self.lint.allow.is_empty() {
            out.push_str("\n[lint]\n");
            if !self.lint.warn.is_empty() {
                let vals: Vec<String> = self.lint.warn.iter().map(|v| format!("\"{}\"", v)).collect();
                out.push_str(&format!("warn = [{}]\n", vals.join(", ")));
            }
            if !self.lint.deny.is_empty() {
                let vals: Vec<String> = self.lint.deny.iter().map(|v| format!("\"{}\"", v)).collect();
                out.push_str(&format!("deny = [{}]\n", vals.join(", ")));
            }
            if !self.lint.allow.is_empty() {
                let vals: Vec<String> = self.lint.allow.iter().map(|v| format!("\"{}\"", v)).collect();
                out.push_str(&format!("allow = [{}]\n", vals.join(", ")));
            }
        }

        Ok(out)
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {}", path, e))?;
        Self::from_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_manifest() {
        let toml = r#"
[package]
name = "my-project"
version = "0.1.0"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "my-project");
        assert_eq!(manifest.package.version, "0.1.0");
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn test_parse_with_dependencies() {
        let toml = r#"
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = "1.28"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.dependencies.len(), 2);
        match &manifest.dependencies["serde"] {
            Dependency::Simple(v) => assert_eq!(v, "1.0"),
            _ => panic!("expected simple dependency"),
        }
        match &manifest.dependencies["tokio"] {
            Dependency::Simple(v) => assert_eq!(v, "1.28"),
            _ => panic!("expected simple dependency"),
        }
    }

    #[test]
    fn test_parse_with_detailed_deps() {
        let toml = r#"
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
my-lib = { version = "2.0", git = "https://github.com/example/lib", features = ["async", "tls"] }
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        match &manifest.dependencies["my-lib"] {
            Dependency::Detailed(d) => {
                assert_eq!(d.version.as_deref(), Some("2.0"));
                assert_eq!(d.git.as_deref(), Some("https://github.com/example/lib"));
                assert_eq!(d.features, vec!["async", "tls"]);
            }
            _ => panic!("expected detailed dependency"),
        }
    }

    #[test]
    fn test_parse_with_features() {
        let toml = r#"
[package]
name = "test"
version = "1.0.0"

[features]
default = ["std"]
std = []
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.features["default"], vec!["std"]);
        assert!(manifest.features["std"].is_empty());
    }

    #[test]
    fn test_parse_with_build_config() {
        let toml = r#"
[package]
name = "test"
version = "1.0.0"

[build]
target = "x86_64"
opt_level = 2
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.build.target.as_deref(), Some("x86_64"));
        assert_eq!(manifest.build.opt_level, Some(2));
    }

    #[test]
    fn test_parse_with_lint_config() {
        let toml = r#"
[package]
name = "test"
version = "1.0.0"

[lint]
warn = ["unused"]
deny = ["unsafe"]
allow = ["dead_code"]
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.lint.warn, vec!["unused"]);
        assert_eq!(manifest.lint.deny, vec!["unsafe"]);
        assert_eq!(manifest.lint.allow, vec!["dead_code"]);
    }

    #[test]
    fn test_to_string_pretty() {
        let toml = r#"
[package]
name = "my-project"
version = "0.1.0"
authors = ["Alice", "Bob"]
edition = "2024"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        let output = manifest.to_string_pretty().unwrap();
        assert!(output.contains("name = \"my-project\""));
        assert!(output.contains("version = \"0.1.0\""));
        assert!(output.contains("authors = [\"Alice\", \"Bob\"]"));
        assert!(output.contains("edition = \"2024\""));
    }

    #[test]
    fn test_parse_missing_package() {
        let toml = r#"
[dependencies]
serde = "1.0"
"#;
        assert!(Manifest::from_str(toml).is_err());
    }

    #[test]
    fn test_parse_with_comments() {
        let toml = r#"
# This is a project manifest
[package]
name = "test"   # project name
version = "1.0.0"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "test");
        assert_eq!(manifest.package.version, "1.0.0");
    }

    #[test]
    fn test_parse_multiline_array() {
        let toml = r#"
[package]
name = "test"
version = "1.0.0"
authors = [
    "Alice",
    "Bob",
    "Charlie"
]
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.authors.len(), 3);
        assert_eq!(manifest.package.authors[0], "Alice");
        assert_eq!(manifest.package.authors[2], "Charlie");
    }
}
