use std::collections::HashMap;
use std::path::Path;

use topo_core::TermFreqs;
use topo_score::{Bm25fScorer, CorpusStats, Tokenizer};

use crate::entry::{Entry, EntryWithState};
use crate::error::ArxError;
use crate::journal;
use crate::PublicEntry;

/// Options for search filtering.
pub struct SearchOptions {
    pub entry_type: Option<String>,
    pub state: Option<String>,
    pub scope: Option<String>,
    pub limit: usize,
}

/// A scored search result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    #[serde(flatten)]
    pub entry: PublicEntry,
    pub score: f64,
}

/// Build BM25F term frequencies for a journal entry.
///
/// Maps entry fields to topo-score's BM25F fields:
/// - ID + entry type → filename (weight 5.0)
/// - title + scope → symbols (weight 3.0)
/// - content → body (weight 1.0)
fn build_term_freqs(entry: &Entry) -> (HashMap<String, TermFreqs>, u32) {
    let id_tokens = Tokenizer::tokenize(&format!("{} {}", entry.id, entry.entry_type));
    let title_tokens = Tokenizer::tokenize(&entry.title);
    let scope_tokens = entry
        .scope
        .as_deref()
        .map(Tokenizer::tokenize)
        .unwrap_or_default();
    let content_tokens = Tokenizer::tokenize(&entry.content);

    let mut freqs: HashMap<String, TermFreqs> = HashMap::new();

    // ID + type tokens → filename field
    for t in &id_tokens {
        freqs.entry(t.clone()).or_default().filename += 1;
    }
    // Title + scope → symbols field
    for t in title_tokens.iter().chain(scope_tokens.iter()) {
        freqs.entry(t.clone()).or_default().symbols += 1;
    }
    // Content → body field
    for t in &content_tokens {
        freqs.entry(t.clone()).or_default().body += 1;
    }

    let doc_len =
        (id_tokens.len() + title_tokens.len() + scope_tokens.len() + content_tokens.len()) as u32;
    (freqs, doc_len)
}

