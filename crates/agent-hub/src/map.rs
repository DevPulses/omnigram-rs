//! Карта задач (`file_map.json`): модули tdesktop, их приоритет и зависимости.

use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct FileMap {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: String,
    /// Пути в репозитории tdesktop, которые покрывает задача.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Уровень приоритета: 1 (фундамент) .. 5 (UI).
    pub priority: u32,
    /// Задачи, которые должны быть complete, прежде чем эту можно claim.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Связанные задачи: пока эта в работе, их тоже блокируем (защита
    /// от параллельного редактирования тесно связанных модулей).
    #[serde(default)]
    pub related: Vec<String>,
}

impl FileMap {
    pub fn load(path: &std::path::Path) -> Result<FileMap, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("не удалось открыть {}: {e}", path.display()))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("ошибка парсинга {}: {e}", path.display()))
    }

    pub fn task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Все идентификаторы задач, чьи пути пересекаются с путями задачи `id`
    /// (автоматические related при отсутствии явных).
    pub fn implicit_related(&self, id: &str) -> Vec<String> {
        let Some(task) = self.task(id) else { return vec![] };
        self.tasks
            .iter()
            .filter(|t| t.id != id)
            .filter(|t| t.paths.iter().any(|p| task.paths.contains(p)))
            .map(|t| t.id.clone())
            .collect()
    }
}
