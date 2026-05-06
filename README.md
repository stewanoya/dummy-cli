# dummy

A single CLI that routes bulk I/O tasks to cheap LLM providers so Claude can focus on reasoning.

**Core principle:** Claude reasons. `dummy` reads and writes.

---

## Installation

```sh
cargo install --path .
```

Requires Rust 1.75+. Place the resulting `dummy` binary somewhere on your `PATH`.

---

## Quick Start

Set environment variables and run any command:

```sh
set DUMMY_API_KEY=sk-...
set DUMMY_BASE_URL=https://api.deepseek.com/v1
set DUMMY_MODEL=deepseek-chat
```

Or create a config file at `%APPDATA%\dummy\config.toml` (Windows) / `~/.config/dummy/config.toml` (macOS/Linux):

```toml
default_provider = "deepseek"

[providers.deepseek]
api_key  = "sk-..."
base_url = "https://api.deepseek.com/v1"
model    = "deepseek-chat"

[providers.kimi]
api_key  = "..."
base_url = "https://api.moonshot.ai/v1"
model    = "kimi-k2.5"

[providers.ollama]
api_key  = "ollama"
base_url = "http://localhost:11434/v1"
model    = "llama3"

[providers.claude-haiku]
api_type = "anthropic"
api_key  = "sk-ant-..."
model    = "claude-haiku-4-5-20251001"
```

Verify your setup:

```sh
dummy config
```

---

## Commands

All commands have detailed `--help` output designed to be read by an LLM.

| Command | What it does | LLM call |
|---|---|---|
| `dummy read` | Read files and answer a question | Yes |
| `dummy write` | Generate code or docs to a file | Yes |
| `dummy chat` | Extract text from a Claude JSONL transcript | No |
| `dummy config` | Show the resolved provider config | No |

Use `--provider <name>` on any command to override the default provider for that call.

---

## Provider API Types

Two wire formats are supported. Set `api_type` in the provider block — it defaults to `openai`.

| `api_type` | Works with |
|---|---|
| `openai` | DeepSeek, Kimi (Moonshot), Ollama, Groq, Together AI, OpenAI, and any OpenAI-compatible endpoint |
| `anthropic` | Anthropic Claude models (direct API) |

---

## CLAUDE.md Integration

Add the following block to any project's `CLAUDE.md`. The rules use mandatory language ("STOP", "NEVER", "MUST") so Claude treats them as hard constraints rather than optional suggestions.

Adjust the examples to match your project's file paths and patterns.

---

```markdown
## Cheap-Model Delegation (Token Budget)

**Rule:** Claude reasons and edits. `dummy` reads and generates.

The `dummy` CLI is always available. Use it to delegate bulk I/O to a cheap model so Claude's tokens are spent on reasoning, not reading files.

---

### MANDATORY: Before reading any file, apply this check

**STOP and use `dummy read` if ANY of these are true:**
- The task touches 3 or more files
- Any single file is likely over 400 lines (source files, schemas, lockfiles, generated code)
- The task requires surveying a module, package, or codebase before acting
- The task is "do X across the whole repo" or "find all Y" — anything repo-wide

**NEVER use the Read tool, Glob tool, or Explore subagent to build context for a task. These are all bulk I/O — dummy handles them. Use `dummy read` first, then only open the specific file you need to edit.**

```sh
dummy read --paths <file1> [file2 ...] --question "<specific question>"
```

**Examples:**

```sh
# Understand a large module before editing it
dummy read --paths src/auth/middleware.rs --question "What does this middleware do and what does it expect on the request?"

# Map relationships across files before refactoring
dummy read --paths src/models/user.rs src/models/order.rs src/db/schema.sql \
           --question "List all foreign key relationships and which structs map to which tables"

# Check a config file without reading it directly
dummy read --paths Cargo.toml --question "What version of reqwest is being used and what features are enabled?"

# Survey a whole package before touching it
dummy read --paths src/payments/mod.rs src/payments/stripe.rs src/payments/webhooks.rs \
           --question "Summarize the payment flow end to end"
```

The only time to use the Read tool directly is when you already know which specific lines you need to edit.

---

### MANDATORY: Before generating boilerplate, apply this check

**STOP and use `dummy write` if ANY of these are true:**
- The output is a new test file
- The output is a config file (CI, Docker, linting, env template)
- The output mirrors the structure of an existing file
- The output is repetitive by nature (CRUD, migrations, API specs)

```sh
dummy write --spec "<what to generate>" --context <reference-file> --target <output-path>
```

**Examples:**

```sh
# Generate unit tests matching the style of existing tests
dummy write --spec "unit tests for all public functions in src/payments/stripe.rs" \
            --context tests/auth_test.rs \
            --target tests/stripe_test.rs

# Generate a CI workflow matching the project's existing one
dummy write --spec "GitHub Actions workflow: run cargo test on push to main" \
            --context .github/workflows/lint.yml \
            --target .github/workflows/test.yml

# Scaffold a new module matching the project's patterns
dummy write --spec "REST handler module for /api/v1/products — list, get, create, delete" \
            --context src/handlers/users.rs \
            --target src/handlers/products.rs
```

Review the output and make surgical edits. Do not regenerate — just fix what is wrong.

---

### Documentation updates after a session

```sh
dummy chat --input <session.jsonl> --output /tmp/chat.txt
dummy read --paths /tmp/chat.txt docs/CHANGELOG.md docs/ARCHITECTURE.md \
           --question "Based on this session, what exact changes should I make to these docs?"
```

Session logs:
- Windows: `%APPDATA%\Claude\projects\<project-hash>\<session-id>.jsonl`
- macOS/Linux: `~/.claude/projects/<project-hash>/<session-id>.jsonl`

---

### Exceptions — do NOT use `dummy` for:

- **Tasks under ~2000 tokens** — small single-file edits, quick lookups
- **Architectural decisions** — which approach, which abstraction, which tradeoff
- **Debugging** — root cause analysis, logic errors, subtle state bugs
- **Security-sensitive code** — auth, crypto, input validation, permissions
- **Anything requiring careful reasoning** — if it needs thought, it stays with Claude
```

---

## Environment Variables

All commands respect these overrides, which take priority over the config file:

| Variable | Purpose |
|---|---|
| `DUMMY_API_KEY` | API key for the active provider |
| `DUMMY_BASE_URL` | Base URL override (e.g. a local proxy) |
| `DUMMY_MODEL` | Model name override |
| `DUMMY_PROVIDER` | Select a named provider from the config file |

---

## Notes on Thinking Models

DeepSeek R1, Kimi K2.5, and other thinking models consume reasoning tokens internally before producing output. These count against `--max-tokens`. If a response is cut off:

- `dummy read`: try `--max-tokens 16384`
- `dummy write`: try `--max-tokens 32768`
