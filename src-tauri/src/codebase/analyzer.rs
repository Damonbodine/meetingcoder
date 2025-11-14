use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a project's codebase structure and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseManifest {
    pub root_path: PathBuf,
    pub framework: Option<String>,
    pub languages: Vec<String>,
    pub entry_points: Vec<PathBuf>,
    pub key_directories: HashMap<String, PathBuf>,
    pub dependencies: HashMap<String, String>,
    pub total_files: usize,
    pub analyzed_at: String,
}

/// Framework detection result
#[derive(Debug)]
pub struct FrameworkInfo {
    pub name: String,
    pub confidence: f64,
    pub indicators: Vec<String>,
}

/// Analyzes a codebase and generates a comprehensive manifest
pub async fn analyze_codebase(project_path: &Path) -> Result<CodebaseManifest> {
    log::info!("Starting codebase analysis for: {:?}", project_path);

    // Detect framework
    let framework = detect_framework(project_path).await?;
    log::info!("Detected framework: {:?}", framework);

    // Detect languages
    let languages = detect_languages(project_path).await?;
    log::info!("Detected languages: {:?}", languages);

    // Find entry points
    let entry_points = find_entry_points(project_path, framework.as_deref()).await?;
    log::info!("Found {} entry points", entry_points.len());

    // Map key directories
    let key_directories = map_key_directories(project_path, framework.as_deref()).await?;
    log::info!("Mapped {} key directories", key_directories.len());

    // Extract dependencies
    let dependencies = extract_dependencies(project_path, framework.as_deref()).await?;
    log::info!("Found {} dependencies", dependencies.len());

    // Count total files (excluding common ignore patterns)
    let total_files = count_source_files(project_path).await?;
    log::info!("Total source files: {}", total_files);

    let manifest = CodebaseManifest {
        root_path: project_path.to_path_buf(),
        framework,
        languages,
        entry_points,
        key_directories,
        dependencies,
        total_files,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(manifest)
}

/// Detects the primary framework used in the project
async fn detect_framework(project_path: &Path) -> Result<Option<String>> {
    let mut candidates: Vec<FrameworkInfo> = Vec::new();

    // Check for Next.js
    if project_path.join("next.config.js").exists()
        || project_path.join("next.config.mjs").exists()
        || project_path.join("next.config.ts").exists()
    {
        candidates.push(FrameworkInfo {
            name: "Next.js".to_string(),
            confidence: 0.95,
            indicators: vec!["next.config.*".to_string()],
        });
    }

    // Check for React (without Next.js)
    let package_json = project_path.join("package.json");
    if package_json.exists() {
        if let Ok(content) = fs::read_to_string(&package_json) {
            if content.contains("\"react\"") && !content.contains("\"next\"") {
                candidates.push(FrameworkInfo {
                    name: "React".to_string(),
                    confidence: 0.85,
                    indicators: vec!["package.json with react".to_string()],
                });
            }
            if content.contains("\"vue\"") {
                candidates.push(FrameworkInfo {
                    name: "Vue".to_string(),
                    confidence: 0.85,
                    indicators: vec!["package.json with vue".to_string()],
                });
            }
            if content.contains("\"@angular/core\"") {
                candidates.push(FrameworkInfo {
                    name: "Angular".to_string(),
                    confidence: 0.85,
                    indicators: vec!["package.json with @angular/core".to_string()],
                });
            }
            if content.contains("\"svelte\"") {
                candidates.push(FrameworkInfo {
                    name: "Svelte".to_string(),
                    confidence: 0.85,
                    indicators: vec!["package.json with svelte".to_string()],
                });
            }
        }
    }

    // Check for Tauri
    if project_path.join("src-tauri").exists() {
        candidates.push(FrameworkInfo {
            name: "Tauri".to_string(),
            confidence: 0.95,
            indicators: vec!["src-tauri directory".to_string()],
        });
    }

    // Check for Django
    if project_path.join("manage.py").exists() {
        if let Ok(content) = fs::read_to_string(project_path.join("manage.py")) {
            if content.contains("django") {
                candidates.push(FrameworkInfo {
                    name: "Django".to_string(),
                    confidence: 0.95,
                    indicators: vec!["manage.py with django".to_string()],
                });
            }
        }
    }

    // Check for FastAPI/Flask
    if let Ok(entries) = fs::read_dir(project_path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name == "app.py" || name == "main.py" {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("FastAPI") || content.contains("from fastapi") {
                            candidates.push(FrameworkInfo {
                                name: "FastAPI".to_string(),
                                confidence: 0.85,
                                indicators: vec![format!("{} with FastAPI import", name)],
                            });
                        }
                        if content.contains("Flask") || content.contains("from flask") {
                            candidates.push(FrameworkInfo {
                                name: "Flask".to_string(),
                                confidence: 0.85,
                                indicators: vec![format!("{} with Flask import", name)],
                            });
                        }
                    }
                }
            }
        }
    }

    // Check for Rails
    if project_path.join("config").join("application.rb").exists() {
        candidates.push(FrameworkInfo {
            name: "Ruby on Rails".to_string(),
            confidence: 0.95,
            indicators: vec!["config/application.rb".to_string()],
        });
    }

    // Return the highest confidence framework
    candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    Ok(candidates.first().map(|f| f.name.clone()))
}

