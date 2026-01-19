# arx

A decision journal for AI-assisted development.

## The Problem

AI assistants forget. Every session starts fresh. Context evaporates.

When you're building software with AI help, you make dozens of decisions: architecture choices, trade-offs, assumptions about requirements. These decisions live only in chat history—scattered, unsearchable, and eventually lost to context windows.

Worse, there's no audit trail. Six months later, you can't answer: *Why did we choose REST over GraphQL? What assumptions were we making about load? Who decided to defer the caching layer?*

Traditional solutions don't fit:

- **mem0 and memory systems** store facts for retrieval—useful for "remember my preferences," less useful for "track why we made this architectural decision and whether it's still valid"
- **RAG** retrieves relevant documents to augment responses—great for knowledge bases, but decisions aren't documents to retrieve, they're living records that get superseded, reversed, or deferred
- **Chat history** captures everything but surfaces nothing—you can't query it, filter it, or see which decisions are still active

arx is different. It's a structured journal that tracks the lifecycle of project decisions: when they were made, by whom (human or AI), and whether they're still active, have been superseded, or were reversed. It's git-friendly, searchable, and designed for the messy reality of human-AI collaboration.

## Installation

```bash
go install github.com/demwunz/arx@latest
```

## Quick Start

Record a decision during your development session:

```bash
arx add decision "Use PostgreSQL for primary storage"
```

Track an assumption you're making:

```bash
arx add assumption "API will receive <1000 requests/second" --scope backend
```

Later, when requirements change, supersede the old decision:

```bash
arx add decision "Switch to CockroachDB for multi-region support" --supersedes decision-2026-01-19-a1b2c3
```

See what's currently active:

```bash
arx list --state active
```

## Journal Entry Format

Each entry is a markdown file with YAML frontmatter, stored in `.arx/journal/`:

```markdown
---
id: decision-2026-01-19-a1b2c3
type: decision
actor: human
date: 2026-01-19T14:30:00Z
title: Use PostgreSQL for primary storage
scope: backend
---

PostgreSQL chosen for its maturity, JSONB support, and team familiarity.
Considered MongoDB but rejected due to transaction requirements.
```

When a decision gets superseded:

```markdown
---
id: decision-2026-01-19-d4e5f6
type: decision
actor: human
date: 2026-01-19T16:45:00Z
title: Switch to CockroachDB for multi-region support
scope: backend
supersedes: decision-2026-01-19-a1b2c3
---

Requirements changed: we now need multi-region deployment.
CockroachDB provides PostgreSQL compatibility with distributed SQL.
```

The original entry remains unchanged—arx computes derived state by following the `supersedes` chain. Query `arx list --state active` and you'll see only CockroachDB; the PostgreSQL decision is automatically marked `superseded`.

## Entry Types

Different situations call for different entry types:

| Type | Use When |
|------|----------|
| `decision` | Making a choice between alternatives |
| `assumption` | Proceeding based on something unverified |
| `clarification` | Recording an answer to a question |
| `blocker` | Something is preventing progress |
| `risk` | Identifying a potential problem |
| `override` | Deliberately going against a previous decision |
| `defer` | Postponing something for later |
| `tombstone` | Marking something as deprecated or removed |

## Checkpoints

Long-running sessions need more than decisions—they need resumable state. Checkpoints capture where you are in a task:

```bash
# View current checkpoint
arx checkpoint show

# Clear when done
arx checkpoint clear
```

The checkpoint file (`.arx/checkpoint.json`) tracks task ID, status, and last activity time. Stale checkpoints (>72 hours) are flagged automatically.

```json
{
  "version": "1",
  "task_id": "implement-auth-system",
  "started_at": "2026-01-19T10:00:00Z",
  "last_activity": "2026-01-19T14:30:00Z",
  "status": "in_progress"
}
```

## Resume Context

Starting a new session? Generate context from your journal and checkpoint:

```bash
arx resume --print
```

This produces markdown summarizing active decisions, current checkpoint state, and recent entries—ready to paste into a new AI session.

## AI Assistant Integration

arx includes an MCP server so AI assistants can read and write journal entries directly:

```bash
go install github.com/demwunz/arx/cmd/mcp@latest
```

Add to Claude Code's MCP configuration:

```json
{
  "mcpServers": {
    "arx": {
      "command": "arx-mcp"
    }
  }
}
```

Available tools:
- `arx_add` — Create journal entries
- `arx_list` — List entries with filtering
- `arx_show` — Display full entry details
- `arx_checkpoint_show` — Check session state
- `arx_checkpoint_clear` — Clear checkpoint
- `arx_resume` — Generate resume context

## Directory Structure

```
.arx/
├── checkpoint.json           # Current session state
├── journal/                  # Entry files
│   ├── decision-2026-01-19-a1b2c3.md
│   ├── decision-2026-01-19-d4e5f6.md
│   └── assumption-2026-01-19-g7h8i9.md
└── resume-prompt.md          # Generated resume context
```

The `.arx/` directory belongs in your project root. Commit it to git—your decision history is part of your project's story.

## License

MIT
