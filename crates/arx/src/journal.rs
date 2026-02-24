use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::archive;
use crate::entry::{Entry, EntryState, EntryWithState};
use crate::error::ArxError;

const JOURNAL_DIR: &str = ".arx/journal";

pub fn journal_dir(root: &Path) -> PathBuf {
    root.join(JOURNAL_DIR)
}

pub fn read_by_id(root: &Path, id: &str) -> Result<Entry, ArxError> {
    let file_path = journal_dir(root).join(format!("{}.md", id));
    read_entry(&file_path)
}

pub fn write(root: &Path, entry: &Entry) -> Result<(), ArxError> {
    let dir = journal_dir(root);
    fs::create_dir_all(&dir)?;

    let frontmatter = serde_yml::to_string(entry)?;
    let mut buf = String::new();
    buf.push_str("---\n");
    buf.push_str(&frontmatter);
    buf.push_str("---\n");
    if !entry.content.is_empty() {
        buf.push('\n');
        buf.push_str(&entry.content);
    }

    let file_path = dir.join(format!("{}.md", entry.id));
    fs::write(&file_path, buf)?;
    Ok(())
}

/// Read only the .md entries from journal/ directory.
pub fn read_all_md(root: &Path) -> Result<Vec<Entry>, ArxError> {
    let dir = journal_dir(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for dir_entry in fs::read_dir(&dir)? {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let entry = read_entry(&path)?;
        entries.push(entry);
    }

    Ok(entries)
}

/// Read all entries from both journal/*.md and archive.jsonl.
pub fn read_all(root: &Path) -> Result<Vec<Entry>, ArxError> {
    let mut entries = read_all_md(root)?;

    let archived = archive::read_archive(root)?;
    entries.extend(archived.into_iter().map(|a| a.into_entry()));

    entries.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(entries)
}

/// Delete a journal .md file by entry ID.
pub fn delete_entry_file(root: &Path, id: &str) -> Result<(), ArxError> {
    let file_path = journal_dir(root).join(format!("{id}.md"));
    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }
    Ok(())
}

fn read_entry(file_path: &Path) -> Result<Entry, ArxError> {
    let data = fs::read_to_string(file_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ArxError::NotFound
        } else {
            ArxError::Io(e)
        }
    })?;

    let mut frontmatter_lines = Vec::new();
    let mut content_lines = Vec::new();
    let mut in_frontmatter = false;
    let mut frontmatter_done = false;

    for line in data.lines() {
        if line == "---" {
            if !in_frontmatter && !frontmatter_done {
                in_frontmatter = true;
                continue;
            } else if in_frontmatter {
                in_frontmatter = false;
                frontmatter_done = true;
                continue;
            }
        }
        if in_frontmatter {
            frontmatter_lines.push(line);
        } else if frontmatter_done {
            content_lines.push(line);
        }
    }

    let frontmatter_yaml = frontmatter_lines.join("\n");
    let mut entry: Entry = serde_yml::from_str(&frontmatter_yaml)?;

    let content = content_lines.join("\n");
    entry.content = content.trim().to_string();

    Ok(entry)
}

pub fn update_reversed_by(
    root: &Path,
    target_id: &str,
    reversing_id: &str,
) -> Result<(), ArxError> {
    let mut entry = read_by_id(root, target_id)?;
    entry.reversed_by = Some(reversing_id.to_string());
    write(root, &entry)
}

pub fn get_state(root: &Path) -> Result<HashMap<String, EntryWithState>, ArxError> {
    // Start with .md entries
    let md_entries = read_all_md(root)?;

    let mut state: HashMap<String, EntryWithState> = HashMap::new();

    // First pass: add all .md entries as active
    for e in &md_entries {
        state.insert(
            e.id.clone(),
            EntryWithState {
                entry: e.clone(),
                state: EntryState::Active,
            },
        );
    }

    // Add archived entries with their stored state
    let archived = archive::read_archive(root)?;
    for ae in &archived {
        state.insert(
            ae.id.clone(),
            EntryWithState {
                entry: ae.clone().into_entry(),
                state: ae.entry_state(),
            },
        );
    }

    // Second pass: mark superseded and reversed from relationships
    let all_entries: Vec<Entry> = state.values().map(|ews| ews.entry.clone()).collect();
    for e in &all_entries {
        if let Some(ref supersedes) = e.supersedes {
            if let Some(superseded) = state.get_mut(supersedes) {
                superseded.state = EntryState::Superseded;
            }
        }
        if e.reversed_by.is_some() {
            if let Some(current) = state.get_mut(&e.id) {
                current.state = EntryState::Reversed;
            }
        }
    }

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{ActorType, EntryType};
    use chrono::TimeZone;
    use tempfile::TempDir;

    #[test]
    fn test_write_read_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let entry = Entry {
            id: "decision-2026-01-19-abc123".to_string(),
            entry_type: EntryType::Decision,
            actor: ActorType::Human,
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 19, 10, 0, 0).unwrap(),
            title: "Test Decision".to_string(),
            scope: None,
            content: "This is the content of the decision.\n\nWith multiple paragraphs."
                .to_string(),
            supersedes: None,
            reversed_by: None,
        };

        write(root, &entry).unwrap();

        let entries = read_all(root).unwrap();
        assert_eq!(entries.len(), 1);

        let read = &entries[0];
        assert_eq!(read.id, entry.id);
        assert_eq!(read.entry_type, entry.entry_type);
        assert_eq!(read.actor, entry.actor);
        assert_eq!(read.title, entry.title);
        assert_eq!(read.content, entry.content);
    }

    #[test]
    fn test_get_state_derived() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let entry1 = Entry {
            id: "decision-2026-01-19-000001".to_string(),
            entry_type: EntryType::Decision,
            actor: ActorType::Human,
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 19, 10, 0, 0).unwrap(),
            title: "Original Decision".to_string(),
            scope: None,
            content: "Original content".to_string(),
            supersedes: None,
            reversed_by: None,
        };

        let entry2 = Entry {
            id: "decision-2026-01-19-000002".to_string(),
            entry_type: EntryType::Decision,
            actor: ActorType::Human,
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 19, 11, 0, 0).unwrap(),
            title: "Updated Decision".to_string(),
            scope: None,
            content: "Updated content".to_string(),
            supersedes: Some("decision-2026-01-19-000001".to_string()),
            reversed_by: None,
        };

        let entry3 = Entry {
            id: "assumption-2026-01-19-000003".to_string(),
            entry_type: EntryType::Assumption,
            actor: ActorType::Planner,
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 19, 12, 0, 0).unwrap(),
            title: "Reversed Assumption".to_string(),
            scope: None,
            content: "This was wrong".to_string(),
            supersedes: None,
            reversed_by: Some("decision-2026-01-19-000004".to_string()),
        };

        for e in [&entry1, &entry2, &entry3] {
            write(root, e).unwrap();
        }

        let state = get_state(root).unwrap();

        assert_eq!(state[&entry1.id].state, EntryState::Superseded);
        assert_eq!(state[&entry2.id].state, EntryState::Active);
        assert_eq!(state[&entry3.id].state, EntryState::Reversed);
    }

    #[test]
    fn test_read_all_empty() {
        let tmp = TempDir::new().unwrap();
        let entries = read_all(tmp.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_read_by_id_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = read_by_id(tmp.path(), "nonexistent");
        assert!(result.is_err());
    }
}
