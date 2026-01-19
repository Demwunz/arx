# arx

Conventional Commits for decisions.

## The Problem

Context window exhaustion is invisible failure.

When you're building software with AI help, you make dozens of decisions: architecture choices, trade-offs, assumptions about requirements. These decisions live only in chat history—scattered, unsearchable, and eventually lost when the context window fills up.

Current state:
- AI forgets decisions
- AI forgets rationale
- AI contradicts earlier work
- Human re-explains everything

Six months later, you can't answer: *Why did we choose REST over GraphQL? What assumptions were we making about load? Who decided to defer the caching layer?*

## Why Not...

| Tool | What It Does | Gap |
|------|--------------|-----|
| **mem0** | Stores facts for retrieval | "Remember my preferences" ≠ "Track why we made this decision and whether it's still valid" |
| **RAG** | Retrieves documents to augment responses | Decisions aren't documents to retrieve—they're living records that get superseded, reversed, or deferred |
| **Chat history** | Captures everything | But surfaces nothing—you can't query it, filter it, or see which decisions are still active |
| **ADRs** | Architecture Decision Records | Heavy process, markdown templates, designed for major decisions—not the stream of micro-decisions in an AI session |

arx fills the gap: a structured journal that tracks the **lifecycle** of decisions. When they were made, by whom (human or AI), and whether they're still active, have been superseded, or were reversed.

## Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Zero dependencies** | Flatfiles in git. No database, no service, no account. |
| **Git-native** | `.arx/` directory commits with your code. Decision history is part of your project's story. |
| **Model agnostic** | Works with Claude, GPT, Gemini, local models, or no AI at all |
| **Framework agnostic** | Works with any agentic framework or none |
| **Immutable journal** | Entries are never modified. Superseding or reversing creates a new entry with a backward link. |
| **Derived state** | Active/superseded/reversed computed at query time, not stored |

## The Spec

The spec is the core value. Everything else is tooling.

### Journal Entry Format

Each entry is a markdown file with YAML frontmatter:

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

### Entry Types

| Type | Use When |
|------|----------|
| `clarification` | Recording an answer to a question |
| `decision` | Making a choice between alternatives |
| `override` | Deliberately going against a previous decision |
| `blocker` | Something is preventing progress |
| `assumption` | Proceeding based on something unverified |
| `risk` | Identifying a potential problem |
| `defer` | Postponing something for later |
| `tombstone` | Marking something as deprecated or removed |

### Relationship Links

Inspired by the saga pattern's compensating actions:

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

| Link | When to Use | Effect |
|------|-------------|--------|
| `supersedes` | Requirements changed, better approach found | Original becomes `superseded` (inactive but valid history) |
| `reverses` | Approach failed, mistake discovered, rollback needed | Original becomes `reversed` (inactive, marked unsuccessful) |
| `relates_to` | Context without replacement | No state change |

**Critical: Only backward links.** New entries point to old entries, never the reverse. This preserves immutability—no back-patching required.

### Derived State

State is computed at query time by following the link graph:

| State | Rule |
|-------|------|
| `active` | No other entry supersedes or reverses this one |
| `superseded` | Another entry has `supersedes: <this-id>` |
| `reversed` | Another entry has `reverses: <this-id>` |

### Checkpoint Format

For session continuity—where you are in a task:

```json
{
  "version": "1",
  "task_id": "implement-auth-system",
  "started_at": "2026-01-19T10:00:00Z",
  "last_activity": "2026-01-19T14:30:00Z",
  "status": "in_progress"
}
```

Stale checkpoints (>72 hours) are flagged automatically.

## Directory Structure

```
.arx/
├── checkpoint.json           # Current session state (overwrite)
├── journal/                  # Entry files (append-only)
│   ├── decision-2026-01-19-a1b2c3.md
│   ├── decision-2026-01-19-d4e5f6.md
│   └── assumption-2026-01-19-g7h8i9.md
└── resume-prompt.md          # Generated context for new sessions
```

The `.arx/` directory belongs in your project root. Commit it to git.

## Adoption

| Level | Setup | What You Get |
|-------|-------|--------------|
| 1. Spec only | Create `.arx/journal/` | Manual file creation, git tracks history |
| 2. CLI | `go install github.com/demwunz/arx@latest` | Validation, proper formatting, queries |
| 3. MCP | `go install github.com/demwunz/arx/cmd/mcp@latest` | AI assistants read/write directly |

Start simple. Add rigor as needed.

## CLI Usage

```bash
# Record a decision
arx add decision "Use PostgreSQL for primary storage"

# Track an assumption with scope
arx add assumption "API will receive <1000 requests/second" --scope backend

# Supersede when requirements change
arx add decision "Switch to CockroachDB for multi-region" --supersedes decision-2026-01-19-a1b2c3

# See what's currently active
arx list --state active

# Show full entry details
arx show decision-2026-01-19-a1b2c3

# Generate resume context for a new session
arx resume --print
```

## MCP Integration

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

Tools: `arx_add`, `arx_list`, `arx_show`, `arx_checkpoint_show`, `arx_checkpoint_clear`, `arx_resume`

## Comparison

| Factor | arx |
|--------|-----|
| Solves real pain | ✓ Context loss is universal |
| Zero dependencies | ✓ Markdown files in git |
| Works with existing tools | ✓ MCP, CLI, any editor |
| Easy to understand | ✓ "Conventional Commits for decisions" |
| Easy to adopt incrementally | ✓ Add one file, you're using it |
| Model agnostic | ✓ Works with any AI or none |
| Framework agnostic | ✓ Works with any agentic framework |

## The Key Insight

**It turns "start over" into "continue from."**

With arx:
- Position is checkpointed
- Decisions are journaled
- Resume reconstructs context
- Continuity survives session boundaries

## License

MIT
