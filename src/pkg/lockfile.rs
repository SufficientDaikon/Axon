// lockfile.rs — Lock file generation and parsing

use crate::pkg::resolver::{ResolvedDep, DepSource};

#[derive(Debug, Clone)]
pub struct LockFile {
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: Option<String>,
}

impl LockFile {
    /// Generate a lock file from resolved dependencies.
    pub fn generate(resolved: &[ResolvedDep]) -> Self {
        let packages = resolved
            .iter()
            .map(|dep| {
                let source = match &dep.source {
                    DepSource::Registry(url) => format!("registry+{}", url),
                    DepSource::Git { url, branch } => {
                        if let Some(b) = branch {
                            format!("git+{}#{}", url, b)
                        } else {
                            format!("git+{}", url)
                        }
                    }
                    DepSource::Path(p) => format!("path+{}", p),
                };
                LockedPackage {
                    name: dep.name.clone(),
                    version: dep.version.clone(),
                    source,
                    checksum: None,
                }
            })
            .collect();

        LockFile { packages }
    }

    /// Serialize the lock file to a TOML-like string.
    pub fn to_string(&self) -> String {
        let mut out = String::from("# Axon.lock - auto-generated, do not edit\n\n");
        for pkg in &self.packages {
            out.push_str("[[package]]\n");
            out.push_str(&format!("name = \"{}\"\n", pkg.name));
            out.push_str(&format!("version = \"{}\"\n", pkg.version));
            out.push_str(&format!("source = \"{}\"\n", pkg.source));
            if let Some(ref checksum) = pkg.checksum {
                out.push_str(&format!("checksum = \"{}\"\n", checksum));
            }
            out.push('\n');
        }
        out
    }

    /// Parse a lock file from its string representation.
    pub fn from_str(content: &str) -> Result<Self, String> {
        let mut packages = Vec::new();
        let mut current: Option<LockedPackage> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('#') || trimmed.is_empty() {
                if trimmed.is_empty() {
                    if let Some(pkg) = current.take() {
                        packages.push(pkg);
                    }
                }
                continue;
            }

            if trimmed == "[[package]]" {
                if let Some(pkg) = current.take() {
                    packages.push(pkg);
                }
                current = Some(LockedPackage {
                    name: String::new(),
                    version: String::new(),
                    source: String::new(),
                    checksum: None,
                });
                continue;
            }

            if let Some(ref mut pkg) = current {
                if let Some(eq_pos) = trimmed.find('=') {
                    let key = trimmed[..eq_pos].trim();
                    let value = trimmed[eq_pos + 1..].trim().trim_matches('"');
                    match key {
                        "name" => pkg.name = value.to_string(),
                        "version" => pkg.version = value.to_string(),
                        "source" => pkg.source = value.to_string(),
                        "checksum" => pkg.checksum = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        }

        // Push final package if file doesn't end with blank line
        if let Some(pkg) = current {
            packages.push(pkg);
        }

        Ok(LockFile { packages })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkg::resolver::{ResolvedDep, DepSource};

    #[test]
    fn test_generate_lockfile() {
        let deps = vec![
            ResolvedDep {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                source: DepSource::Registry("https://registry.axon-lang.org".to_string()),
            },
            ResolvedDep {
                name: "my-lib".to_string(),
                version: "2.0.0".to_string(),
                source: DepSource::Git {
                    url: "https://github.com/example/lib".to_string(),
                    branch: Some("main".to_string()),
                },
            },
        ];
        let lockfile = LockFile::generate(&deps);
        assert_eq!(lockfile.packages.len(), 2);
        assert_eq!(lockfile.packages[0].name, "serde");
        assert!(lockfile.packages[0].source.starts_with("registry+"));
        assert!(lockfile.packages[1].source.starts_with("git+"));
    }

    #[test]
    fn test_lockfile_to_string() {
        let lockfile = LockFile {
            packages: vec![LockedPackage {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                source: "registry+https://registry.axon-lang.org".to_string(),
                checksum: Some("abc123".to_string()),
            }],
        };
        let output = lockfile.to_string();
        assert!(output.contains("[[package]]"));
        assert!(output.contains("name = \"serde\""));
        assert!(output.contains("checksum = \"abc123\""));
    }

    #[test]
    fn test_lockfile_from_str() {
        let input = r#"# Axon.lock - auto-generated, do not edit

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://registry.axon-lang.org"
checksum = "abc123"
"#;
        let lockfile = LockFile::from_str(input).unwrap();
        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages[0].name, "serde");
        assert_eq!(lockfile.packages[0].version, "1.0.0");
        assert_eq!(lockfile.packages[0].checksum.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_lockfile_roundtrip() {
        let deps = vec![
            ResolvedDep {
                name: "tokio".to_string(),
                version: "1.28.0".to_string(),
                source: DepSource::Git {
                    url: "https://github.com/tokio-rs/tokio".to_string(),
                    branch: Some("main".to_string()),
                },
            },
            ResolvedDep {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                source: DepSource::Registry("https://registry.axon-lang.org".to_string()),
            },
        ];
        let lockfile = LockFile::generate(&deps);
        let text = lockfile.to_string();
        let parsed = LockFile::from_str(&text).unwrap();
        assert_eq!(parsed.packages.len(), 2);
        for pkg in &parsed.packages {
            assert!(!pkg.name.is_empty());
            assert!(!pkg.version.is_empty());
            assert!(!pkg.source.is_empty());
        }
    }
}
