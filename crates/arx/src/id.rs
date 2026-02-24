use chrono::Utc;
use rand::Rng;

use crate::entry::EntryType;
use crate::error::ArxError;

pub fn generate_id(entry_type: &EntryType) -> Result<String, ArxError> {
    let bytes: [u8; 3] = rand::thread_rng().gen();
    let date_str = Utc::now().format("%Y-%m-%d");
    let hex_str = hex::encode(bytes);
    Ok(format!("{}-{}-{}", entry_type, date_str, hex_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::collections::HashSet;

    #[test]
    fn test_generate_id_all_types() {
        let types = [
            EntryType::Clarification,
            EntryType::Decision,
            EntryType::Override,
            EntryType::Blocker,
            EntryType::Assumption,
            EntryType::Risk,
            EntryType::Defer,
            EntryType::Tombstone,
        ];

        let pattern = Regex::new(r"^[a-z]+-\d{4}-\d{2}-\d{2}-[0-9a-f]{6}$").unwrap();

        for t in &types {
            let id = generate_id(t).unwrap();
            assert!(pattern.is_match(&id), "ID {id:?} does not match pattern");
            let expected_prefix = format!("{}-", t);
            assert!(
                id.starts_with(&expected_prefix),
                "ID {id:?} missing prefix {expected_prefix:?}"
            );
        }
    }

    #[test]
    fn test_generate_id_uniqueness() {
        let mut ids = HashSet::new();
        for _ in 0..100 {
            let id = generate_id(&EntryType::Decision).unwrap();
            assert!(ids.insert(id.clone()), "Duplicate ID: {id}");
        }
    }
}
