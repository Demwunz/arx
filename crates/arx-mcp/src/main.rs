use std::path::PathBuf;

use arx::checkpoint;
use arx::entry::{ActorType, EntryState, EntryType};
use arx::id::generate_id;
use arx::journal;
use chrono::Utc;
use rmcp::model::*;
use rmcp::{schemars, tool, ServerHandler, ServiceExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
struct ArxServer {
    root: PathBuf,
}

// --- Parameter structs ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AddParams {
    #[schemars(
        description = "Entry type: clarification, decision, override, blocker, assumption, risk, defer, tombstone"
    )]
    r#type: String,
    #[schemars(description = "The entry title/message")]
    message: String,
    #[schemars(description = "Optional scope for the entry")]
    scope: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListParams {
    #[schemars(description = "Filter by entry type")]
    r#type: Option<String>,
    #[schemars(description = "Filter by state: active, superseded, reversed")]
    state: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ShowParams {
    #[schemars(description = "The entry ID to display")]
    id: String,
}

// --- Response structs (same JSON shape as Go) ---

#[derive(Serialize)]
struct AddResult {
    id: String,
}

#[derive(Serialize)]
struct ListEntry {
    id: String,
    r#type: String,
    state: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    date: String,
}

#[derive(Serialize)]
struct ListResult {
    entries: Vec<ListEntry>,
}

#[derive(Serialize)]
struct ShowResult {
    id: String,
    r#type: String,
    actor: String,
    date: String,
    title: String,
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supersedes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reversed_by: Option<String>,
}

#[derive(Serialize)]
struct CheckpointResult {
    exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_activity: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    is_stale: Option<bool>,
}

fn is_false(v: &Option<bool>) -> bool {
    !matches!(v, Some(true))
}

#[derive(Serialize)]
struct ClearResult {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct ResumeResult {
    markdown: String,
}

// --- Tool implementations ---

#[tool(tool_box)]
impl ArxServer {
    #[tool(description = "Create a new journal entry", name = "arx_add")]
    async fn add(&self, #[tool(aggr)] params: AddParams) -> String {
        let entry_type_str = &params.r#type;
        let message = &params.message;

        let et = match entry_type_str.parse::<EntryType>() {
            Ok(et) => et,
            Err(_) => {
                return format!(
                    "invalid entry type {:?}; valid types: {}",
                    entry_type_str,
                    EntryType::all().join(", ")
                );
            }
        };

        if message.trim().is_empty() {
            return "message cannot be empty".to_string();
        }

        let id = match generate_id(&et) {
            Ok(id) => id,
            Err(e) => return format!("failed to generate ID: {e}"),
        };

        let entry = arx::entry::Entry {
            id: id.clone(),
            entry_type: et,
            actor: ActorType::System,
            date: Utc::now(),
            title: message.to_string(),
            scope: params.scope.clone(),
            content: String::new(),
            supersedes: None,
            reversed_by: None,
        };

        if let Err(e) = journal::write(&self.root, &entry) {
            return format!("failed to write entry: {e}");
        }

        serde_json::to_string(&AddResult { id }).unwrap()
    }

    #[tool(
        description = "List journal entries sorted by date (newest first)",
        name = "arx_list"
    )]
    async fn list(&self, #[tool(aggr)] params: ListParams) -> String {
        let state = match journal::get_state(&self.root) {
            Ok(s) => s,
            Err(e) => return format!("failed to get journal state: {e}"),
        };

        let mut entries: Vec<_> = state.into_values().collect();
        entries.sort_by(|a, b| b.entry.date.cmp(&a.entry.date));

        let mut result_entries = Vec::new();
        for e in &entries {
            if let Some(ref t) = params.r#type {
                if e.entry.entry_type.to_string() != *t {
                    continue;
                }
            }
            if let Some(ref s) = params.state {
                if e.state.to_string() != *s {
                    continue;
                }
            }

            result_entries.push(ListEntry {
                id: e.entry.id.clone(),
                r#type: e.entry.entry_type.to_string(),
                state: e.state.to_string(),
                title: e.entry.title.clone(),
                scope: e.entry.scope.clone(),
                date: e.entry.date.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            });
        }

        serde_json::to_string(&ListResult {
            entries: result_entries,
        })
        .unwrap()
    }

    #[tool(
        description = "Display a single journal entry by ID",
        name = "arx_show"
    )]
    async fn show(&self, #[tool(aggr)] params: ShowParams) -> String {
        let entry = match journal::read_by_id(&self.root, &params.id) {
            Ok(e) => e,
            Err(e) => return format!("failed to read entry: {e}"),
        };

        let state_map = match journal::get_state(&self.root) {
            Ok(s) => s,
            Err(e) => return format!("failed to get state: {e}"),
        };

        let entry_state = state_map
            .get(&params.id)
            .map(|ews| ews.state.clone())
            .unwrap_or(EntryState::Active);

        let result = ShowResult {
            id: entry.id.clone(),
            r#type: entry.entry_type.to_string(),
            actor: entry.actor.to_string(),
            date: entry.date.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            title: entry.title.clone(),
            state: entry_state.to_string(),
            scope: entry.scope.clone(),
            content: if entry.content.is_empty() {
                None
            } else {
                Some(entry.content.clone())
            },
            supersedes: entry.supersedes.clone(),
            reversed_by: entry.reversed_by.clone(),
        };

        serde_json::to_string(&result).unwrap()
    }

    #[tool(
        description = "Display current checkpoint status",
        name = "arx_checkpoint_show"
    )]
    async fn checkpoint_show(&self) -> String {
        let cp = match checkpoint::load(&self.root) {
            Ok(cp) => cp,
            Err(e) => return format!("failed to load checkpoint: {e}"),
        };

        let result = match cp {
            None => CheckpointResult {
                exists: false,
                version: None,
                task_id: None,
                status: None,
                started_at: None,
                last_activity: None,
                is_stale: None,
            },
            Some(cp) => CheckpointResult {
                exists: true,
                version: Some(cp.version.clone()),
                task_id: Some(cp.task_id.clone()),
                status: Some(cp.status.to_string()),
                started_at: Some(cp.started_at.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                last_activity: Some(cp.last_activity.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                is_stale: Some(checkpoint::is_stale(&cp)),
            },
        };

        serde_json::to_string(&result).unwrap()
    }

    #[tool(
        description = "Remove the checkpoint file",
        name = "arx_checkpoint_clear"
    )]
    async fn checkpoint_clear(&self) -> String {
        if let Err(e) = checkpoint::clear(&self.root) {
            return format!("failed to clear checkpoint: {e}");
        }

        serde_json::to_string(&ClearResult {
            success: true,
            message: "Checkpoint cleared successfully".to_string(),
        })
        .unwrap()
    }

    #[tool(
        description = "Generate resume context as markdown",
        name = "arx_resume"
    )]
    async fn resume(&self) -> String {
        let mut sb = String::new();
        sb.push_str("# Resume Context\n\n");
        sb.push_str(&format!(
            "Generated: {}\n\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
        ));

        // Checkpoint status
        sb.push_str("## Checkpoint\n\n");
        match checkpoint::load(&self.root) {
            Err(e) => sb.push_str(&format!("Error loading checkpoint: {e}\n\n")),
            Ok(None) => sb.push_str("No active checkpoint.\n\n"),
            Ok(Some(cp)) => {
                sb.push_str(&format!("- Task ID: {}\n", cp.task_id));
                sb.push_str(&format!("- Status: {}\n", cp.status));
                sb.push_str(&format!(
                    "- Last Activity: {}\n",
                    cp.last_activity.format("%Y-%m-%dT%H:%M:%SZ")
                ));
                if checkpoint::is_stale(&cp) {
                    sb.push_str("- \u{26a0}\u{fe0f}  STALE: Last activity > 72 hours ago\n");
                }
                sb.push('\n');
            }
        }

        // Active journal entries
        sb.push_str("## Active Journal Entries\n\n");
        match journal::get_state(&self.root) {
            Err(e) => sb.push_str(&format!("Error loading journal: {e}\n\n")),
            Ok(state) => {
                let mut active: Vec<_> = state
                    .into_values()
                    .filter(|e| e.state == EntryState::Active)
                    .collect();
                active.sort_by(|a, b| b.entry.date.cmp(&a.entry.date));

                if active.is_empty() {
                    sb.push_str("No active entries.\n\n");
                } else {
                    for e in &active {
                        sb.push_str(&format!("### {}\n", e.entry.id));
                        sb.push_str(&format!("- Type: {}\n", e.entry.entry_type));
                        sb.push_str(&format!(
                            "- Date: {}\n",
                            e.entry.date.format("%Y-%m-%dT%H:%M:%SZ")
                        ));
                        sb.push_str(&format!("- Title: {}\n", e.entry.title));
                        if let Some(ref scope) = e.entry.scope {
                            sb.push_str(&format!("- Scope: {scope}\n"));
                        }
                        if !e.entry.content.is_empty() {
                            sb.push_str(&format!("\n{}\n", e.entry.content));
                        }
                        sb.push('\n');
                    }
                }
            }
        }

        serde_json::to_string(&ResumeResult { markdown: sb }).unwrap()
    }
}

#[tool(tool_box)]
impl ServerHandler for ArxServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("arx - Journal and checkpoint management".into()),
            ..Default::default()
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let server = ArxServer {
        root: PathBuf::from("."),
    };
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
