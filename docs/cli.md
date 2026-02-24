# CLI Reference

The arx CLI is a thin wrapper around the spec. You can always create files manually—the CLI just makes it easier.

## Installation

```bash
cargo install --path crates/arx-cli
```

## Commands

### add

Create a journal entry.

```bash
arx add <type> "<message>" [flags]
```

**Arguments:**
- `type` — Entry type (decision, assumption, clarification, blocker, risk, override, defer, tombstone)
- `message` — Short description (becomes the title)

**Flags:**
- `--scope` — Optional scope (e.g., backend, api, auth)
- `--supersedes` — ID of entry this replaces
- `--reverses` — ID of entry this undoes

**Examples:**

```bash
# Simple decision
arx add decision "Use PostgreSQL for storage"

# With scope
arx add assumption "API handles 1000 req/s" --scope backend

# Superseding a previous decision
arx add decision "Switch to CockroachDB" --supersedes decision-2026-01-19-a1b2c3

# Reversing a failed approach
arx add decision "Revert to polling" --reverses decision-2026-01-19-d4e5f6
```

### list

List journal entries.

```bash
arx list [flags]
```

**Flags:**
- `--type` — Filter by entry type
- `--state` — Filter by state (active, superseded, reversed)

**Examples:**

```bash
# All entries
arx list

# Only decisions
arx list --type decision

# Only active entries
arx list --state active
```

**Output:**

```
[active] decision-2026-01-19-d4e5f6 (decision) - Switch to CockroachDB
[superseded] decision-2026-01-19-a1b2c3 (decision) - Use PostgreSQL
[active] assumption-2026-01-19-g7h8i9 (assumption) - API handles 1000 req/s
```

### show

Display a single entry with full details.

```bash
arx show <id>
```

**Example:**

```bash
arx show decision-2026-01-19-a1b2c3
```

**Output:**

```
ID:          decision-2026-01-19-a1b2c3
Type:        decision
Actor:       human
Date:        2026-01-19T14:30:00Z
Title:       Use PostgreSQL for storage
State:       superseded
Scope:       backend

Content:
Chosen for maturity and team familiarity.
```

### checkpoint

Manage session state.

```bash
arx checkpoint [subcommand]
```

**Subcommands:**
- `show` — Display current checkpoint (default)
- `clear` — Remove checkpoint file

**Examples:**

```bash
# View checkpoint
arx checkpoint
arx checkpoint show

# Clear checkpoint
arx checkpoint clear
```

### resume

Generate context for resuming work in a new session.

```bash
arx resume [flags]
```

**Flags:**
- `--print` — Print to stdout instead of writing to file

**Examples:**

```bash
# Write to .arx/resume-prompt.md
arx resume

# Print to stdout (for pasting into new session)
arx resume --print
```

**Output includes:**
- Current checkpoint status
- Active journal entries
- Ready-to-use context for AI sessions

### search

Search journal entries by query text.

```bash
arx search <QUERY> [flags]
```

**Flags:**
- `--type` — Filter by entry type
- `--state` — Filter by state (active, superseded, reversed)
- `--scope` — Filter by scope
- `--limit` — Maximum results (0 = no limit, default)

**Examples:**

```bash
arx search "PostgreSQL"
# [1.842] [active] decision-2026-01-19-d4e5f6 (decision) - Switch to CockroachDB
# [0.917] [superseded] decision-2026-01-19-a1b2c3 (decision) - Use PostgreSQL

arx search "database" --type decision --state active
```

### compact

Compact old/inactive journal entries into archive.

```bash
arx compact [flags]
```

**Flags:**
- `--older-than` — Move entries older than this many days (default: 30)

**Examples:**

```bash
arx compact --older-than 30
# Compacted: 12 entries moved to archive, 5 entries remaining in journal/
```

### supersede

Create a new entry that supersedes an existing one.

```bash
arx supersede <OLD_ID> --type <TYPE> -m "<MESSAGE>" [flags]
```

**Flags:**
- `--type` — Type of the new entry (required)
- `-m` / `--message` — Message for the new entry (required)
- `--scope` — Scope for the new entry

**Examples:**

```bash
arx supersede decision-2026-01-19-a1b2c3 --type decision -m "Switch to CockroachDB"
# Created entry: decision-2026-01-19-d4e5f6 (supersedes decision-2026-01-19-a1b2c3)
```

### reverse

Create a tombstone entry that reverses an existing one.

```bash
arx reverse <TARGET_ID> --reason "<REASON>"
```

**Flags:**
- `--reason` — Reason for reversal (required)

**Examples:**

```bash
arx reverse assumption-2026-01-19-g7h8i9 --reason "Load tests disproved this"
# Created tombstone: tombstone-2026-01-19-j1k2l3 (reverses assumption-2026-01-19-g7h8i9)
```

## Exit Codes

- `0` — Success
- `1` — Error (invalid arguments, file not found, etc.)
