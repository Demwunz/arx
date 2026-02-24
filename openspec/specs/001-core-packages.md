# Arx Core Packages

**Status:** Implemented
**Created:** 2026-01-19

## Problem Statement

Build the core library for journal entries and checkpoint management.

## Requirements

### Project Setup

1. The project SHALL use a Cargo workspace with the core library at `crates/arx/`
2. The project SHALL use serde_yml for YAML parsing

### Journal Module

3. Journal entries SHALL be stored in `.arx/journal/` directory
4. Each entry SHALL have a unique ID in format: `{type}-{YYYY-MM-DD}-{6-char-hex}`
5. Entry types SHALL be: `clarification`, `decision`, `override`, `blocker`, `assumption`, `risk`, `defer`, `tombstone`
6. Actor types SHALL be: `human`, `planner`, `executor`, `reviewer`, `system`
7. The journal module SHALL implement `generate_id()` function
8. The journal module SHALL implement `write()` to save entries with YAML frontmatter
9. The journal module SHALL implement `read_all()` to load all entries sorted by date
10. The journal module SHALL implement `get_state()` to compute derived state

### Checkpoint Module

11. Checkpoint SHALL be stored at `.arx/checkpoint.json`
12. Checkpoint SHALL have required fields: `version`, `task_id`, `started_at`, `last_activity`, `status`
13. The checkpoint module SHALL implement `save()` with auto-initialization of defaults
14. The checkpoint module SHALL implement `load()` returning `None` if no checkpoint exists
15. The checkpoint module SHALL implement `clear()` to remove checkpoint
16. The checkpoint module SHALL implement `is_stale()` with 72-hour default threshold

### Testing

17. Tests SHALL verify ID generation matches pattern `{type}-{YYYY-MM-DD}-{6-char-hex}`
18. Tests SHALL verify entry write/read round-trip preserves all fields
19. Tests SHALL verify derived state computation (active/superseded/reversed)
20. Tests SHALL verify checkpoint staleness detection

## Acceptance Criteria

1. `cargo build -p arx` succeeds
2. `cargo test -p arx` passes with all tests green
3. Journal entries can be written and read back
4. Checkpoint can be saved, loaded, and cleared
