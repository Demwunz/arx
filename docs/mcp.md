# MCP Server

The arx MCP server exposes journal and checkpoint functionality as tools for AI assistants. It's a thin wrapper—the files remain the source of truth.

## Installation

```bash
go install github.com/demwunz/arx/cmd/mcp@latest
```

This installs `arx-mcp`.

## Configuration

### Claude Code

Add to your MCP settings (`~/.claude/settings.json` or project `.claude/settings.local.json`):

```json
{
  "mcpServers": {
    "arx": {
      "command": "arx-mcp"
    }
  }
}
```

### Other MCP Clients

The server uses stdio transport. Run `arx-mcp` and communicate via stdin/stdout using the MCP protocol.

## Tools

### arx_add

Create a journal entry.

**Parameters:**
- `type` (required) — Entry type
- `message` (required) — Entry title/description
- `scope` (optional) — Entry scope

**Returns:** JSON with created entry ID

```json
{"id": "decision-2026-01-19-a1b2c3"}
```

### arx_list

List journal entries.

**Parameters:**
- `type` (optional) — Filter by entry type
- `state` (optional) — Filter by state (active, superseded, reversed)

**Returns:** JSON array of entries

```json
{
  "entries": [
    {
      "id": "decision-2026-01-19-d4e5f6",
      "type": "decision",
      "state": "active",
      "title": "Switch to CockroachDB",
      "date": "2026-01-19T16:45:00Z"
    }
  ]
}
```

### arx_show

Display a single entry.

**Parameters:**
- `id` (required) — Entry ID

**Returns:** Full entry details as JSON

```json
{
  "id": "decision-2026-01-19-a1b2c3",
  "type": "decision",
  "actor": "human",
  "date": "2026-01-19T14:30:00Z",
  "title": "Use PostgreSQL for storage",
  "state": "superseded",
  "scope": "backend",
  "content": "Chosen for maturity and team familiarity."
}
```

### arx_checkpoint_show

Display current checkpoint status.

**Parameters:** None

**Returns:** Checkpoint details or exists: false

```json
{
  "exists": true,
  "version": "1",
  "task_id": "implement-auth",
  "status": "in_progress",
  "started_at": "2026-01-19T10:00:00Z",
  "last_activity": "2026-01-19T14:30:00Z",
  "is_stale": false
}
```

### arx_checkpoint_clear

Remove the checkpoint file.

**Parameters:** None

**Returns:** Success confirmation

```json
{
  "success": true,
  "message": "Checkpoint cleared successfully"
}
```

### arx_resume

Generate resume context as markdown.

**Parameters:** None

**Returns:** Markdown content for session resumption

```json
{
  "markdown": "# Resume Context\n\nGenerated: 2026-01-19T15:00:00Z\n\n## Checkpoint\n..."
}
```

## Error Handling

All tools return errors as tool result errors with descriptive messages:

- Invalid entry type
- Missing required parameters
- Entry not found
- File system errors

## Notes

- Entries created via MCP have `actor: system` (not `human`)
- The MCP server operates in the current working directory
- All changes are written to `.arx/` immediately
