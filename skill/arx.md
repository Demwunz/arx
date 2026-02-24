# arx Decision Capture

Automatically capture decisions at natural inflection points during development.

## When to Capture

Capture **proactively**—don't wait to be asked.

| Trigger | Entry Type | Example |
|---------|------------|---------|
| Human chooses between alternatives | `decision` | "Let's use PostgreSQL" → record it |
| Proceeding on something unverified | `assumption` | "Assuming the API can handle 1000 req/s" → record it |
| Human clarifies a requirement | `clarification` | "The deadline is flexible" → record it |
| Something blocks progress | `blocker` | "Can't proceed without API credentials" → record it |
| Acknowledging a risk and continuing | `risk` | "This might not scale, but OK for MVP" → record it |
| Going against your recommendation | `override` | You suggested X, human chose Y → record it |
| Explicitly postponing something | `defer` | "We'll handle caching later" → record it |

## How to Capture

Use the `arx_add` tool:

```
arx_add(type="decision", message="Use PostgreSQL for primary storage", scope="backend")
```

Keep messages concise. The title should make sense in a list.

## When to Check Context

At **session start**, check for existing context:

1. Call `arx_checkpoint_show` — is there work in progress?
2. Call `arx_list` with `state=active` — what decisions are current?
3. If resuming, call `arx_resume` to generate full context

## When to Search or Compact

- Use `arx_search` to find relevant decisions before making new ones — avoids duplicates and surfaces context
- Use `arx_compact` when the journal grows large — moves inactive entries to archive while keeping active ones editable

## When to Supersede or Reverse

**Supersede** when requirements changed or a better approach is found:
```
arx_add(type="decision", message="Switch to CockroachDB for multi-region", supersedes="decision-2026-01-19-a1b2c3")
```

**Reverse** when an approach failed or a mistake was discovered:
```
arx_add(type="decision", message="Revert to polling—WebSockets too complex", reverses="decision-2026-01-19-d4e5f6")
```

## Capture Style

**Do:**
- Capture immediately when the moment happens
- Keep titles short and scannable
- Include scope when relevant (backend, api, auth, etc.)
- Add body text for important context

**Don't:**
- Wait for human to ask you to record
- Capture trivial things (minor code style choices)
- Capture the same decision twice
- Capture implementation details (that's what git is for)

## Execution Failure Capture

When automated steps or commands fail during execution, capture immediately.

### Step/Command Failure → `blocker`

**Trigger:** A pipeline step, script, or automated command fails

**Capture:**
- **Type:** `blocker`
- **Title:** Brief description of what failed
- **Body:** Root cause, what was attempted, resolution if known
- **Scope:** Affected file, component, or system

```
arx_add(type="blocker", message="Build failed - missing jwt import in auth.go", scope="backend/auth", body="Generated code referenced jwt.ParseToken but import was missing. Resolution: Added import manually.")
```

### Manual Intervention Required → `override`

**Trigger:** Human must fix something the AI/automation got wrong

**Capture:**
- **Type:** `override`
- **Title:** What was overridden
- **Body:** What automation attempted, why it was wrong, what human did instead

```
arx_add(type="override", message="Manual fix for incorrect file path", scope="execute", body="Plan targeted src/auth.go but file is at pkg/auth/auth.go. Human corrected path.")
```

### Edge Case Discovered → `clarification`

**Trigger:** Execution reveals an undocumented constraint or behavior

**Capture:**
- **Type:** `clarification`
- **Title:** The discovery
- **Body:** What was discovered, how, and impact

```
arx_add(type="clarification", message="API rate limit is 100/min not 1000/min", scope="api", body="Discovered during load test. Documentation was wrong. Affects batch processing design.")
```

### What NOT to Capture (Execution)

- Routine successful completions (no learning value)
- Transient errors that self-resolve (retries, network blips)
- Errors already captured in the current session

## Examples

### Decision captured automatically

```
Human: "Let's go with REST instead of GraphQL—simpler for the team"

AI thinking: Human chose between alternatives → capture decision

AI action: arx_add(type="decision", message="Use REST API instead of GraphQL", scope="api")

AI response: "Got it—REST for simplicity. I've recorded that decision."
```

### Assumption captured automatically

```
Human: "The database should handle our load fine"

AI thinking: Unverified statement being used as basis for proceeding → capture assumption

AI action: arx_add(type="assumption", message="Database will handle expected load", scope="backend")

AI response: "I've noted that as an assumption. We should validate it under load."
```

### Risk acknowledged

```
Human: "I know it's not ideal, but let's skip tests for now to hit the deadline"

AI thinking: Human acknowledging risk and proceeding anyway → capture risk

AI action: arx_add(type="risk", message="Skipping tests to meet deadline—technical debt", scope="testing")

AI response: "Understood. I've logged that as a known risk so we remember to address it."
```

### Session start

```
AI thinking: New session → check for existing context

AI action: arx_checkpoint_show()
AI action: arx_list(state="active")

AI response: "I see we were working on the auth system. Here are the active decisions:
- Use JWT for authentication
- Store tokens in httpOnly cookies
Want me to generate full resume context?"
```

## Installation

Copy this file to your project:
- Claude Code: `.claude/skills/arx.md`
- Or include in your `CLAUDE.md` / `AGENTS.md`