<p align="center">
  <h1 align="center">arx</h1>
  <p align="center">Conventional Commits for decisions</p>
</p>

<p align="center">
  <a href="docs/spec.md">Spec</a> •
  <a href="docs/cli.md">CLI</a> •
  <a href="docs/mcp.md">MCP</a> •
  <a href="skill/arx.md">AI Skill</a>
</p>

**arx** gives your AI sessions a decision journal — structured, searchable, and version-aware. Every decision, assumption, and override is captured as an immutable markdown entry with full lifecycle tracking.

### How arx compares

|  | Decisions | Lifecycle | Searchable | |
|---|:-:|:-:|:-:|---|
| **arx** | ✅ | ✅ | ✅ | Structured journal with full decision lifecycle |
| **git log** | ❌ | ❌ | ⚠️ | Records _what_ changed — not _why_ or whether it still holds |
| **MEMORY.md** | ⚠️ | ❌ | ❌ | Flat notes with no lifecycle — stale sits next to current |
| **Project docs** | ⚠️ | ❌ | ❌ | Captures conclusions — not the reasoning or when overturned |
| **ADRs** | ✅ | ⚠️ | ❌ | Heavy process — not built for micro-decisions in AI sessions |
| **mem0** | ❌ | ❌ | ✅ | Stores facts for retrieval — not decision lifecycle |
| **RAG** | ❌ | ❌ | ✅ | Retrieves documents — doesn't track what's still valid |
| **Chat history** | ❌ | ❌ | ❌ | Captures everything — surfaces nothing |

### What you get

🔗 **Decision lifecycle** — track decisions from active through superseded or reversed
🔍 **Full-text search** — BM25F ranked search across titles, scopes, and content
📁 **Git-native** — plain markdown files that commit alongside your code
🤖 **AI-native** — MCP server for Claude Code, Cursor, and any MCP client
📦 **Compact archive** — inactive entries move to append-only archive, active stay editable
🔒 **Immutable by design** — entries are never modified, only linked forward

---

## Quick Start

**1. Install**
```bash
cargo install --path crates/arx-mcp
cargo install --path crates/arx-cli
```

**2. Configure** your AI client (Claude Code, Cursor, etc.)
```json
{
  "mcpServers": {
    "arx": { "command": "arx-mcp" }
  }
}
```

**3. Use** — the AI captures decisions automatically, or ask explicitly:
> "What decisions have we made?"
> "Record that we chose PostgreSQL for the database"
> "Search for decisions about the database"

---

## What It Looks Like

Decisions are markdown files in `.arx/journal/`:

```markdown
---
id: decision-2026-01-19-a1b2c3
type: decision
actor: human
title: Use PostgreSQL for primary storage
scope: backend
---

Chosen for maturity and team familiarity.
```

When things change, a new entry links back:

```markdown
---
id: decision-2026-01-19-d4e5f6
type: decision
title: Switch to CockroachDB
supersedes: decision-2026-01-19-a1b2c3
---

Requirements changed. Need multi-region.
```

State (`active`, `superseded`, `reversed`) is computed by following links. Original entries are never modified.

---

## Search & Compaction

Find any decision instantly—arx uses ranked full-text search across titles, scopes, and content:

```bash
arx search "PostgreSQL"
# [1.842] [active] decision-2026-01-19-d4e5f6 (decision) - Switch to CockroachDB
# [0.917] [superseded] decision-2026-01-19-a1b2c3 (decision) - Use PostgreSQL

arx search "database" --type decision --state active
```

Over time, superseded and reversed entries compact into an archive—keeping the journal lean while preserving full history:

```bash
arx compact --older-than 30
# Compacted: 12 entries moved to archive, 5 entries remaining in journal/
```

Nothing is lost. Active entries stay as editable markdown. Inactive entries move to `.arx/archive.jsonl`—append-only, git-diffable, and still searchable.

---

## Supersede & Reverse

Decisions change. arx tracks the lineage:

```bash
arx supersede decision-2026-01-19-a1b2c3 --type decision -m "Switch to CockroachDB"
arx reverse assumption-2026-01-19-g7h8i9 --reason "Load tests disproved this"
```

The original entry is never modified—its state updates to `superseded` or `reversed` by following the chain of links.

---

## What Gets Captured

| Moment | Entry Type |
|--------|------------|
| Choosing between alternatives | `decision` |
| Proceeding on something unverified | `assumption` |
| Answering a clarifying question | `clarification` |
| Something blocking progress | `blocker` |
| Acknowledging a risk | `risk` |
| Going against a recommendation | `override` |
| Postponing for later | `defer` |

The [AI skill file](skill/arx.md) teaches assistants when to capture automatically.

---

## Design

| Principle | Why |
|-----------|-----|
| **Flatfiles in git** | No database, no service. Commits with your code. |
| **Immutable entries** | Never edit—supersede or reverse instead. |
| **Backward links only** | New points to old. Preserves immutability. |
| **Derived state** | Active/superseded/reversed computed at query time. |
| **Compact archive** | Inactive entries move to `.arx/archive.jsonl`. Full history, zero clutter. |
| **Model agnostic** | Works with any AI or none. |

---

## The Key Insight

**It turns "start over" into "continue from."**

- Position is checkpointed
- Decisions are journaled
- Resume reconstructs context
- Continuity survives session boundaries

---

## More

| Resource | Description |
|----------|-------------|
| [Full Spec](docs/spec.md) | Entry format, relationships, checkpoint |
| [CLI Reference](docs/cli.md) | Terminal usage |
| [MCP Tools](docs/mcp.md) | AI assistant integration |
| [AI Skill](skill/arx.md) | Automatic capture triggers |

---

## License

MIT
