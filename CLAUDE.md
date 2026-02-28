# Claude Instructions

**This project uses wobot governance. All tools are governed.**

---

## HARD RULES — violations are governance failures

1. **NO Read tool** on source files → use `wobot context read <file1> <file2>` via Bash
2. **NO Write/Edit tools** → use `wobot stage --apply` or `wobot mutate --spec`
3. **NO Grep/Glob** for file discovery → use `wobot context "task"` via Bash
4. **NO scope expansion** beyond the declared intent

### Allowed uses of Read tool
- `.wobot/` or `.claude/` paths (governance artifacts)
- Images (png, jpg, svg, etc.) and PDFs
- Nothing else

### Allowed uses of Grep/Glob
- Line-level search WITHIN files already identified by `wobot context`
- Searching inside `.wobot/` or `.claude/` paths
- Nothing else

---

## Prompt Classification

Classify every user message before responding:

| Class | Purpose | Rules |
|-------|---------|-------|
| **EXPLORATION** | Thinking, tradeoffs, alternatives | No code generation, no execution |
| **DECISION** | Converting ideas to plans | Structured output, declare risks, stop at planning |
| **EXECUTION** | Artifact generation | Follow schema exactly, no scope expansion |

1. Announce the class before responding
2. Do not mix classes in a single response
3. If ambiguous, default to EXPLORATION

---

## Workflow

### File Discovery
    wobot context "describe the task"                    # ALWAYS DO THIS FIRST
    wobot context query --preset deep --top 20 "task"    # deeper search

### File Reading
    wobot context read src/main.rs src/lib.rs              # batch read files
    wobot context read src/main.rs --symbol "fn main"      # narrow to symbol

### Mutations (preferred: single command)
    echo '<spec-json>' | wobot stage --apply             # validate + apply
    echo '<spec-json>' | wobot stage --dry-run           # validate only
    wobot draft --list                                   # list spec templates
    wobot draft cargo-dep | wobot stage --apply          # template → apply

### Staging content for specs
Write large content to `.wobot/scratch/` via Bash redirect, then reference via `content_file`.

### System Contract
Run `wobot describe --json` first, then obey it.

<!-- topo:start -->
## File Discovery — MANDATORY

**RULE: Use `wobot context "task"` via Bash as the FIRST step for ANY file discovery.**

Do NOT use Grep or Glob for finding files. Use them ONLY for line-level search within files already identified by `wobot context`.
<!-- topo:end -->
