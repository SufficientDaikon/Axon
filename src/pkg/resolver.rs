// resolver.rs — Dependency resolution

use crate::pkg::manifest::{Manifest, Dependency};

pub struct Resolver;

#[derive(Debug, Clone)]
pub struct ResolvedDep {
    pub name: String,
    pub version: String,
    pub source: DepSource,
}

#[derive(Debug, Clone)]
pub enum DepSource {
    Registry(String),
    Git { url: String, branch: Option<String> },
    Path(String),
}

impl Resolver {
    pub fn new() -> Self {
        Resolver
    }

    /// Resolve all dependencies from the manifest into concrete sources.
    pub fn resolve(manifest: &Manifest) -> Result<Vec<ResolvedDep>, String> {
        let mut resolved = Vec::new();

        for (name, dep) in &manifest.dependencies {
            resolved.push(Self::resolve_dep(name, dep)?);
        }

        for (name, dep) in &manifest.dev_dependencies {
            resolved.push(Self::resolve_dep(name, dep)?);
        }

        Ok(resolved)
    }

    fn resolve_dep(name: &str, dep: &Dependency) -> Result<ResolvedDep, String> {
        match dep {
            Dependency::Simple(version) => Ok(ResolvedDep {
                name: name.to_string(),
                version: version.clone(),
                source: DepSource::Registry("https://registry.axon-lang.org".to_string()),
            }),
            Dependency::Detailed(d) => {
                let version = d.version.clone().unwrap_or_else(|| "0.0.0".to_string());
                let source = if let Some(ref git_url) = d.git {
                    DepSource::Git {
                        url: git_url.clone(),
                        branch: d.branch.clone(),
                    }
                } else if let Some(ref path) = d.path {
                    DepSource::Path(path.clone())
                } else {
                    DepSource::Registry("https://registry.axon-lang.org".to_string())
                };
                Ok(ResolvedDep {
                    name: name.to_string(),
                    version,
                    source,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkg::manifest::*;
    use std::collections::HashMap;

    fn make_manifest(deps: HashMap<String, Dependency>) -> Manifest {
        Manifest {
            package: PackageInfo {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                authors: vec![],
                edition: None,
                description: None,
                license: None,
                repository: None,
            },
            dependencies: deps,
            dev_dependencies: HashMap::new(),
            build: BuildConfig::default(),
            features: HashMap::new(),
            lint: LintConfig::default(),
        }
    }

    #[test]
    fn test_resolve_simple_deps() {
        let mut deps = HashMap::new();
        deps.insert("serde".to_string(), Dependency::Simple("1.0".to_string()));
        deps.insert("tokio".to_string(), Dependency::Simple("1.28".to_string()));
        let manifest = make_manifest(deps);
        let resolved = Resolver::resolve(&manifest).unwrap();
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().any(|r| r.name == "serde" && r.version == "1.0"));
        assert!(resolved.iter().any(|r| r.name == "tokio" && r.version == "1.28"));
    }

    #[test]
    fn test_resolve_detailed_deps() {
        let mut deps = HashMap::new();
        deps.insert(
            "my-lib".to_string(),
            Dependency::Detailed(DetailedDep {
                version: Some("2.0".to_string()),
                git: Some("https://github.com/example/lib".to_string()),
                branch: Some("main".to_string()),
                path: None,
                features: vec![],
            }),
        );
        let manifest = make_manifest(deps);
        let resolved = Resolver::resolve(&manifest).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].name, "my-lib");
        assert_eq!(resolved[0].version, "2.0");
        assert!(matches!(resolved[0].source, DepSource::Git { .. }));
    }

    #[test]
    fn test_resolve_path_dep() {
        let mut deps = HashMap::new();
        deps.insert(
            "local-lib".to_string(),
            Dependency::Detailed(DetailedDep {
                version: Some("0.1.0".to_string()),
                git: None,
                branch: None,
                path: Some("../local-lib".to_string()),
                features: vec![],
            }),
        );
        let manifest = make_manifest(deps);
        let resolved = Resolver::resolve(&manifest).unwrap();
        assert_eq!(resolved.len(), 1);
        assert!(matches!(resolved[0].source, DepSource::Path(ref p) if p == "../local-lib"));
    }

    #[test]
    fn test_resolve_empty() {
        let manifest = make_manifest(HashMap::new());
        let resolved = Resolver::resolve(&manifest).unwrap();
        assert!(resolved.is_empty());
    }
}