/// Search journal entries using BM25F scoring.
pub fn search(
    root: &Path,
    query: &str,
    opts: &SearchOptions,
) -> Result<Vec<SearchResult>, ArxError> {
    let state_map = journal::get_state(root)?;
    let entries: Vec<&EntryWithState> = state_map.values().collect();

    if entries.is_empty() || query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Build term freqs for each entry
    let mut doc_data: Vec<(&EntryWithState, HashMap<String, TermFreqs>, u32)> = Vec::new();
    for ews in &entries {
        let (freqs, doc_len) = build_term_freqs(&ews.entry);
        doc_data.push((ews, freqs, doc_len));
    }

    // Build corpus stats
    let stats = CorpusStats::from_documents(
        doc_data
            .iter()
            .map(|(ews, freqs, doc_len)| (ews.entry.id.as_str(), freqs, *doc_len)),
    );

    let scorer = Bm25fScorer::new(query, stats);

    // Score each entry
    let mut results: Vec<SearchResult> = Vec::new();
    for (ews, freqs, doc_len) in &doc_data {
        let score = scorer.score(freqs, *doc_len);
        if score <= 0.0 {
            continue;
        }

        let entry = crate::convert_entry(&ews.entry, &ews.state);

        // Apply filters
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

        results.push(SearchResult { entry, score });
    }

    // Sort by score descending
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply limit
    if opts.limit > 0 && results.len() > opts.limit {
        results.truncate(opts.limit);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{ActorType, EntryType};
    use chrono::TimeZone;
    use tempfile::TempDir;

    fn add_entry(
        root: &Path,
        id_suffix: &str,
        entry_type: EntryType,
        title: &str,
        content: &str,
        scope: Option<&str>,
    ) {
        let entry = Entry {
            id: format!("{}-2026-01-15-{id_suffix}", entry_type),
            entry_type,
            actor: ActorType::Human,
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            title: title.to_string(),
            scope: scope.map(|s| s.to_string()),
            content: content.to_string(),
            supersedes: None,
            reversed_by: None,
        };
        journal::write(root, &entry).unwrap();
    }

    #[test]
    fn search_finds_entry_by_title() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        add_entry(
            root,
            "aaa",
            EntryType::Decision,
            "Use PostgreSQL for persistence",
            "",
            None,
        );
        add_entry(
            root,
            "bbb",
            EntryType::Assumption,
            "Redis is available",
            "",
            None,
        );

        let opts = SearchOptions {
            entry_type: None,
            state: None,
            scope: None,
            limit: 0,
        };
        let results = search(root, "PostgreSQL", &opts).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].entry.title, "Use PostgreSQL for persistence");
    }

    #[test]
    fn search_finds_entry_by_content() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        add_entry(
            root,
            "aaa",
            EntryType::Decision,
            "Database choice",
            "We chose PostgreSQL because of JSONB support",
            None,
        );
        add_entry(
            root,
            "bbb",
            EntryType::Decision,
            "Cache strategy",
            "Use in-memory caching",
            None,
        );

        let opts = SearchOptions {
            entry_type: None,
            state: None,
            scope: None,
            limit: 0,
        };
        let results = search(root, "PostgreSQL", &opts).unwrap();

        assert!(!results.is_empty());
        assert!(results[0].entry.title == "Database choice");
    }

    #[test]
    fn search_filters_by_type() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        add_entry(root, "aaa", EntryType::Decision, "Use PostgreSQL", "", None);
        add_entry(
            root,
            "bbb",
            EntryType::Assumption,
            "PostgreSQL is fast",
            "",
            None,
        );

        let opts = SearchOptions {
            entry_type: Some("decision".to_string()),
            state: None,
            scope: None,
            limit: 0,
        };
        let results = search(root, "PostgreSQL", &opts).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.entry_type, "decision");
    }

    #[test]
    fn search_filters_by_scope() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        add_entry(
            root,
            "aaa",
            EntryType::Decision,
            "Use PostgreSQL",
            "",
            Some("backend"),
        );
        add_entry(
            root,
            "bbb",
            EntryType::Decision,
            "Use PostgreSQL too",
            "",
            Some("frontend"),
        );

        let opts = SearchOptions {
            entry_type: None,
            state: None,
            scope: Some("backend".to_string()),
            limit: 0,
        };
        let results = search(root, "PostgreSQL", &opts).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.scope, Some("backend".to_string()));
    }

    #[test]
    fn search_respects_limit() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        add_entry(root, "aaa", EntryType::Decision, "PostgreSQL one", "", None);
        add_entry(root, "bbb", EntryType::Decision, "PostgreSQL two", "", None);
        add_entry(
            root,
            "ccc",
            EntryType::Decision,
            "PostgreSQL three",
            "",
            None,
        );

        let opts = SearchOptions {
            entry_type: None,
            state: None,
            scope: None,
            limit: 2,
        };
        let results = search(root, "PostgreSQL", &opts).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_title_ranks_higher_than_body() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Entry with "PostgreSQL" in title (symbols field, weight 3.0)
        add_entry(
            root,
            "aaa",
            EntryType::Decision,
            "PostgreSQL decision",
            "some other content",
            None,
        );
        // Entry with "PostgreSQL" only in body (body field, weight 1.0)
        add_entry(
            root,
            "bbb",
            EntryType::Decision,
            "Database choice",
            "We use PostgreSQL here",
            None,
        );

        let opts = SearchOptions {
            entry_type: None,
            state: None,
            scope: None,
            limit: 0,
        };
        let results = search(root, "PostgreSQL", &opts).unwrap();

        assert!(results.len() >= 2);
        // Title match should rank higher
        assert!(results[0].entry.title.contains("PostgreSQL"));
    }

    #[test]
    fn search_empty_query_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        add_entry(root, "aaa", EntryType::Decision, "Test", "", None);

        let opts = SearchOptions {
            entry_type: None,
            state: None,
            scope: None,
            limit: 0,
        };
        let results = search(root, "", &opts).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_no_match_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        add_entry(root, "aaa", EntryType::Decision, "Use PostgreSQL", "", None);

        let opts = SearchOptions {
            entry_type: None,
            state: None,
            scope: None,
            limit: 0,
        };
        let results = search(root, "zebra", &opts).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_filters_by_state() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create two entries, one superseding the other
        let old_entry = Entry {
            id: "decision-2026-01-15-old".to_string(),
            entry_type: EntryType::Decision,
            actor: ActorType::Human,
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            title: "Use PostgreSQL old".to_string(),
            scope: None,
            content: String::new(),
            supersedes: None,
            reversed_by: None,
        };
        journal::write(root, &old_entry).unwrap();

        let new_entry = Entry {
            id: "decision-2026-01-15-new".to_string(),
            entry_type: EntryType::Decision,
            actor: ActorType::Human,
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 16, 10, 0, 0).unwrap(),
            title: "Use PostgreSQL new".to_string(),
            scope: None,
            content: String::new(),
            supersedes: Some("decision-2026-01-15-old".to_string()),
            reversed_by: None,
        };
        journal::write(root, &new_entry).unwrap();

        // Search for active only
        let opts = SearchOptions {
            entry_type: None,
            state: Some("active".to_string()),
            scope: None,
            limit: 0,
        };
        let results = search(root, "PostgreSQL", &opts).unwrap();

        // Only the new (active) entry should appear
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.state, "active");
    }
}
