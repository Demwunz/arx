use std::path::{Path, PathBuf};

use arx::checkpoint;
use arx::entry::{EntryState, EntryType};
use arx::journal;
use chrono::Utc;
use clap::{Parser, Subcommand};

const RESUME_PROMPT_PATH: &str = ".arx/resume-prompt.md";

#[derive(Parser)]
#[command(name = "arx", about = "Journal and checkpoint management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new journal entry
    Add {
        /// Entry type
        #[arg(value_name = "TYPE")]
        entry_type: String,
        /// Entry message
        #[arg(value_name = "MESSAGE")]
        message: String,
        /// Entry scope
        #[arg(long)]
        scope: Option<String>,
        /// ID of entry this supersedes
        #[arg(long)]
        supersedes: Option<String>,
        /// ID of entry this reverses
        #[arg(long)]
        reverses: Option<String>,
    },
    /// List all journal entries
    List {
        /// Filter by entry type
        #[arg(long = "type")]
        type_filter: Option<String>,
        /// Filter by state (active, superseded, reversed)
        #[arg(long = "state")]
        state_filter: Option<String>,
    },
    /// Show a single journal entry
    Show {
        /// The entry ID to display
        id: String,
    },
    /// Manage checkpoint state
    Checkpoint {
        /// Subcommand: show (default) or clear
        #[arg(default_value = "show")]
        subcommand: String,
    },
    /// Search journal entries
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
        /// Filter by entry type
        #[arg(long = "type")]
        type_filter: Option<String>,
        /// Filter by state (active, superseded, reversed)
        #[arg(long = "state")]
        state_filter: Option<String>,
        /// Filter by scope
        #[arg(long)]
        scope: Option<String>,
        /// Maximum number of results (0 = no limit)
        #[arg(long, default_value = "0")]
        limit: usize,
    },
    /// Compact old/inactive entries into archive.jsonl
    Compact {
        /// Move entries older than this many days (default: 30)
        #[arg(long = "older-than", default_value = "30")]
        older_than: u32,
    },
    /// Create a new entry that supersedes an existing one
    Supersede {
        /// ID of the entry to supersede
        #[arg(value_name = "OLD_ID")]
        old_id: String,
        /// Type of the new entry
        #[arg(long = "type")]
        entry_type: String,
        /// Message for the new entry
        #[arg(long = "message", short = 'm')]
        message: String,
        /// Scope for the new entry
        #[arg(long)]
        scope: Option<String>,
    },
    /// Reverse an existing entry by creating a tombstone
    Reverse {
        /// ID of the entry to reverse
        #[arg(value_name = "TARGET_ID")]
        target_id: String,
        /// Reason for reversal
        #[arg(long)]
        reason: String,
    },
    /// Generate resume context
    Resume {
        /// Print to stdout instead of file
        #[arg(long)]
        print: bool,
    },
}

