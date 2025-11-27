use crate::summarization::llm::call_claude_api_text;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Debug)]
pub struct Suggestion {
    pub id: String,
    pub text: String,
    #[serde(rename = "type")]
    pub kind: String, // "question", "technical", "edge-case"
    pub persona_name: String,
    pub persona_role: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum MeetingMode {
    Discovery,
    Technical,
    Review,
}

#[derive(Clone, Debug)]
struct Persona {
    name: String,
    role: String,
    system_prompt: String,
}

#[derive(Clone)]
pub struct SuggestionEngine {
    app_handle: AppHandle,
    is_running: Arc<Mutex<bool>>,
    current_mode: Arc<Mutex<MeetingMode>>,
    personas: HashMap<MeetingMode, Vec<Persona>>,
    memory_manager: Arc<crate::managers::memory::MemoryManager>,
}

impl SuggestionEngine {
    pub fn new(app_handle: AppHandle, memory_manager: Arc<crate::managers::memory::MemoryManager>) -> Self {
        let mut personas = HashMap::new();

        personas.insert(
            MeetingMode::Discovery,
            vec![
                Persona {
                    name: "Alice".to_string(),
                    role: "Product Manager".to_string(),
                    system_prompt: "You are an experienced Product Manager. Listen to the conversation and suggest questions about business value, user needs, and feature prioritization. Focus on the 'why'.".to_string(),
                },
                Persona {
                    name: "Sam".to_string(),
                    role: "User Researcher".to_string(),
                    system_prompt: "You are a User Researcher. Listen for assumptions about user behavior and suggest questions to validate them. Focus on user pain points.".to_string(),
                },
            ],
        );

        personas.insert(
            MeetingMode::Technical,
            vec![
                Persona {
                    name: "Bob".to_string(),
                    role: "Senior Architect".to_string(),
                    system_prompt: "You are a Senior System Architect. Listen to the technical discussion and suggest questions about scalability, security, and system design patterns. Identify potential bottlenecks.".to_string(),
                },
                Persona {
                    name: "Charlie".to_string(),
                    role: "Junior Developer".to_string(),
                    system_prompt: "You are a Junior Developer. Listen for implementation details and ask clarifying questions about specific libraries, APIs, or edge cases that might be missed.".to_string(),
                },
            ],
        );

        personas.insert(
            MeetingMode::Review,
            vec![
                Persona {
                    name: "Quinn".to_string(),
                    role: "QA Engineer".to_string(),
                    system_prompt: "You are a QA Engineer. Listen for new features and suggest test cases or potential bugs. Ask 'How will we test this?'".to_string(),
                },
                Persona {
                    name: "Sarah".to_string(),
                    role: "Security Specialist".to_string(),
                    system_prompt: "You are a Security Specialist. Listen for data handling and API endpoints. Suggest questions about authentication, authorization, and data privacy.".to_string(),
                },
            ],
        );

        Self {
            app_handle,
            is_running: Arc::new(Mutex::new(false)),
            current_mode: Arc::new(Mutex::new(MeetingMode::Technical)), // Default to Technical
            personas,
            memory_manager,
        }
    }

    pub fn start(&self) {
        let is_running = self.is_running.clone();
        let app_handle = self.app_handle.clone();
        let current_mode = self.current_mode.clone();
        let personas_map = self.personas.clone();

        let memory_manager = self.memory_manager.clone();

        tauri::async_runtime::spawn(async move {
            let mut running = is_running.lock().await;
            if *running {
                return;
            }
            *running = true;
            drop(running);

            log::info!("Suggestion Engine started (Multi-Persona)");

            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;

                // Check if we should stop
                if !*is_running.lock().await {
                    break;
                }

                // Get current mode and active personas
                let mode = current_mode.lock().await.clone();
                if let Some(active_personas) = personas_map.get(&mode) {
                    // For now, pick a random persona or iterate. Let's pick one random for this interval.
                    // In a real app, we might run them in parallel or round-robin.
                    use rand::seq::SliceRandom;
                    let mut rng = rand::thread_rng();
                    if let Some(persona) = active_personas.choose(&mut rng) {
                         // In a real implementation, we would get the recent transcript here
                        let transcript_snippet = "User: We need to add a phone number field to the user model.";
                        
                        // Retrieve relevant context from memory
                        let context = if let Ok(memories) = memory_manager.search_memory(transcript_snippet, 3) {
                            memories.iter().map(|m| m.text.clone()).collect::<Vec<_>>().join("\n")
                        } else {
                            String::new()
                        };

                        if let Ok(suggestion) = generate_suggestion(transcript_snippet, persona, &context).await {
                            let _ = app_handle.emit("new-suggestion", suggestion);
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        let is_running = self.is_running.clone();
        tauri::async_runtime::spawn(async move {
            *is_running.lock().await = false;
        });
    }

    pub async fn set_mode(&self, mode: MeetingMode) {
        *self.current_mode.lock().await = mode;
        // Optionally emit an event that mode changed
    }
}

async fn generate_suggestion(transcript: &str, persona: &Persona, context: &str) -> Result<Suggestion> {
    let user_prompt = format!("Transcript:\n{}\n\nRelevant Past Context:\n{}\n\nBased on your role as {}, suggest ONE specific question or consideration:", transcript, context, persona.role);

    let response = call_claude_api_text("claude-3-sonnet-20240229", &persona.system_prompt, &user_prompt).await?;

    Ok(Suggestion {
        id: uuid::Uuid::new_v4().to_string(),
        text: response.trim().to_string(),
        kind: "suggestion".to_string(),
        persona_name: persona.name.clone(),
        persona_role: persona.role.clone(),
    })
}
