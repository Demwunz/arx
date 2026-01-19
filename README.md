# arx

Conventional Commits for decisions.

---

## The Problem

AI assistants forget. Every session starts fresh. Decisions made yesterday are lost today.

You make dozens of choices during development—architecture, trade-offs, assumptions. These live only in chat history, invisible when context fills up. Six months later: *Why did we choose this approach? What were we assuming?*

**arx** is a spec for tracking decisions across sessions. Like Conventional Commits standardized commit messages, arx standardizes decision records.

---

## Quick Look

A decision is a markdown file:

```markdown
---
id: decision-2026-01-19-a1b2c3
type: decision
actor: human
date: 2026-01-19T14:30:00Z
title: Use PostgreSQL for primary storage
---

Chosen for maturity and team familiarity.
```

When requirements change, add a new entry that points back:

```markdown
---
id: decision-2026-01-19-d4e5f6
type: decision
actor: human
date: 2026-01-19T16:45:00Z
title: Switch to CockroachDB for multi-region
supersedes: decision-2026-01-19-a1b2c3
---

Requirements changed. Need multi-region support.
```

The original stays untouched. State (`active`, `superseded`, `reversed`) is computed by following links.

---

## Why arx?

| Approach | Limitation |
|----------|------------|
| **mem0** | Stores facts, not decision lifecycle |
| **RAG** | Retrieves documents, doesn't track what's still valid |
| **Chat history** | Captures everything, surfaces nothing |
| **ADRs** | Heavy process for major decisions only |

arx fills the gap: lightweight, git-native, tracks what's current vs. what's history.

---

## Principles

- **Flatfiles in git** — No database, no service
- **Immutable entries** — Never edit, only supersede or reverse
- **Computed state** — Active/superseded/reversed derived at query time
- **Backward links only** — New points to old, preserving immutability
- **Model agnostic** — Works with any AI or none

---

## The Spec

### Directory Structure

```
.arx/
├── journal/              # Decision entries (append-only)
│   └── *.md
├── checkpoint.json       # Session state (overwrite)
└── resume-prompt.md      # Generated context
```

### Entry Format

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

### Entry Types

| Type | When to Use |
|------|-------------|
| `decision` | Choosing between alternatives |
| `assumption` | Proceeding on something unverified |
| `clarification` | Recording an answer |
| `blocker` | Something preventing progress |
| `risk` | A potential problem |
| `override` | Going against a previous decision |
| `defer` | Postponing for later |
| `tombstone` | Marking as removed/deprecated |

### Relationships

| Link | Meaning | State Effect |
|------|---------|--------------|
| `supersedes` | Replaces (requirements changed) | Original → `superseded` |
| `reverses` | Undoes (approach failed) | Original → `reversed` |

Entries without incoming links are `active`.

### Checkpoint

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

---

## Getting Started

You can use arx with just files—create `.arx/journal/` and start writing markdown.

Or use the tooling:

```bash
# Install CLI
go install github.com/demwunz/arx@latest

# Add a decision
arx add decision "Use PostgreSQL for storage"

# See active decisions
arx list --state active

# Resume context for new session
arx resume --print
```

See [docs/cli.md](docs/cli.md) for full CLI reference.

---

## Integrations

| Tool | Purpose | Docs |
|------|---------|------|
| **CLI** | Human interaction | [docs/cli.md](docs/cli.md) |
| **MCP Server** | AI assistant integration | [docs/mcp.md](docs/mcp.md) |

Both are thin wrappers around the spec. The files are the source of truth.

---

## License

MIT
