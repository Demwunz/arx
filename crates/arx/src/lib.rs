pub mod archive;
pub mod checkpoint;
pub mod entry;
pub mod error;
pub mod id;
pub mod journal;
pub mod search;

use std::path::Path;

use chrono::Utc;

use archive::ArchiveEntry;
use entry::{ActorType, Entry, EntryState, EntryType};
use error::ArxError;
pub use search::{SearchOptions, SearchResult};

/// Record creates a new journal entry and returns its ID.
pub fn record(
    root: &Path,
    entry_type: &str,
    actor: Option<&str>,
    title: &str,
    scope: Option<&str>,
    supersedes: Option<&str>,
    reverses: Option<&str>,
) -> Result<String, ArxError> {
    let et = entry_type
        .parse::<EntryType>()
        .map_err(|_| ArxError::InvalidType)?;
    let actor = actor
        .and_then(|a| a.parse::<ActorType>().ok())
        .unwrap_or(ActorType::Human);

    let id = id::generate_id(&et)?;

    let entry = Entry {
        id: id.clone(),
        entry_type: et,
        actor,
        date: Utc::now(),
        title: title.to_string(),
        scope: scope.map(|s| s.to_string()),
        content: String::new(),
        supersedes: supersedes.map(|s| s.to_string()),
        reversed_by: None,
    };

    journal::write(root, &entry)?;

    // If reverses is set, update the target entry
    if let Some(target_id) = reverses {
        journal::update_reversed_by(root, target_id, &id)?;
    }

    Ok(id)
}

/// Public entry type used for list/show results (mirrors Go's pkg/arx/entry.go)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublicEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub actor: String,
    pub date: chrono::DateTime<Utc>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverses: Option<String>,
    pub state: String,
}

fn convert_entry(e: &Entry, state: &EntryState) -> PublicEntry {
    PublicEntry {
        id: e.id.clone(),
        entry_type: e.entry_type.to_string(),
        actor: e.actor.to_string(),
        date: e.date,
        title: e.title.clone(),
        scope: e.scope.clone(),
        body: if e.content.is_empty() {
            None
        } else {
            Some(e.content.clone())
        },
        supersedes: e.supersedes.clone(),
        reverses: e.reversed_by.clone(),
        state: state.to_string(),
    }
}

/// List options for filtering
pub struct ListOptions {
    pub entry_type: Option<String>,
    pub state: Option<String>,
    pub scope: Option<String>,
}

/// List returns entries with optional filtering.
pub fn list(root: &Path, opts: &ListOptions) -> Result<Vec<PublicEntry>, ArxError> {
    let state = journal::get_state(root)?;

    let mut result = Vec::new();
    for ews in state.values() {
        let entry = convert_entry(&ews.entry, &ews.state);

        if let Some(ref t) = opts.entry_type {
            if entry.entry_type != *t {
                continue;
            }
        }
        if let Some(ref s) = opts.state {
            if entry.state != *s {
                continue;
            }
        }
        if let Some(ref sc) = opts.scope {
            if entry.scope.as_deref() != Some(sc.as_str()) {
                continue;
            }
        }

        result.push(entry);
    }

    Ok(result)
}

/// Show returns a single entry by ID.
pub fn show(root: &Path, id: &str) -> Result<PublicEntry, ArxError> {
    let entry = journal::read_by_id(root, id)?;
    let state_map = journal::get_state(root)?;

    let entry_state = state_map
        .get(id)
        .map(|ews| ews.state.clone())
        .unwrap_or(EntryState::Active);

    Ok(convert_entry(&entry, &entry_state))
}

/// Search journal entries using BM25F scoring.
pub fn search_entries(
    root: &Path,
    query: &str,
    opts: &SearchOptions,
) -> Result<Vec<SearchResult>, ArxError> {
    search::search(root, query, opts)
}

/// Result of a compaction operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompactResult {
    pub moved: usize,
    pub remaining: usize,
}

/// Compact moves eligible entries from journal/*.md to archive.jsonl.
///
/// Entries are eligible if:
/// - Their state is superseded or reversed, OR
/// - They are older than `older_than_days` days
pub fn compact(root: &Path, older_than_days: u32) -> Result<CompactResult, ArxError> {
    let state_map = journal::get_state(root)?;
    let cutoff = Utc::now() - chrono::Duration::days(older_than_days as i64);

    let mut to_archive = Vec::new();
    let mut to_remove_ids = Vec::new();

    // Only consider .md entries (not already-archived ones)
    let md_entries = journal::read_all_md(root)?;
    for entry in &md_entries {
        let ews = state_map.get(&entry.id);
        let entry_state = ews.map(|e| e.state.clone()).unwrap_or(EntryState::Active);

        let eligible = matches!(entry_state, EntryState::Superseded | EntryState::Reversed)
            || entry.date < cutoff;

        if eligible {
            to_archive.push(ArchiveEntry::from_entry(entry, &entry_state));
            to_remove_ids.push(entry.id.clone());
        }
    }

    let moved = to_archive.len();

    if moved > 0 {
        archive::append_to_archive(root, &to_archive)?;
        for id in &to_remove_ids {
            journal::delete_entry_file(root, id)?;
        }
    }

    let remaining = md_entries.len() - moved;
    Ok(CompactResult { moved, remaining })
}

