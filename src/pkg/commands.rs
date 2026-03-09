// commands.rs — CLI command handlers for the Axon package manager

use std::fs;
use std::path::Path;

use crate::pkg::manifest::Manifest;
use crate::pkg::scaffold;
use crate::pkg::resolver::Resolver;
use crate::pkg::lockfile::LockFile;

fn find_manifest() -> Result<Manifest, String> {
    Manifest::from_file("Axon.toml")
}

pub fn cmd_new(name: &str) -> Result<(), String> {
    scaffold::create_project(name, ".")?;
    println!("Created new Axon project '{}'", name);
    Ok(())
}

pub fn cmd_init() -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("failed to get current directory: {}", e))?;
    scaffold::init_project(cwd.to_str().ok_or("invalid path")?)?;
    println!("Initialized Axon project");
    Ok(())
}

pub fn cmd_build() -> Result<(), String> {
    let manifest = find_manifest()?;
    let resolved = Resolver::resolve(&manifest)?;
    let lockfile = LockFile::generate(&resolved);

    fs::write("Axon.lock", lockfile.to_string())
        .map_err(|e| format!("failed to write Axon.lock: {}", e))?;

    let entry = Path::new("src").join("main.axon");
    if !entry.exists() {
        return Err("no src/main.axon found".to_string());
    }

    let source = fs::read_to_string(&entry)
        .map_err(|e| format!("failed to read {}: {}", entry.display(), e))?;

    let (_program, errors) = crate::parse_source(&source, &entry.to_string_lossy());
    if !errors.is_empty() {
        let mut msg = String::from("build failed with errors:\n");
        for e in &errors {
            msg.push_str(&format!("  {}\n", e.format_human()));
        }
        return Err(msg);
    }

    println!(
        "Build succeeded: {} v{}",
        manifest.package.name, manifest.package.version
    );
    Ok(())
}

pub fn cmd_run() -> Result<(), String> {
    cmd_build()?;
    println!("note: interpreted execution not yet implemented");
    Ok(())
}

pub fn cmd_test() -> Result<(), String> {
    let _manifest = find_manifest()?;
    let test_dir = Path::new("tests");

    if !test_dir.exists() {
        println!("No tests directory found");
        return Ok(());
    }

    let mut test_count = 0;
    let mut fail_count = 0;

    let entries = fs::read_dir(test_dir)
        .map_err(|e| format!("failed to read tests directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("axon") {
            test_count += 1;
            let source = fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
            let (_program, errors) = crate::parse_source(&source, &path.to_string_lossy());
            if !errors.is_empty() {
                fail_count += 1;
                eprintln!("FAIL: {}", path.display());
                for e in &errors {
                    eprintln!("  {}", e.format_human());
                }
            } else {
                println!("PASS: {}", path.display());
            }
        }
    }

    if test_count == 0 {
        println!("No test files found");
    } else {
        println!("{} tests, {} failed", test_count, fail_count);
    }

    if fail_count > 0 {
        Err(format!("{} test(s) failed", fail_count))
    } else {
        Ok(())
    }
}

pub fn cmd_add(package: &str, version: Option<&str>) -> Result<(), String> {
    let manifest_path = "Axon.toml";
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("failed to read Axon.toml: {}", e))?;

    // Verify it's a valid manifest
    let _manifest = Manifest::from_str(&content)?;

    let version_str = version.unwrap_or("*");
    let dep_line = format!("{} = \"{}\"\n", package, version_str);

    let new_content = if let Some(pos) = content.find("[dependencies]") {
        let after = pos + "[dependencies]".len();
        let insert_pos = content[after..]
            .find('\n')
            .map(|p| after + p + 1)
            .unwrap_or(content.len());
        let mut new = content[..insert_pos].to_string();
        new.push_str(&dep_line);
        new.push_str(&content[insert_pos..]);
        new
    } else {
        let mut new = content;
        new.push_str("\n[dependencies]\n");
        new.push_str(&dep_line);
        new
    };

    fs::write(manifest_path, new_content)
        .map_err(|e| format!("failed to write Axon.toml: {}", e))?;

    println!("Added {} = \"{}\"", package, version_str);
    Ok(())
}

pub fn cmd_remove(package: &str) -> Result<(), String> {
    let manifest_path = "Axon.toml";
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("failed to read Axon.toml: {}", e))?;

    let prefix = format!("{} = ", package);
    let mut found = false;
    let new_content: String = content
        .lines()
        .filter(|line| {
            if line.trim().starts_with(&prefix) {
                found = true;
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if !found {
        return Err(format!("dependency '{}' not found in Axon.toml", package));
    }

    fs::write(manifest_path, new_content + "\n")
        .map_err(|e| format!("failed to write Axon.toml: {}", e))?;

    println!("Removed {}", package);
    Ok(())
}

pub fn cmd_clean() -> Result<(), String> {
    let build_dir = Path::new("build");
    if build_dir.exists() {
        fs::remove_dir_all(build_dir)
            .map_err(|e| format!("failed to remove build directory: {}", e))?;
    }

    let lock_path = Path::new("Axon.lock");
    if lock_path.exists() {
        fs::remove_file(lock_path)
            .map_err(|e| format!("failed to remove Axon.lock: {}", e))?;
    }

    println!("Cleaned build artifacts");
    Ok(())
}

pub fn cmd_fmt() -> Result<(), String> {
    let _manifest = find_manifest()?;
    let src_dir = Path::new("src");

    if !src_dir.exists() {
        return Err("no src directory found".to_string());
    }

    format_dir(src_dir)?;
    println!("Formatted all source files");
    Ok(())
}

fn format_dir(dir: &Path) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            format_dir(&path)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("axon") {
            let source = fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
            match crate::fmt::Formatter::format(&source, &path.to_string_lossy()) {
                Ok(formatted) => {
                    fs::write(&path, formatted)
                        .map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
                    println!("  Formatted {}", path.display());
                }
                Err(_) => {
                    eprintln!("  Skipped {} (parse errors)", path.display());
                }
            }
        }
    }

    Ok(())
}

pub fn cmd_lint() -> Result<(), String> {
    let _manifest = find_manifest()?;
    let src_dir = Path::new("src");

    if !src_dir.exists() {
        return Err("no src directory found".to_string());
    }

    let mut total_warnings = 0;
    lint_dir(src_dir, &mut total_warnings)?;

    if total_warnings == 0 {
        println!("No lint warnings");
    } else {
        println!("{} total warning(s)", total_warnings);
    }

    Ok(())
}

fn lint_dir(dir: &Path, total: &mut usize) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            lint_dir(&path, total)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("axon") {
            let source = fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
            let warnings = crate::lint::Linter::lint(&source, &path.to_string_lossy());
            if !warnings.is_empty() {
                for w in &warnings {
                    eprintln!("{}", w.format_human());
                }
                *total += warnings.len();
            }
        }
    }

    Ok(())
}