/// Detects programming languages used in the project
async fn detect_languages(project_path: &Path) -> Result<Vec<String>> {
    let mut languages = Vec::new();
    let mut extensions_found = std::collections::HashSet::new();

    for entry in WalkDir::new(project_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                extensions_found.insert(ext.to_string_lossy().to_string());
            }
        }
    }

    // Map extensions to languages
    if extensions_found.contains("ts") || extensions_found.contains("tsx") {
        languages.push("TypeScript".to_string());
    }
    if extensions_found.contains("js") || extensions_found.contains("jsx") {
        languages.push("JavaScript".to_string());
    }
    if extensions_found.contains("py") {
        languages.push("Python".to_string());
    }
    if extensions_found.contains("rs") {
        languages.push("Rust".to_string());
    }
    if extensions_found.contains("go") {
        languages.push("Go".to_string());
    }
    if extensions_found.contains("java") {
        languages.push("Java".to_string());
    }
    if extensions_found.contains("rb") {
        languages.push("Ruby".to_string());
    }
    if extensions_found.contains("php") {
        languages.push("PHP".to_string());
    }
    if extensions_found.contains("swift") {
        languages.push("Swift".to_string());
    }

    Ok(languages)
}

/// Finds entry point files for the project
async fn find_entry_points(project_path: &Path, framework: Option<&str>) -> Result<Vec<PathBuf>> {
    let mut entry_points = Vec::new();

    match framework {
        Some("Next.js") => {
            // Next.js entry points
            let candidates = vec![
                "pages/_app.tsx",
                "pages/_app.js",
                "app/layout.tsx",
                "app/layout.js",
                "app/page.tsx",
                "app/page.js",
            ];
            for candidate in candidates {
                let path = project_path.join(candidate);
                if path.exists() {
                    entry_points.push(path);
                }
            }
        }
        Some("React") => {
            let candidates = vec![
                "src/index.tsx",
                "src/index.js",
                "src/main.tsx",
                "src/main.js",
                "src/App.tsx",
                "src/App.js",
            ];
            for candidate in candidates {
                let path = project_path.join(candidate);
                if path.exists() {
                    entry_points.push(path);
                }
            }
        }
        Some("Tauri") => {
            let candidates = vec!["src-tauri/src/main.rs", "src-tauri/src/lib.rs"];
            for candidate in candidates {
                let path = project_path.join(candidate);
                if path.exists() {
                    entry_points.push(path);
                }
            }
        }
        Some("Django") => {
            entry_points.push(project_path.join("manage.py"));
            if let Ok(entries) = fs::read_dir(project_path) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        let settings_path = entry.path().join("settings.py");
                        if settings_path.exists() {
                            entry_points.push(settings_path);
                        }
                    }
                }
            }
        }
        Some("FastAPI") | Some("Flask") => {
            let candidates = vec!["main.py", "app.py", "api.py"];
            for candidate in candidates {
                let path = project_path.join(candidate);
                if path.exists() {
                    entry_points.push(path);
                }
            }
        }
        _ => {
            // Generic detection
            let candidates = vec![
                "main.py",
                "app.py",
                "index.js",
                "index.ts",
                "main.rs",
                "src/main.rs",
            ];
            for candidate in candidates {
                let path = project_path.join(candidate);
                if path.exists() {
                    entry_points.push(path);
                }
            }
        }
    }

    Ok(entry_points)
}

