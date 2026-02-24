# arx Specification

The full format specification for arx decision records.

## Directory Structure

```
.arx/
├── journal/              # Decision entries (append-only)
│   └── *.md
├── archive.jsonl         # Compacted inactive entries (append-only)
├── checkpoint.json       # Session state (overwrite)
└── resume-prompt.md      # Generated context
```

## Journal Entry Format

Each entry is a markdown file with YAML frontmatter:

```yaml
---
id: {type}-{YYYY-MM-DD}-{6-char-hex}
type: decision | assumption | clarification | blocker | risk | override | defer | tombstone
actor: human | system
date: {ISO 8601}
title: {short description}
scope: {optional, freeform}
supersedes: {optional, id of entry this replaces}
reverses: {optional, id of entry this undoes}
---

{optional body with context}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier: `{type}-{date}-{hex}` |
| `type` | Yes | Entry type (see below) |
| `actor` | Yes | `human` or `system` |
| `date` | Yes | ISO 8601 timestamp |
| `title` | Yes | Short description |
| `scope` | No | Area affected (freeform) |
| `supersedes` | No | ID of entry this replaces |
| `reverses` | No | ID of entry this undoes |

### Entry Types

| Type | When to Use |
|------|-------------|
| `decision` | Choosing between alternatives |
| `assumption` | Proceeding on something unverified |
| `clarification` | Recording an answer to a question |
| `blocker` | Something preventing progress |
| `risk` | A potential problem being acknowledged |
| `override` | Going against a previous decision or recommendation |
| `defer` | Postponing something for later |
| `tombstone` | Marking as removed/deprecated |

## Relationships

Entries can link to previous entries:

| Link | Meaning | When to Use |
|------|---------|-------------|
| `supersedes` | Replaces | Requirements changed, better approach found |
| `reverses` | Undoes | Approach failed, mistake discovered |

**Critical: Backward links only.** New entries point to old entries. Old entries are never modified.

## Derived State

State is computed at query time by following the link graph:

| State | Rule |
|-------|------|
| `active` | No other entry supersedes or reverses this one |
| `superseded` | Another entry has `supersedes: <this-id>` |
| `reversed` | Another entry has `reverses: <this-id>` |

## Checkpoint Format

Session position for resuming work:

```json
{
  "version": "1",
  "task_id": "implement-auth",
  "status": "in_progress",
  "started_at": "2026-01-19T10:00:00Z",
  "last_activity": "2026-01-19T14:30:00Z"
}
```

### Fields

| Field | Description |
|-------|-------------|
| `version` | Schema version |
| `task_id` | Current task identifier |
| `status` | `in_progress`, `paused`, `completed`, `failed` |
| `started_at` | When work began |
| `last_activity` | Last update |

### Staleness

Checkpoints older than 72 hours are considered stale and should prompt for confirmation before resuming.

## Principles

- **Flatfiles in git** — No database, no service
- **Immutable entries** — Never edit, only supersede or reverse
- **Computed state** — Active/superseded/reversed derived at query time
- **Backward links only** — New points to old, preserving immutability
- **Model agnostic** — Works with any AI or none
