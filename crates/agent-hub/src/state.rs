//! Динамическое состояние выполнения задач (`.agent-state.json`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    /// id задачи -> текущий статус.
    pub tasks: std::collections::BTreeMap<String, TaskState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub status: Status,
    #[serde(default)]
    pub agent: Option<String>,
    /// Related-лок: задачу нельзя claim, пока держатель не отпустит.
    /// НЕ означает, что работа идёт — только блокировку.
    #[serde(default)]
    pub locked: bool,
}

impl State {
    /// Загрузить из строки (пустой файл = пустое состояние).
    pub fn parse(raw: &str) -> Result<State, String> {
        if raw.trim().is_empty() {
            return Ok(State::default());
        }
        serde_json::from_str(raw).map_err(|e| format!("битый .agent-state.json: {e}"))
    }

    pub fn status_of(&self, id: &str) -> Status {
        self.tasks
            .get(id)
            .map(|t| t.status.clone())
            .unwrap_or(Status::Pending)
    }

    pub fn agent_of(&self, id: &str) -> Option<String> {
        self.tasks.get(id).and_then(|t| t.agent.clone())
    }
}
