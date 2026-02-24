# Public API

**Status:** Implemented
**Created:** 2026-01-20

## Problem Statement

Enable external Rust projects to use arx as a library via the public API in `crates/arx/src/lib.rs`.

## Requirements

### Crate Location

1. The public API SHALL be in the `arx` crate at `crates/arx/`
2. The crate SHALL re-export core modules: `entry`, `journal`, `checkpoint`, `id`, `error`

### Entry Types

3. EntryType SHALL be an enum with variants: `Decision`, `Assumption`, `Clarification`, `Blocker`, `Risk`, `Override`, `Defer`, `Tombstone`
4. EntryState SHALL be an enum with variants: `Active`, `Superseded`, `Reversed`
5. Entry SHALL have fields: id, entry_type, actor, date, title, scope, content, supersedes, reversed_by

### Core Functions

6. `record()` SHALL create a new entry and return its ID
7. `list()` SHALL return entries with optional filtering by type, state, scope
8. `show()` SHALL return a single entry by ID or `ArxError::NotFound`

### Options Types

9. ListOptions SHALL have fields: entry_type, state, scope

### Errors

11. `ArxError::NotFound` SHALL be returned when entry doesn't exist
12. `ArxError::InvalidType` SHALL be returned for invalid entry types

## Acceptance Criteria

1. `cargo build -p arx` succeeds
2. External crates can depend on `arx = { path = "crates/arx" }`
3. `record()` creates entries in `.arx/journal/`
4. `list()` returns filtered entries
5. `show()` returns single entry or error