/// Maps key directories in the project
async fn map_key_directories(
    project_path: &Path,
    framework: Option<&str>,
) -> Result<HashMap<String, PathBuf>> {
    let mut directories = HashMap::new();

    // Common directories
    let common_dirs = vec![
        "src",
        "lib",
        "components",
        "pages",
        "app",
        "api",
        "public",
        "static",
        "tests",
        "test",
        "__tests__",
        "migrations",
        "models",
        "views",
        "controllers",
        "routes",
        "utils",
        "helpers",
        "config",
        "styles",
    ];

    for dir_name in common_dirs {
        let dir_path = project_path.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            directories.insert(dir_name.to_string(), dir_path);
        }
    }

    // Framework-specific directories
    if let Some(fw) = framework {
        match fw {
            "Tauri" => {
                if let Some(tauri_dir) = directories.get("src-tauri") {
                    directories.insert("backend".to_string(), tauri_dir.clone());
                }
            }
            "Django" => {
                // Django apps
                if let Ok(entries) = fs::read_dir(project_path) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let models_path = entry.path().join("models.py");
                            if models_path.exists() {
                                directories.insert(
                                    format!("app_{}", entry.file_name().to_string_lossy()),
                                    entry.path(),
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(directories)
}

/// Extracts dependencies from package/dependency files
async fn extract_dependencies(
    project_path: &Path,
    _framework: Option<&str>,
) -> Result<HashMap<String, String>> {
    let mut dependencies = HashMap::new();

    // JavaScript/TypeScript - package.json
    let package_json = project_path.join("package.json");
    if package_json.exists() {
        if let Ok(content) = fs::read_to_string(&package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(deps) = json["dependencies"].as_object() {
                    for (name, version) in deps {
                        if let Some(v) = version.as_str() {
                            dependencies.insert(name.clone(), v.to_string());
                        }
                    }
                }
            }
        }
    }

    // Python - requirements.txt
    let requirements = project_path.join("requirements.txt");
    if requirements.exists() {
        if let Ok(content) = fs::read_to_string(&requirements) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    if let Some(pos) = line.find("==") {
                        let name = line[..pos].trim();
                        let version = line[pos + 2..].trim();
                        dependencies.insert(name.to_string(), version.to_string());
                    } else {
                        dependencies.insert(line.to_string(), "*".to_string());
                    }
                }
            }
        }
    }

    // Rust - Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            if let Ok(toml) = content.parse::<toml::Value>() {
                if let Some(deps) = toml.get("dependencies").and_then(|d| d.as_table()) {
                    for (name, value) in deps {
                        let version = match value {
                            toml::Value::String(v) => v.clone(),
                            toml::Value::Table(t) => t
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("*")
                                .to_string(),
                            _ => "*".to_string(),
                        };
                        dependencies.insert(name.clone(), version);
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

/// Counts source files, excluding common ignore patterns
async fn count_source_files(project_path: &Path) -> Result<usize> {
    let ignore_patterns = vec![
        "node_modules",
        "target",
        "dist",
        "build",
        ".git",
        ".next",
        ".vercel",
        "__pycache__",
        ".pytest_cache",
        "venv",
        ".venv",
    ];

    let mut count = 0;
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_entry(|e| {
            !ignore_patterns
                .iter()
                .any(|pattern| e.path().to_string_lossy().contains(pattern))
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy();
                if matches!(
                    ext_str.as_ref(),
                    "rs" | "ts"
                        | "tsx"
                        | "js"
                        | "jsx"
                        | "py"
                        | "go"
                        | "java"
                        | "rb"
                        | "php"
                        | "swift"
                        | "vue"
                        | "svelte"
                ) {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

/// Saves the manifest to .claude/.meeting-state.json
pub async fn save_manifest_to_state(
    project_path: &Path,
    manifest: &CodebaseManifest,
) -> Result<()> {
    let claude_dir = project_path.join(".claude");
    fs::create_dir_all(&claude_dir).context("Failed to create .claude directory")?;

    let state_file = claude_dir.join(".meeting-state.json");

    // Read existing state or create new
    let mut state: serde_json::Value = if state_file.exists() {
        let content = fs::read_to_string(&state_file)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Update with manifest data
    state["codebase_manifest"] = serde_json::to_value(manifest)?;
    state["manifest_updated_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());

    // Write back
    let content = serde_json::to_string_pretty(&state)?;
    fs::write(&state_file, content).context("Failed to write meeting state")?;

    log::info!("Saved codebase manifest to {:?}", state_file);
    Ok(())
}

/// Convenience function: analyze and save in one call
pub async fn analyze_and_save_codebase(project_path: &Path) -> Result<CodebaseManifest> {
    let manifest = analyze_codebase(project_path).await?;
    save_manifest_to_state(project_path, &manifest).await?;
    Ok(manifest)
}
