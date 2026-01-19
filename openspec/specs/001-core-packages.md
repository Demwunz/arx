# Arx Core Packages

**Status:** Draft
**Created:** 2026-01-19

## Problem Statement

Build the core Go packages for journal entries and checkpoint management.

## Requirements

### Project Setup

1. The project SHALL use Go modules with module path `github.com/demwunz/arx`
2. The project SHALL use gopkg.in/yaml.v3 for YAML parsing

### Journal Package

3. Journal entries SHALL be stored in `.arx/journal/` directory
4. Each entry SHALL have a unique ID in format: `{type}-{YYYY-MM-DD}-{6-char-hex}`
5. Entry types SHALL be: `clarification`, `decision`, `override`, `blocker`, `assumption`, `risk`, `defer`, `tombstone`
6. Actor types SHALL be: `human`, `planner`, `executor`, `reviewer`, `system`
7. The journal package SHALL implement `GenerateID()` function
8. The journal package SHALL implement `Write()` to save entries with YAML frontmatter
9. The journal package SHALL implement `ReadAll()` to load all entries sorted by date
10. The journal package SHALL implement `GetState()` to compute derived state

### Checkpoint Package

11. Checkpoint SHALL be stored at `.arx/checkpoint.json`
12. Checkpoint SHALL have required fields: `version`, `task_id`, `started_at`, `last_activity`, `status`
13. The checkpoint package SHALL implement `Save()` with auto-initialization of defaults
14. The checkpoint package SHALL implement `Load()` returning nil if no checkpoint exists
15. The checkpoint package SHALL implement `Clear()` to remove checkpoint
16. The checkpoint package SHALL implement `IsStale()` with 72-hour default threshold

### Testing

17. Tests SHALL verify ID generation matches pattern `{type}-{YYYY-MM-DD}-{6-char-hex}`
18. Tests SHALL verify entry write/read round-trip preserves all fields
19. Tests SHALL verify derived state computation (active/superseded/reversed)
20. Tests SHALL verify checkpoint staleness detection

## Acceptance Criteria

1. `go build ./...` succeeds
2. `go test ./...` passes with all tests green
3. Journal entries can be written and read back
4. Checkpoint can be saved, loaded, and cleared
