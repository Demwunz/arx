# arx

A lightweight journal and checkpoint system for tracking decisions, assumptions, and project state during AI-assisted development.

## Overview

arx provides:

- **Journal entries** - Track decisions, assumptions, clarifications, blockers, risks, and more
- **Checkpoint state** - Save and restore session state for resuming work
- **MCP integration** - Expose functionality as MCP tools for AI assistants
- **Derived state** - Automatically track which entries are active, superseded, or reversed

## Installation

```bash
go install github.com/demwunz/arx@latest
```

For the MCP server:

```bash
go install github.com/demwunz/arx/cmd/mcp@latest
```

## CLI Usage

### Adding entries

```bash
# Add a decision
arx add decision "Use REST API instead of GraphQL"

# Add an assumption with scope
arx add assumption "Database will handle 10k concurrent connections" --scope backend

# Supersede a previous decision
arx add decision "Switch to GraphQL for flexibility" --supersedes decision-2026-01-19-abc123
```

### Listing entries

```bash
# List all entries
arx list

# Filter by type
arx list --type decision

# Filter by state (active only)
arx list --state active
```

### Showing entries

```bash
arx show decision-2026-01-19-abc123
```

### Managing checkpoints

```bash
# Show current checkpoint
arx checkpoint show

# Clear checkpoint
arx checkpoint clear
```

### Generating resume context

```bash
# Write to file
arx resume

# Print to stdout
arx resume --print
```

## Entry Types

- `clarification` - Clarifying a requirement or question
- `decision` - A project decision
- `override` - Overriding a previous decision
- `blocker` - Something blocking progress
- `assumption` - An assumption being made
- `risk` - A risk being tracked
- `defer` - Something deferred for later
- `tombstone` - Marking something as deprecated/removed

## MCP Integration

arx includes an MCP server that exposes the following tools:

- `arx_add` - Create a new journal entry
- `arx_list` - List journal entries
- `arx_show` - Display a single entry
- `arx_checkpoint_show` - Display checkpoint status
- `arx_checkpoint_clear` - Clear the checkpoint
- `arx_resume` - Generate resume context

### Claude Code Configuration

Add to your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "arx": {
      "command": "arx-mcp"
    }
  }
}
```

## Data Storage

All data is stored in `.arx/` directory:

```
.arx/
├── checkpoint.json      # Current session checkpoint
├── journal/             # Journal entry markdown files
│   ├── decision-2026-01-19-abc123.md
│   └── assumption-2026-01-19-def456.md
└── resume-prompt.md     # Generated resume context
```

## License

MIT
