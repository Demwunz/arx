# arx

Conventional Commits for decisions.

---

## The Problem

AI assistants forget. Decisions made yesterday are lost today.

**arx** standardizes decision records—like Conventional Commits for commit messages, but for the choices you make during development.

---

## Setup

```bash
go install github.com/demwunz/arx/cmd/mcp@latest
```

Add to your AI client config:

```json
{
  "mcpServers": {
    "arx": { "command": "arx-mcp" }
  }
}
```

Works with Claude Code, Cursor, Gemini, and other MCP clients.

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

Under the hood, decisions are markdown files in `.arx/journal/`:

```markdown
---
id: decision-2026-01-19-a1b2c3
type: decision
title: Use PostgreSQL for primary storage
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

State (`active`, `superseded`, `reversed`) is computed by following links.

---

## Why Not...

| Tool | Gap |
|------|-----|
| mem0 | Stores facts, not decision lifecycle |
| RAG | Retrieves docs, doesn't track what's current |
| Chat history | Captures everything, surfaces nothing |
| ADRs | Heavy process, major decisions only |

---

## More

- [Full spec](docs/spec.md)
- [CLI reference](docs/cli.md)
- [MCP tools](docs/mcp.md)
- [AI skill file](skill/arx.md) — add to your project for automatic capture

---

## License

MIT
