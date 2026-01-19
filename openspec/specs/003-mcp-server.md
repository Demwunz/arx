# Arx MCP Server

**Status:** Draft
**Created:** 2026-01-19
**Depends:** 001-core-packages, 002-cli-commands

## Purpose

Expose arx journal and checkpoint functionality as MCP tools for AI assistant integration.

## Requirements

### Server Structure

1. The MCP server SHALL be implemented in `cmd/mcp/main.go`
2. The server SHALL use stdio transport for MCP communication
3. The server SHALL implement the MCP tools protocol

### Tools

4. The server SHALL implement `arx_add` tool to create journal entries
5. The `arx_add` tool SHALL accept type, message, and optional scope parameters
6. The `arx_add` tool SHALL validate entry types before creation
7. The `arx_add` tool SHALL return the created entry ID on success

8. The server SHALL implement `arx_list` tool to list journal entries
9. The `arx_list` tool SHALL support optional type and state filters
10. The `arx_list` tool SHALL return entries sorted by date (newest first)

11. The server SHALL implement `arx_show` tool to display a single entry
12. The `arx_show` tool SHALL accept an entry ID parameter
13. The `arx_show` tool SHALL return full entry details including state

14. The server SHALL implement `arx_checkpoint_show` tool
15. The `arx_checkpoint_show` tool SHALL return current checkpoint status
16. The `arx_checkpoint_show` tool SHALL indicate if checkpoint is stale

17. The server SHALL implement `arx_checkpoint_clear` tool
18. The `arx_checkpoint_clear` tool SHALL remove the checkpoint file

19. The server SHALL implement `arx_resume` tool
20. The `arx_resume` tool SHALL return resume context as markdown

### Error Handling

21. All tools SHALL return descriptive error messages on failure
22. The server SHALL handle missing journal gracefully (return empty list)
23. The server SHALL handle missing checkpoint gracefully (return "no checkpoint")
