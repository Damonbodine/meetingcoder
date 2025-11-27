use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn read_file(path: &str) -> Result<String> {
    fs::read_to_string(path).map_err(|e| anyhow!("Failed to read file: {}", e))
}

pub fn write_file(path: &str, content: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).map_err(|e| anyhow!("Failed to write file: {}", e))
}

pub fn list_dir(path: &str) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let type_str = if path.is_dir() { "DIR" } else { "FILE" };
        entries.push(format!("{} ({})", name, type_str));
    }
    Ok(entries)
}

pub fn run_command(command: &str, cwd: Option<&str>) -> Result<String> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(command);
        c
    };

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if output.status.success() {
        Ok(stdout.to_string())
    } else {
        Err(anyhow!("Command failed with code {:?}\nStderr: {}", output.status.code(), stderr))
    }
}
