use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entry::{ActorType, Entry, EntryState, EntryType};
use crate::error::ArxError;

const ARCHIVE_FILE: &str = "archive.jsonl";

/// Entry as stored in archive.jsonl (includes content and state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub actor: String,
    pub date: DateTime<Utc>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reversed_by: Option<String>,
    pub state: String,
}

impl ArchiveEntry {
    /// Convert an Entry + EntryState into an ArchiveEntry.
    pub fn from_entry(entry: &Entry, state: &EntryState) -> Self {
        Self {
            id: entry.id.clone(),
            entry_type: entry.entry_type.to_string(),
            actor: entry.actor.to_string(),
            date: entry.date,
            title: entry.title.clone(),
            scope: entry.scope.clone(),
            content: entry.content.clone(),
            supersedes: entry.supersedes.clone(),
            reversed_by: entry.reversed_by.clone(),
            state: state.to_string(),
        }
    }

    /// Convert back into an Entry (lossy: state is not carried).
    pub fn into_entry(self) -> Entry {
        let entry_type = self
            .entry_type
            .parse::<EntryType>()
            .unwrap_or(EntryType::Decision);
        let actor = self.actor.parse::<ActorType>().unwrap_or(ActorType::Human);
        Entry {
            id: self.id,
            entry_type,
            actor,
            date: self.date,
            title: self.title,
            scope: self.scope,
            content: self.content,
            supersedes: self.supersedes,
            reversed_by: self.reversed_by,
        }
    }

    /// Parse the state string back into an EntryState.
    pub fn entry_state(&self) -> EntryState {
        match self.state.as_str() {
            "superseded" => EntryState::Superseded,
            "reversed" => EntryState::Reversed,
            _ => EntryState::Active,
        }
    }
}

/// Read all entries from archive.jsonl.
pub fn read_archive(root: &Path) -> Result<Vec<ArchiveEntry>, ArxError> {
    let path = root.join(ARCHIVE_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: ArchiveEntry = serde_json::from_str(trimmed)?;
        entries.push(entry);
    }

    Ok(entries)
}

/// Append entries to archive.jsonl (creates file if it doesn't exist).
pub fn append_to_archive(root: &Path, entries: &[ArchiveEntry]) -> Result<(), ArxError> {
    if entries.is_empty() {
        return Ok(());
    }

    let path = root.join(ARCHIVE_FILE);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

    for entry in entries {
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{json}")?;
    }

    Ok(())
}

/// Remove a specific entry from the archive by ID (rewrites the file).
///
/// # Warning
///
/// This is a **destructive, irreversible** operation intended only for data
/// sensitivity scenarios (e.g. GDPR right-to-erasure). arx is designed as an
/// append-only log — prefer `supersede` or `reverse` to preserve full history.
/// Use this only when the entry content itself must be permanently expunged.
pub fn remove_from_archive(root: &Path, id: &str) -> Result<bool, ArxError> {
    let path = root.join(ARCHIVE_FILE);
    if !path.exists() {
        return Ok(false);
    }

    let entries = read_archive(root)?;
    let original_len = entries.len();
    let filtered: Vec<_> = entries.into_iter().filter(|e| e.id != id).collect();

    if filtered.len() == original_len {
        return Ok(false);
    }

    // Rewrite the file
    let mut file = fs::File::create(&path)?;
    for entry in &filtered {
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{json}")?;
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{ActorType, EntryType};
    use chrono::TimeZone;
    use tempfile::TempDir;

    fn make_entry(id: &str, title: &str) -> Entry {
        Entry {
            id: id.to_string(),
            entry_type: EntryType::Decision,
            actor: ActorType::Human,
            date: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            title: title.to_string(),
            scope: Some("backend".to_string()),
            content: "Some content here".to_string(),
            supersedes: None,
            reversed_by: None,
        }
    }

    #[test]
    fn archive_write_read_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let entry = make_entry("decision-2026-01-15-a1b2c3", "Use PostgreSQL");
        let archive_entry = ArchiveEntry::from_entry(&entry, &EntryState::Superseded);

        append_to_archive(root, std::slice::from_ref(&archive_entry)).unwrap();

        let entries = read_archive(root).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "decision-2026-01-15-a1b2c3");
        assert_eq!(entries[0].title, "Use PostgreSQL");
        assert_eq!(entries[0].state, "superseded");
        assert_eq!(entries[0].scope, Some("backend".to_string()));
        assert_eq!(entries[0].content, "Some content here");
    }

    #[test]
    fn archive_append_preserves_existing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let e1 = ArchiveEntry::from_entry(
            &make_entry("decision-2026-01-15-a1b2c3", "First"),
            &EntryState::Superseded,
        );
        let e2 = ArchiveEntry::from_entry(
            &make_entry("assumption-2026-01-16-d4e5f6", "Second"),
            &EntryState::Reversed,
        );

        append_to_archive(root, &[e1]).unwrap();
        append_to_archive(root, &[e2]).unwrap();

        let entries = read_archive(root).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "First");
        assert_eq!(entries[1].title, "Second");
    }

    #[test]
    fn archive_read_empty() {
        let tmp = TempDir::new().unwrap();
        let entries = read_archive(tmp.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn archive_entry_into_entry_roundtrip() {
        let original = make_entry("decision-2026-01-15-a1b2c3", "Use PostgreSQL");
        let archive = ArchiveEntry::from_entry(&original, &EntryState::Active);
        let restored = archive.into_entry();

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.entry_type, original.entry_type);
        assert_eq!(restored.actor, original.actor);
        assert_eq!(restored.title, original.title);
        assert_eq!(restored.content, original.content);
        assert_eq!(restored.scope, original.scope);
    }

    #[test]
    fn archive_entry_state_parsing() {
        let e = ArchiveEntry::from_entry(&make_entry("test", "Test"), &EntryState::Superseded);
        assert_eq!(e.entry_state(), EntryState::Superseded);

        let e2 = ArchiveEntry::from_entry(&make_entry("test", "Test"), &EntryState::Reversed);
        assert_eq!(e2.entry_state(), EntryState::Reversed);

        let e3 = ArchiveEntry::from_entry(&make_entry("test", "Test"), &EntryState::Active);
        assert_eq!(e3.entry_state(), EntryState::Active);
    }

    #[test]
    fn archive_remove_entry() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let e1 = ArchiveEntry::from_entry(&make_entry("id-1", "First"), &EntryState::Superseded);
        let e2 = ArchiveEntry::from_entry(&make_entry("id-2", "Second"), &EntryState::Reversed);

        append_to_archive(root, &[e1, e2]).unwrap();

        let removed = remove_from_archive(root, "id-1").unwrap();
        assert!(removed);

        let entries = read_archive(root).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "id-2");
    }

    #[test]
    fn archive_remove_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let removed = remove_from_archive(tmp.path(), "nonexistent").unwrap();
        assert!(!removed);
    }
}
