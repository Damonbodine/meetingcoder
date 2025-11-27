use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: PlanStatus,
    pub created_at: String,
}

pub struct PlanManager {
    app_handle: AppHandle,
    plan: Mutex<Vec<PlanItem>>,
}

impl PlanManager {
    pub fn new(app_handle: &AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
            plan: Mutex::new(Vec::new()),
        }
    }

    pub fn add_item(&self, title: String, description: Option<String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let item = PlanItem {
            id: id.clone(),
            title,
            description,
            status: PlanStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let mut plan = self.plan.lock().unwrap();
        plan.push(item.clone());
        
        // Safety: Keep only the last 100 items to prevent memory growth
        if plan.len() > 100 {
            let remove_count = plan.len() - 100;
            plan.drain(0..remove_count);
        }
        
        // Emit event
        let _ = self.app_handle.emit("plan-updated", &*plan);
        
        id
    }

    pub fn update_status(&self, id: &str, status: PlanStatus) {
        let mut plan = self.plan.lock().unwrap();
        if let Some(item) = plan.iter_mut().find(|i| i.id == id) {
            item.status = status;
            let _ = self.app_handle.emit("plan-updated", &*plan);
        }
    }

    pub fn get_plan(&self) -> Vec<PlanItem> {
        self.plan.lock().unwrap().clone()
    }
}
