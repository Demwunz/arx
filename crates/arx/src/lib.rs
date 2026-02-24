pub mod checkpoint;
pub mod entry;
pub mod error;
pub mod id;
pub mod journal;

use std::path::Path;

use chrono::Utc;

use entry::{ActorType, Entry, EntryState, EntryType};
use error::ArxError;

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
