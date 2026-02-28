use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::ArxError;

const CHECKPOINT_PATH: &str = "checkpoint.json";
const STALE_HOURS: i64 = 72;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointStatus {
    InProgress,
    Completed,
    Failed,
    Paused,
}

impl std::fmt::Display for CheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CheckpointStatus::InProgress => "in_progress",
            CheckpointStatus::Completed => "completed",
            CheckpointStatus::Failed => "failed",
            CheckpointStatus::Paused => "paused",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub version: String,
    pub task_id: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: CheckpointStatus,
}

impl Checkpoint {
    pub fn new(task_id: String) -> Self {
        let now = Utc::now();
        Self {
            version: "1".to_string(),
            task_id,
            started_at: now,
            last_activity: now,
            status: CheckpointStatus::InProgress,
        }
    }
}

pub fn save(root: &Path, cp: &mut Checkpoint) -> Result<(), ArxError> {
    if cp.version.is_empty() {
        cp.version = "1".to_string();
    }
    if cp.started_at == DateTime::<Utc>::default() {
        cp.started_at = Utc::now();
    }
    if cp.last_activity == DateTime::<Utc>::default() {
        cp.last_activity = Utc::now();
    }
    if cp.status == CheckpointStatus::InProgress && cp.task_id.is_empty() {
        // keep defaults
    }

    let path = root.join(CHECKPOINT_PATH);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let data = serde_json::to_string_pretty(cp)?;
    fs::write(&path, data)?;
    Ok(())
}

pub fn load(root: &Path) -> Result<Option<Checkpoint>, ArxError> {
    let path = root.join(CHECKPOINT_PATH);
    match fs::read_to_string(&path) {
        Ok(data) => {
            let cp: Checkpoint = serde_json::from_str(&data)?;
            Ok(Some(cp))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ArxError::Io(e)),
    }
}

pub fn clear(root: &Path) -> Result<(), ArxError> {
    let path = root.join(CHECKPOINT_PATH);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ArxError::Io(e)),
    }
}

pub fn is_stale(cp: &Checkpoint) -> bool {
    is_stale_with_threshold(cp, Duration::hours(STALE_HOURS))
}

pub fn is_stale_with_threshold(cp: &Checkpoint, threshold: Duration) -> bool {
    Utc::now().signed_duration_since(cp.last_activity) > threshold
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_load() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let mut cp = Checkpoint {
            version: String::new(),
            task_id: "test-task-123".to_string(),
            started_at: DateTime::<Utc>::default(),
            last_activity: DateTime::<Utc>::default(),
            status: CheckpointStatus::InProgress,
        };

        save(root, &mut cp).unwrap();

        assert_eq!(cp.version, "1");
        assert_ne!(cp.started_at, DateTime::<Utc>::default());
        assert_ne!(cp.last_activity, DateTime::<Utc>::default());

        let loaded = load(root).unwrap().expect("checkpoint should exist");
        assert_eq!(loaded.task_id, "test-task-123");
        assert_eq!(loaded.version, "1");
        assert_eq!(loaded.status, CheckpointStatus::InProgress);
    }

    #[test]
    fn test_load_no_checkpoint() {
        let tmp = TempDir::new().unwrap();
        let cp = load(tmp.path()).unwrap();
        assert!(cp.is_none());
    }

    #[test]
    fn test_clear() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let mut cp = Checkpoint::new("test-task".to_string());
        save(root, &mut cp).unwrap();

        clear(root).unwrap();

        let loaded = load(root).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_clear_no_checkpoint() {
        let tmp = TempDir::new().unwrap();
        // Should not fail
        clear(tmp.path()).unwrap();
    }

    #[test]
    fn test_is_stale() {
        let recent = Checkpoint {
            version: "1".to_string(),
            task_id: "t".to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now() - Duration::hours(1),
            status: CheckpointStatus::InProgress,
        };
        assert!(!is_stale(&recent));

        let old = Checkpoint {
            version: "1".to_string(),
            task_id: "t".to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now() - Duration::hours(100),
            status: CheckpointStatus::InProgress,
        };
        assert!(is_stale(&old));
    }

    #[test]
    fn test_is_stale_with_threshold() {
        let cp = Checkpoint {
            version: "1".to_string(),
            task_id: "t".to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now() - Duration::hours(73),
            status: CheckpointStatus::InProgress,
        };

        assert!(!is_stale_with_threshold(&cp, Duration::hours(74)));
        assert!(is_stale_with_threshold(&cp, Duration::hours(72)));
    }
}
