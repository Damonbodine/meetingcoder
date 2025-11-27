use std::process::Command;
use anyhow::Result;

pub struct TTSEngine;

impl TTSEngine {
    pub fn speak(text: &str, voice: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("say");
        
        if let Some(v) = voice {
            cmd.arg("-v").arg(v);
        }
        
        cmd.arg(text);
        
        // Spawn the command so it doesn't block the main thread
        // We don't wait for it to finish
        cmd.spawn()?;
        
        Ok(())
    }
    
    pub fn get_persona_voice(persona_role: &str) -> &'static str {
        match persona_role {
            "Product Manager" => "Samantha",
            "User Researcher" => "Karen", 
            "Senior Architect" => "Daniel",
            "Junior Developer" => "Rishi",
            "QA Engineer" => "Tessa",
            "Security Specialist" => "Moira",
            _ => "Alex",
        }
    }
}
