use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Generates .claudeignore file to protect sensitive areas
pub fn generate_claudeignore(project_path: &Path, framework: Option<&str>) -> Result<()> {
    let claudeignore_path = project_path.join(".claudeignore");

    // Base patterns that apply to all projects
    let mut patterns = vec![
        "# MeetingCoder File Isolation".to_string(),
        "# Generated automatically to protect core project files".to_string(),
        "".to_string(),
        "# Protect core application code (allow reading, restrict edits)".to_string(),
        "src/**".to_string(),
        "app/**".to_string(),
        "pages/**".to_string(),
        "lib/**".to_string(),
        "components/**".to_string(),
        "api/**".to_string(),
        "routes/**".to_string(),
        "controllers/**".to_string(),
        "models/**".to_string(),
        "views/**".to_string(),
        "".to_string(),
        "# Protect configuration files".to_string(),
        "package.json".to_string(),
        "package-lock.json".to_string(),
        "bun.lockb".to_string(),
        "yarn.lock".to_string(),
        "pnpm-lock.yaml".to_string(),
        "tsconfig.json".to_string(),
        "next.config.*".to_string(),
        "vite.config.*".to_string(),
        "*.config.js".to_string(),
        "*.config.ts".to_string(),
        "*.config.mjs".to_string(),
        "".to_string(),
        "# Protect backend code (Tauri, Django, Rails, etc.)".to_string(),
        "src-tauri/**".to_string(),
        "manage.py".to_string(),
        "wsgi.py".to_string(),
        "asgi.py".to_string(),
        "Gemfile".to_string(),
        "Gemfile.lock".to_string(),
        "Rakefile".to_string(),
        "config.ru".to_string(),
        "".to_string(),
        "# Protect dependency directories".to_string(),
        "node_modules/**".to_string(),
        "target/**".to_string(),
        "dist/**".to_string(),
        "build/**".to_string(),
        ".next/**".to_string(),
        ".vercel/**".to_string(),
        "__pycache__/**".to_string(),
        "venv/**".to_string(),
        ".venv/**".to_string(),
        "".to_string(),
        "# Protect Git and CI/CD".to_string(),
        ".git/**".to_string(),
        ".github/**".to_string(),
        ".gitlab-ci.yml".to_string(),
        ".travis.yml".to_string(),
        "".to_string(),
        "# Protect environment and secrets".to_string(),
        ".env".to_string(),
        ".env.*".to_string(),
        "*.pem".to_string(),
        "*.key".to_string(),
        "*.cert".to_string(),
        "credentials.json".to_string(),
        "secrets.json".to_string(),
        "".to_string(),
        "# Protect database files".to_string(),
        "*.db".to_string(),
        "*.sqlite".to_string(),
        "*.sqlite3".to_string(),
        "".to_string(),
        "# EXCEPTION: Allow experiments directory (where meeting code goes)".to_string(),
        "!experiments/**".to_string(),
        "".to_string(),
        "# EXCEPTION: Allow .claude directory for meeting state".to_string(),
        "!.claude/**".to_string(),
        "".to_string(),
        "# EXCEPTION: Allow tests directory for new tests".to_string(),
        "!tests/**".to_string(),
        "!test/**".to_string(),
        "!__tests__/**".to_string(),
    ];

    // Framework-specific patterns
    if let Some(fw) = framework {
        patterns.push("".to_string());
        patterns.push(format!("# Framework-specific ({}) protections", fw));

        match fw {
            "Next.js" => {
                patterns.extend(vec![
                    "public/**".to_string(),
                    "styles/**".to_string(),
                    "middleware.ts".to_string(),
                    "instrumentation.ts".to_string(),
                ]);
            }
            "Django" => {
                patterns.extend(vec![
                    "*/migrations/**".to_string(),
                    "staticfiles/**".to_string(),
                    "media/**".to_string(),
                    "settings.py".to_string(),
                    "urls.py".to_string(),
                ]);
            }
            "Rails" => {
                patterns.extend(vec![
                    "db/schema.rb".to_string(),
                    "db/migrate/**".to_string(),
                    "config/**".to_string(),
                    "public/**".to_string(),
                ]);
            }
            "Tauri" => {
                patterns.extend(vec![
                    "src-tauri/Cargo.toml".to_string(),
                    "src-tauri/Cargo.lock".to_string(),
                    "src-tauri/tauri.conf.json".to_string(),
                    "src-tauri/capabilities/**".to_string(),
                ]);
            }
            _ => {}
        }
    }

    let content = patterns.join("\n");
    fs::write(&claudeignore_path, content).context("Failed to write .claudeignore file")?;

    log::info!("Generated .claudeignore at {:?}", claudeignore_path);
    Ok(())
}

/// Checks if a path is safe for code generation (within experiments folder)
pub fn is_safe_path(project_path: &Path, target_path: &Path) -> bool {
    let experiments_dir = project_path.join("experiments");
    let claude_dir = project_path.join(".claude");
    let tests_dir_1 = project_path.join("tests");
    let tests_dir_2 = project_path.join("test");
    let tests_dir_3 = project_path.join("__tests__");

    // Check if target is within allowed directories
    target_path.starts_with(&experiments_dir)
        || target_path.starts_with(&claude_dir)
        || target_path.starts_with(&tests_dir_1)
        || target_path.starts_with(&tests_dir_2)
        || target_path.starts_with(&tests_dir_3)
}