fn main() {
    let root = PathBuf::from(".");
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Add {
            entry_type,
            message,
            scope,
            supersedes,
            reverses,
        } => cmd_add(
            &root,
            &entry_type,
            &message,
            scope.as_deref(),
            supersedes.as_deref(),
            reverses.as_deref(),
        ),
        Commands::List {
            type_filter,
            state_filter,
        } => cmd_list(&root, type_filter.as_deref(), state_filter.as_deref()),
        Commands::Show { id } => cmd_show(&root, &id),
        Commands::Search {
            query,
            type_filter,
            state_filter,
            scope,
            limit,
        } => cmd_search(
            &root,
            &query,
            type_filter.as_deref(),
            state_filter.as_deref(),
            scope.as_deref(),
            limit,
        ),
        Commands::Compact { older_than } => cmd_compact(&root, older_than),
        Commands::Supersede {
            old_id,
            entry_type,
            message,
            scope,
        } => cmd_supersede(&root, &old_id, &entry_type, &message, scope.as_deref()),
        Commands::Reverse { target_id, reason } => cmd_reverse(&root, &target_id, &reason),
        Commands::Checkpoint { subcommand } => cmd_checkpoint(&root, &subcommand),
        Commands::Resume { print } => cmd_resume(&root, print),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_add(
    root: &Path,
    entry_type: &str,
    message: &str,
    scope: Option<&str>,
    supersedes: Option<&str>,
    reverses: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if entry_type.parse::<EntryType>().is_err() {
        return Err(format!(
            "invalid entry type {:?}; valid types: {}",
            entry_type,
            EntryType::all().join(", ")
        )
        .into());
    }

    if message.trim().is_empty() {
        return Err("message cannot be empty".into());
    }

    let id = arx::record(root, entry_type, None, message, scope, supersedes, reverses)?;
    println!("Created entry: {id}");
    Ok(())
}

fn cmd_list(
    root: &Path,
    type_filter: Option<&str>,
    state_filter: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = journal::get_state(root)?;

    let mut entries: Vec<_> = state.into_values().collect();
    // Sort by date, newest first
    entries.sort_by(|a, b| b.entry.date.cmp(&a.entry.date));

    for e in &entries {
        if let Some(t) = type_filter {
            if e.entry.entry_type.to_string() != t {
                continue;
            }
        }
        if let Some(s) = state_filter {
            if e.state.to_string() != s {
                continue;
            }
        }

        println!(
            "[{}] {} ({}) - {}",
            e.state, e.entry.id, e.entry.entry_type, e.entry.title
        );
    }

    Ok(())
}

fn cmd_show(root: &Path, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entry = journal::read_by_id(root, id)?;
    let state_map = journal::get_state(root)?;

    let entry_state = state_map
        .get(id)
        .map(|ews| &ews.state)
        .unwrap_or(&EntryState::Active);

    println!("ID:          {}", entry.id);
    println!("Type:        {}", entry.entry_type);
    println!("Actor:       {}", entry.actor);
    println!("Date:        {}", entry.date.format("%Y-%m-%dT%H:%M:%SZ"));
    println!("Title:       {}", entry.title);
    println!("State:       {entry_state}");
    if let Some(ref scope) = entry.scope {
        println!("Scope:       {scope}");
    }
    if let Some(ref s) = entry.supersedes {
        println!("Supersedes:  {s}");
    }
    if let Some(ref r) = entry.reversed_by {
        println!("Reversed By: {r}");
    }
    if !entry.content.is_empty() {
        println!("\nContent:\n{}", entry.content);
    }

    Ok(())
}

fn cmd_search(
    root: &Path,
    query: &str,
    type_filter: Option<&str>,
    state_filter: Option<&str>,
    scope: Option<&str>,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let opts = arx::SearchOptions {
        entry_type: type_filter.map(|s| s.to_string()),
        state: state_filter.map(|s| s.to_string()),
        scope: scope.map(|s| s.to_string()),
        limit,
    };

    let results = arx::search_entries(root, query, &opts)?;

    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    for r in &results {
        println!(
            "[{:.3}] [{}] {} ({}) - {}",
            r.score, r.entry.state, r.entry.id, r.entry.entry_type, r.entry.title
        );
    }

    Ok(())
}

fn cmd_compact(root: &Path, older_than: u32) -> Result<(), Box<dyn std::error::Error>> {
    let result = arx::compact(root, older_than)?;
    println!(
        "Compacted: {} entries moved to archive, {} entries remaining in journal/",
        result.moved, result.remaining
    );
    Ok(())
}

fn cmd_supersede(
    root: &Path,
    old_id: &str,
    entry_type: &str,
    message: &str,
    scope: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let new_id = arx::supersede(root, old_id, entry_type, None, message, scope)?;
    println!("Created entry: {new_id} (supersedes {old_id})");
    Ok(())
}

fn cmd_reverse(
    root: &Path,
    target_id: &str,
    reason: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tombstone_id = arx::reverse(root, target_id, reason, None)?;
    println!("Created tombstone: {tombstone_id} (reverses {target_id})");
    Ok(())
}

fn cmd_checkpoint(root: &Path, subcommand: &str) -> Result<(), Box<dyn std::error::Error>> {
    match subcommand {
        "show" => {
            let cp = checkpoint::load(root)?;
            match cp {
                None => println!("No checkpoint found."),
                Some(cp) => {
                    println!("Version:       {}", cp.version);
                    println!("Task ID:       {}", cp.task_id);
                    println!("Status:        {}", cp.status);
                    println!(
                        "Started At:    {}",
                        cp.started_at.format("%Y-%m-%dT%H:%M:%SZ")
                    );
                    println!(
                        "Last Activity: {}",
                        cp.last_activity.format("%Y-%m-%dT%H:%M:%SZ")
                    );
                    if checkpoint::is_stale(&cp) {
                        println!("\n\u{26a0}\u{fe0f}  WARNING: Checkpoint is stale (last activity > 72 hours ago)");
                    }
                }
            }
        }
        "clear" => {
            checkpoint::clear(root)?;
            println!("Checkpoint cleared.");
        }
        other => {
            return Err(format!("unknown subcommand: {other}; use 'show' or 'clear'").into());
        }
    }
    Ok(())
}

fn cmd_resume(root: &Path, print_flag: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = String::new();

    sb.push_str("# Resume Context\n\n");
    sb.push_str(&format!(
        "Generated: {}\n\n",
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    ));

    // Checkpoint status
    sb.push_str("## Checkpoint\n\n");
    match checkpoint::load(root) {
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
    match journal::get_state(root) {
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

    if print_flag {
        print!("{sb}");
    } else {
        let path = root.join(RESUME_PROMPT_PATH);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, &sb)?;
        println!("Resume context written to {RESUME_PROMPT_PATH}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cmd_add_success() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_add(root, "decision", "Test decision message", None, None, None).unwrap();
        let entries = journal::read_all(root).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Test decision message");
        assert_eq!(entries[0].entry_type, EntryType::Decision);
    }

    #[test]
    fn test_cmd_add_with_scope() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_add(
            root,
            "assumption",
            "API will use REST",
            Some("backend"),
            None,
            None,
        )
        .unwrap();
        let entries = journal::read_all(root).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scope, Some("backend".to_string()));
    }

    #[test]
    fn test_cmd_add_invalid_type() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let result = cmd_add(root, "invalid", "Some message", None, None, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid entry type"));
    }

    #[test]
    fn test_cmd_add_empty_message() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let result = cmd_add(root, "decision", "   ", None, None, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("message cannot be empty"));
    }

    #[test]
    fn test_cmd_add_missing_args() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        // Empty message is the closest analog
        let result = cmd_add(root, "decision", "", None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_add_supersedes() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_add(root, "decision", "Original decision", None, None, None).unwrap();
        let entries = journal::read_all(root).unwrap();
        let first_id = entries[0].id.clone();

        cmd_add(
            root,
            "decision",
            "Updated decision",
            None,
            Some(&first_id),
            None,
        )
        .unwrap();
        let entries = journal::read_all(root).unwrap();
        assert_eq!(entries.len(), 2);

        let updated = entries
            .iter()
            .find(|e| e.title == "Updated decision")
            .unwrap();
        assert_eq!(updated.supersedes, Some(first_id));
    }

    #[test]
    fn test_cmd_list_empty() {
        let tmp = TempDir::new().unwrap();
        cmd_list(tmp.path(), None, None).unwrap();
    }

    #[test]
    fn test_cmd_list_with_entries() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_add(root, "decision", "First decision", None, None, None).unwrap();
        cmd_add(root, "assumption", "First assumption", None, None, None).unwrap();
        cmd_list(root, None, None).unwrap();
    }

    #[test]
    fn test_cmd_list_type_filter() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_add(root, "decision", "A decision", None, None, None).unwrap();
        cmd_add(root, "assumption", "An assumption", None, None, None).unwrap();
        cmd_list(root, Some("decision"), None).unwrap();
    }

    #[test]
    fn test_cmd_show_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = cmd_show(tmp.path(), "nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_show_success() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_add(root, "decision", "Test decision", None, None, None).unwrap();
        let entries = journal::read_all(root).unwrap();
        cmd_show(root, &entries[0].id).unwrap();
    }

    #[test]
    fn test_cmd_checkpoint_show_no_checkpoint() {
        let tmp = TempDir::new().unwrap();
        cmd_checkpoint(tmp.path(), "show").unwrap();
    }

    #[test]
    fn test_cmd_checkpoint_show_default() {
        let tmp = TempDir::new().unwrap();
        cmd_checkpoint(tmp.path(), "show").unwrap();
    }

    #[test]
    fn test_cmd_checkpoint_clear() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let mut cp = checkpoint::Checkpoint::new("test-task".to_string());
        checkpoint::save(root, &mut cp).unwrap();

        cmd_checkpoint(root, "clear").unwrap();

        let loaded = checkpoint::load(root).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_cmd_checkpoint_invalid_subcommand() {
        let tmp = TempDir::new().unwrap();
        let result = cmd_checkpoint(tmp.path(), "invalid");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown subcommand"));
    }

    #[test]
    fn test_cmd_resume_print() {
        let tmp = TempDir::new().unwrap();
        cmd_resume(tmp.path(), true).unwrap();
    }

    #[test]
    fn test_cmd_resume_write_file() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_resume(root, false).unwrap();
        assert!(root.join(RESUME_PROMPT_PATH).exists());
    }

    #[test]
    fn test_cmd_resume_with_data() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        cmd_add(root, "decision", "Test decision", None, None, None).unwrap();
        let mut cp = checkpoint::Checkpoint::new("test-task".to_string());
        checkpoint::save(root, &mut cp).unwrap();
        cmd_resume(root, true).unwrap();
    }

    #[test]
    fn test_all_valid_entry_types() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        for entry_type in EntryType::all() {
            cmd_add(
                root,
                entry_type,
                &format!("Test {entry_type}"),
                None,
                None,
                None,
            )
            .unwrap();
        }
    }
}
