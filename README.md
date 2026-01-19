# arx

Conventional Commits for decisions.

---

## The Problem

Context window exhaustion is invisible failure.

When you're building software with AI help, you make dozens of decisions: architecture choices, trade-offs, assumptions about requirements. These decisions live only in chat history—scattered, unsearchable, and eventually lost when the context window fills up.

Six months later, you can't answer: *Why did we choose REST over GraphQL? What assumptions were we making about load? Who decided to defer the caching layer?*

**arx** standardizes decision records. Like Conventional Commits for commit messages, but for the choices made during development.

---

## Why Not...

| Tool | What It Does | Gap |
|------|--------------|-----|
| **mem0** | Stores facts for retrieval | "Remember my preferences" ≠ "Track why we made this decision and whether it's still valid" |
| **RAG** | Retrieves documents to augment responses | Decisions aren't documents to retrieve—they're living records that get superseded, reversed, or deferred |
| **Chat history** | Captures everything | But surfaces nothing—you can't query it, filter it, or see which decisions are still active |
| **ADRs** | Architecture Decision Records | Heavy process, markdown templates, designed for major decisions—not the stream of micro-decisions in an AI session |

arx fills the gap: a structured journal that tracks the **lifecycle** of decisions.

---

## Design

| Principle | Implementation |
|-----------|----------------|
| **Zero dependencies** | Flatfiles in git. No database, no service, no account. |
| **Git-native** | `.arx/` directory commits with your code. Decision history is part of your project's story. |
| **Immutable journal** | Entries are never modified. Superseding or reversing creates a new entry with a backward link. |
| **Derived state** | Active/superseded/reversed computed at query time, not stored |
| **Model agnostic** | Works with Claude, GPT, Gemini, local models, or no AI at all |
| **Framework agnostic** | Works with any agentic framework or none |

---

## Setup

```bash
go install github.com/demwunz/arx/cmd/mcp@latest
```

Add to your AI client config (Claude Code, Cursor, Gemini, etc.):

```json
{
  "mcpServers": {
    "arx": { "command": "arx-mcp" }
  }
}
```

Add the [skill file](skill/arx.md) to your project for automatic capture.

---

## What Gets Captured

The AI captures decisions at natural inflection points:

| Moment | Entry Type |
|--------|------------|
| Choosing between alternatives | `decision` |
| Stating something unverified | `assumption` |
| Answering a clarifying question | `clarification` |
| Hitting a blocker | `blocker` |
| Acknowledging a risk | `risk` |
| Going against a recommendation | `override` |
| Postponing for later | `defer` |

You don't write YAML. The AI handles it.

---

## What It Looks Like

Decisions are markdown files in `.arx/journal/`:

```markdown
---
id: decision-2026-01-19-a1b2c3
type: decision
actor: human
date: 2026-01-19T14:30:00Z
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
title: Switch to CockroachDB for multi-region
supersedes: decision-2026-01-19-a1b2c3
---

Requirements changed. Need multi-region support.
```

The original stays untouched. State (`active`, `superseded`, `reversed`) is computed by following links.

---

## The Key Insight

**It turns "start over" into "continue from."**

With arx:
- Position is checkpointed
- Decisions are journaled
- Resume reconstructs context
- Continuity survives session boundaries

---

## More

- [Full spec](docs/spec.md) — entry format, relationships, checkpoint
- [CLI reference](docs/cli.md) — terminal usage
- [MCP tools](docs/mcp.md) — AI assistant integration
- [Skill file](skill/arx.md) — automatic capture triggers

---

## License

MIT
