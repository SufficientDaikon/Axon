// scaffold.rs — Project creation and initialization

use std::fs;
use std::path::Path;

/// Create a new Axon project in a subdirectory of `path`.
pub fn create_project(name: &str, path: &str) -> Result<(), String> {
    let project_dir = Path::new(path).join(name);

    if project_dir.exists() {
        return Err(format!("directory '{}' already exists", project_dir.display()));
    }

    fs::create_dir_all(project_dir.join("src"))
        .map_err(|e| format!("failed to create src directory: {}", e))?;
    fs::create_dir_all(project_dir.join("tests"))
        .map_err(|e| format!("failed to create tests directory: {}", e))?;

    let manifest = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"
"#,
        name
    );
    fs::write(project_dir.join("Axon.toml"), manifest)
        .map_err(|e| format!("failed to write Axon.toml: {}", e))?;

    let main_content = r#"// Main entry point

fn main() {
    println("Hello, Axon!")
}
"#;
    fs::write(project_dir.join("src").join("main.axon"), main_content)
        .map_err(|e| format!("failed to write main.axon: {}", e))?;

    let readme = format!("# {}\n\nAn Axon project.\n", name);
    fs::write(project_dir.join("README.md"), readme)
        .map_err(|e| format!("failed to write README.md: {}", e))?;

    Ok(())
}

/// Initialize a new Axon project in an existing directory.
pub fn init_project(path: &str) -> Result<(), String> {
    let project_dir = Path::new(path);
    let manifest_path = project_dir.join("Axon.toml");

    if manifest_path.exists() {
        return Err("Axon.toml already exists in this directory".to_string());
    }

    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("failed to create src directory: {}", e))?;
    }

    let tests_dir = project_dir.join("tests");
    if !tests_dir.exists() {
        fs::create_dir_all(&tests_dir)
            .map_err(|e| format!("failed to create tests directory: {}", e))?;
    }

    let manifest = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"
"#,
        name
    );
    fs::write(&manifest_path, manifest)
        .map_err(|e| format!("failed to write Axon.toml: {}", e))?;

    let main_path = src_dir.join("main.axon");
    if !main_path.exists() {
        let main_content = r#"// Main entry point

fn main() {
    println("Hello, Axon!")
}
"#;
        fs::write(&main_path, main_content)
            .map_err(|e| format!("failed to write main.axon: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project() {
        let tmp = std::env::temp_dir().join("axon_test_create_project");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        create_project("hello", tmp.to_str().unwrap()).unwrap();

        let project_dir = tmp.join("hello");
        assert!(project_dir.join("Axon.toml").exists());
        assert!(project_dir.join("src").join("main.axon").exists());
        assert!(project_dir.join("tests").exists());
        assert!(project_dir.join("README.md").exists());

        let content = fs::read_to_string(project_dir.join("Axon.toml")).unwrap();
        assert!(content.contains("name = \"hello\""));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_init_project() {
        let tmp = std::env::temp_dir().join("axon_test_init_project");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        init_project(tmp.to_str().unwrap()).unwrap();

        assert!(tmp.join("Axon.toml").exists());
        assert!(tmp.join("src").join("main.axon").exists());
        assert!(tmp.join("tests").exists());

        // Second init should fail
        assert!(init_project(tmp.to_str().unwrap()).is_err());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_create_project_duplicate() {
        let tmp = std::env::temp_dir().join("axon_test_create_dup");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        create_project("dup", tmp.to_str().unwrap()).unwrap();
        assert!(create_project("dup", tmp.to_str().unwrap()).is_err());

        let _ = fs::remove_dir_all(&tmp);
    }
}
