# Extended Public API

**Status:** Draft
**Created:** 2026-01-20

## Problem Statement

The core public API (spec 002) provides `record`, `list`, `show`. Additional functions are needed for checkpoint management and entry relationships (supersede/reverse).

## Requirements

### Checkpoint Types

1. CheckpointStatus SHALL be an enum with variants: `InProgress`, `Completed`, `Failed`, `Paused`
2. Checkpoint SHALL have fields: version, task_id, started_at, last_activity, status

### Checkpoint Functions

3. `save()` SHALL save checkpoint to `.arx/checkpoint.json`
4. `load()` SHALL return checkpoint or `None` if none exists
5. `clear()` SHALL remove the checkpoint file
6. `is_stale()` SHALL return true if last activity exceeds 72 hours

### Search Function

7. `search(query, SearchOptions)` SHALL return entries matching query in title or body
8. Search SHALL support filtering by type, state, scope
9. Search SHALL support limit option (0 = no limit)

### Relationship Functions

10. Supersede(new_entry, old_id) SHALL create entry with supersedes field set
11. Reverse(target_id, reason) SHALL create tombstone entry that reverses target

## Acceptance Criteria

1. `cargo build -p arx` succeeds
2. Checkpoint can be saved, loaded, cleared
3. `is_stale()` correctly detects stale checkpoints
4. Search finds entries by query text
5. Supersede marks old entry as superseded
6. Reverse marks target entry as reversed
