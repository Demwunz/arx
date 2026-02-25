<p align="center">
  <h1 align="center">arx</h1>
  <p align="center">Conventional Commits for decisions</p>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> •
  <a href="docs/spec.md">Spec</a> •
  <a href="docs/cli.md">CLI</a> •
  <a href="docs/mcp.md">MCP</a> •
  <a href="skill/arx.md">AI Skill</a>
</p>

**arx** gives your AI sessions a decision journal — structured, searchable, and version-aware. Every decision, assumption, and override is captured as an immutable markdown entry with full lifecycle tracking.

### Features

- 🔗 **Decision lifecycle** — track decisions from active through superseded or reversed
- 🔍 **Full-text search** — BM25F ranked search across titles, scopes, and content
- 📁 **Git-native** — plain markdown files that commit alongside your code
- 🤖 **AI-native** — MCP server for Claude Code, Cursor, and any MCP client
- 📦 **Compact archive** — inactive entries move to append-only archive, active stay editable
- 🔒 **Immutable by design** — entries are never modified, only linked forward

### How arx compares

|  | Decisions | Lifecycle | Searchable | |
|---|:-:|:-:|:-:|---|
| **arx** | ✓ | ✓ | ✓ | Structured journal with full decision lifecycle |
| **git log** | ✗ | ✗ | ~ | Records _what_ changed — not _why_ or whether it still holds |
| **MEMORY.md** | ~ | ✗ | ✗ | Flat notes with no lifecycle — stale sits next to current |
| **Project docs** | ~ | ✗ | ✗ | Captures conclusions — not the reasoning or when overturned |
| **ADRs** | ✓ | ~ | ✗ | Heavy process — not built for micro-decisions in AI sessions |
| **mem0** | ✗ | ✗ | ✓ | Stores facts for retrieval — not decision lifecycle |
| **RAG** | ✗ | ✗ | ✓ | Retrieves documents — doesn't track what's still valid |
| **Chat history** | ✗ | ✗ | ✗ | Captures everything — surfaces nothing |

---

## See It in Action

```bash
# Chose between alternatives? Record it.
arx add decision "Use PostgreSQL for primary storage"

# Proceeding on something unverified? Note the assumption.
arx add assumption "API handles 1000 req/s" --scope backend

# Acknowledging a risk? Log it before you forget.
arx add risk "Skipping tests to hit deadline" --scope testing
```

Every entry is an immutable markdown file in `.arx/journal/`. See what's current:

```bash
arx list --state active
# [active] decision-2026-01-19-a1b2c3 (decision) - Use PostgreSQL for primary storage
# [active] assumption-2026-01-19-b4c5d6 (assumption) - API handles 1000 req/s
# [active] risk-2026-01-19-e7f8a9 (risk) - Skipping tests to hit deadline
```

When decisions change, arx tracks the [full lifecycle](#supersede--reverse) — supersede, reverse, [search](#search--compaction), and compact.

---

## Quick Start

**1. Install**
```bash
cargo install --path crates/arx-mcp
cargo install --path crates/arx-cli
```

**Pi (AI coding agent):**
```bash
pi install git:github.com/Demwunz/pi-wobot
```
Registers the `journal` tool. See [pi-wobot](https://github.com/Demwunz/pi-wobot) for details.

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

State (`active`, `superseded`, `reversed`) is computed by following the link chain — new entries point to old, never the reverse.

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

## When to Record

Decisions aren't the only things worth capturing. Any moment where the project's direction shifts — or *could* shift later — is worth a line in the journal:

| Moment | Entry Type |
|--------|------------|
| Choosing between alternatives | `decision` |
| Proceeding on something unverified | `assumption` |
| Answering a clarifying question | `clarification` |
| Something blocking progress | `blocker` |
| Acknowledging a risk | `risk` |
| Going against a recommendation | `override` |
| Postponing for later | `defer` |

With the [AI skill file](skill/arx.md), assistants capture these automatically — you don't have to remember to ask.

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

MIT with [Commons Clause](LICENSE). Free to use, modify, and redistribute — but not to sell as part of a paid product.