/// Returns the safe experiments directory for a meeting
pub fn get_experiments_dir(project_path: &Path, meeting_id: &str) -> PathBuf {
    project_path.join("experiments").join(meeting_id)
}

/// Creates the experiments directory for a meeting
pub fn create_experiments_dir(project_path: &Path, meeting_id: &str) -> Result<PathBuf> {
    let experiments_dir = get_experiments_dir(project_path, meeting_id);
    fs::create_dir_all(&experiments_dir).context("Failed to create experiments directory")?;

    // Create a README to explain the purpose
    let readme_path = experiments_dir.join("README.md");
    let readme_content = format!(
        "# Experiments: {}\n\n\
        This directory contains experimental code generated during the meeting.\n\n\
        ## Purpose\n\
        - Safe sandbox for AI-generated code\n\
        - Isolated from core application logic\n\
        - Easy to review before merging into main codebase\n\n\
        ## Workflow\n\
        1. AI generates code here during the meeting\n\
        2. Review and test the generated code\n\
        3. Move approved code to appropriate locations in the main codebase\n\
        4. Commit changes via the draft PR\n\n\
        ## Directory Structure\n\
        ```\n\
        experiments/{meeting_id}/\n\
        ├── README.md           (this file)\n\
        ├── src/                (source code)\n\
        ├── tests/              (test files)\n\
        └── docs/               (documentation)\n\
        ```\n",
        meeting_id
    );

    fs::write(&readme_path, readme_content).context("Failed to write experiments README")?;

    // Create subdirectories
    fs::create_dir_all(experiments_dir.join("src"))?;
    fs::create_dir_all(experiments_dir.join("tests"))?;
    fs::create_dir_all(experiments_dir.join("docs"))?;

    log::info!("Created experiments directory at {:?}", experiments_dir);
    Ok(experiments_dir)
}

/// Validates that a file operation is safe
pub fn validate_file_operation(
    project_path: &Path,
    target_path: &Path,
    operation: FileOperation,
) -> Result<()> {
    // Normalize paths
    let target_canonical = target_path
        .canonicalize()
        .or_else(|_| {
            // If file doesn't exist yet, check parent
            if let Some(parent) = target_path.parent() {
                parent.canonicalize()
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Parent directory not found",
                ))
            }
        })
        .context("Failed to resolve target path")?;

    // Check if path is safe
    if !is_safe_path(project_path, &target_canonical) {
        return Err(anyhow::anyhow!(
            "File operation blocked: {} is outside the experiments directory. \
            MeetingCoder isolates generated code to experiments/{{meeting_id}}/ for safety.",
            target_path.display()
        ));
    }

    // Additional checks based on operation type
    match operation {
        FileOperation::Create | FileOperation::Write => {
            // Check if we're trying to overwrite a protected file
            if target_path.exists() {
                log::warn!(
                    "Overwriting existing file in experiments directory: {:?}",
                    target_path
                );
            }
        }
        FileOperation::Delete => {
            // Extra caution for deletions
            log::warn!("Delete operation requested for: {:?}", target_path);
        }
        FileOperation::Read => {
            // Reads are generally safe, but log for audit trail
            log::debug!("Read operation: {:?}", target_path);
        }
    }

    Ok(())
}

/// Types of file operations
#[derive(Debug, Clone, Copy)]
pub enum FileOperation {
    Create,
    Read,
    Write,
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_safe_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create directories
        fs::create_dir_all(project_path.join("experiments/meeting1")).unwrap();
        fs::create_dir_all(project_path.join("src")).unwrap();

        // Safe paths
        assert!(is_safe_path(
            project_path,
            &project_path.join("experiments/meeting1/code.ts")
        ));
        assert!(is_safe_path(
            project_path,
            &project_path.join(".claude/state.json")
        ));

        // Unsafe paths
        assert!(!is_safe_path(
            project_path,
            &project_path.join("src/app.ts")
        ));
        assert!(!is_safe_path(
            project_path,
            &project_path.join("package.json")
        ));
    }

    #[test]
    fn test_create_experiments_dir() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let experiments_dir = create_experiments_dir(project_path, "test-meeting").unwrap();

        assert!(experiments_dir.exists());
        assert!(experiments_dir.join("README.md").exists());
        assert!(experiments_dir.join("src").exists());
        assert!(experiments_dir.join("tests").exists());
        assert!(experiments_dir.join("docs").exists());
    }

    #[test]
    fn test_validate_file_operation_safe() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let experiments_dir = project_path.join("experiments/meeting1");
        fs::create_dir_all(&experiments_dir).unwrap();

        let target = experiments_dir.join("code.ts");
        let result = validate_file_operation(project_path, &target, FileOperation::Create);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_operation_unsafe() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        fs::create_dir_all(project_path.join("src")).unwrap();

        let target = project_path.join("src/app.ts");
        let result = validate_file_operation(project_path, &target, FileOperation::Create);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("outside the experiments directory"));
    }
}