/// Supersede creates a new entry that supersedes an existing entry.
pub fn supersede(
    root: &Path,
    old_id: &str,
    entry_type: &str,
    actor: Option<&str>,
    title: &str,
    scope: Option<&str>,
) -> Result<String, ArxError> {
    // Verify the old entry exists
    journal::read_by_id(root, old_id)?;

    record(root, entry_type, actor, title, scope, Some(old_id), None)
}

/// Reverse creates a tombstone entry that reverses an existing entry.
pub fn reverse(
    root: &Path,
    target_id: &str,
    reason: &str,
    actor: Option<&str>,
) -> Result<String, ArxError> {
    // Verify the target entry exists
    journal::read_by_id(root, target_id)?;

    record(
        root,
        "tombstone",
        actor,
        reason,
        None,
        None,
        Some(target_id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::TempDir;

    #[test]
    fn test_compact_moves_superseded() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let old_id = record(root, "decision", None, "Old decision", None, None, None).unwrap();
        let _new_id = record(
            root,
            "decision",
            None,
            "New decision",
            None,
            Some(&old_id),
            None,
        )
        .unwrap();

        let result = compact(root, 365).unwrap();
        assert_eq!(result.moved, 1); // old entry moved
        assert_eq!(result.remaining, 1); // new entry stays

        // Verify archive has the entry
        let archived = archive::read_archive(root).unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].id, old_id);
        assert_eq!(archived[0].state, "superseded");

        // Verify .md file is gone
        let md_path = journal::journal_dir(root).join(format!("{old_id}.md"));
        assert!(!md_path.exists());
    }

    #[test]
    fn test_compact_moves_reversed() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let id = record(
            root,
            "assumption",
            None,
            "Redis available",
            None,
            None,
            None,
        )
        .unwrap();
        let _tombstone = reverse(root, &id, "Redis not needed", None).unwrap();

        let result = compact(root, 365).unwrap();
        assert_eq!(result.moved, 1); // reversed entry moved
        assert_eq!(result.remaining, 1); // tombstone stays
    }

    #[test]
    fn test_compact_moves_old_entries() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create entry with old date
        let entry = Entry {
            id: "decision-2025-01-01-abc123".to_string(),
            entry_type: EntryType::Decision,
            actor: ActorType::Human,
            date: chrono::Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(),
            title: "Old decision".to_string(),
            scope: None,
            content: String::new(),
            supersedes: None,
            reversed_by: None,
        };
        journal::write(root, &entry).unwrap();

        // Create recent entry
        record(root, "decision", None, "Recent decision", None, None, None).unwrap();

        let result = compact(root, 30).unwrap();
        assert_eq!(result.moved, 1);
        assert_eq!(result.remaining, 1);
    }

    #[test]
    fn test_compact_leaves_active_recent() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        record(root, "decision", None, "Recent active", None, None, None).unwrap();
        record(root, "assumption", None, "Also recent", None, None, None).unwrap();

        let result = compact(root, 30).unwrap();
        assert_eq!(result.moved, 0);
        assert_eq!(result.remaining, 2);
    }

    #[test]
    fn test_supersede_creates_entry() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let old_id = record(root, "decision", None, "Old", None, None, None).unwrap();
        let new_id = supersede(root, &old_id, "decision", None, "New decision", None).unwrap();

        let entry = show(root, &new_id).unwrap();
        assert_eq!(entry.supersedes, Some(old_id.clone()));

        let old = show(root, &old_id).unwrap();
        assert_eq!(old.state, "superseded");
    }

    #[test]
    fn test_supersede_nonexistent_fails() {
        let tmp = TempDir::new().unwrap();
        let result = supersede(tmp.path(), "nonexistent", "decision", None, "New", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_reverse_creates_tombstone() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let id = record(
            root,
            "assumption",
            None,
            "Redis available",
            None,
            None,
            None,
        )
        .unwrap();
        let tombstone_id = reverse(root, &id, "No longer valid", None).unwrap();

        let tombstone = show(root, &tombstone_id).unwrap();
        assert_eq!(tombstone.entry_type, "tombstone");

        let original = show(root, &id).unwrap();
        assert_eq!(original.state, "reversed");
    }

    #[test]
    fn test_reverse_nonexistent_fails() {
        let tmp = TempDir::new().unwrap();
        let result = reverse(tmp.path(), "nonexistent", "reason", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_all_merges_journal_and_archive() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Add an .md entry
        record(root, "decision", None, "Active decision", None, None, None).unwrap();

        // Add an archived entry
        let archived = ArchiveEntry {
            id: "assumption-2026-01-01-aaa".to_string(),
            entry_type: "assumption".to_string(),
            actor: "human".to_string(),
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap(),
            title: "Archived assumption".to_string(),
            scope: None,
            content: String::new(),
            supersedes: None,
            reversed_by: None,
            state: "superseded".to_string(),
        };
        archive::append_to_archive(root, &[archived]).unwrap();

        let all = journal::read_all(root).unwrap();
        assert_eq!(all.len(), 2);
    }
}
