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

Add the following block to any project's `CLAUDE.md`. It instructs Claude Code when to delegate to `dummy` instead of consuming reasoning tokens on bulk I/O work.

Adjust the file-size thresholds and examples to match your project's patterns.

---

```markdown
## Cheap-Model Delegation (Token Budget)

Use the `dummy` CLI to delegate bulk I/O tasks. Reserve Claude for reasoning.

**Rule:** Claude = reasoning and edits. dummy = reading and generating.

---

### dummy read — bulk file reading

Call `dummy read` instead of reading files yourself when:

- Any single file is longer than 400 lines
- You would otherwise open 3 or more files for context
- You need to summarize a module, package, or schema
- You need to find relationships across files (imports, references, types)

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

# Check a config file you don't want to read manually
dummy read --paths Cargo.toml --question "What version of reqwest is being used and what features are enabled?"

# Survey a whole package before touching it
dummy read --paths src/payments/mod.rs src/payments/stripe.rs src/payments/webhooks.rs \
           --question "Summarize the payment flow end to end"
```

Returns structured bullet points to stdout. Read that output instead of the files.

Only read files directly when you need exact line numbers for an edit.

---

### dummy write — boilerplate generation

Call `dummy write` instead of generating repetitive code yourself when:

- Writing test files (unit tests, integration tests, fixtures)
- Creating config files (CI workflows, Dockerfiles, linting configs, env templates)
- Writing docstrings or comments across multiple functions
- Generating CRUD operations, API clients, or migration files
- Any output that is structurally predictable given an example

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

# Generate a Dockerfile from an existing similar one
dummy write --spec "Dockerfile for a Rust web service, expose port 8080" \
            --context services/api/Dockerfile \
            --target services/worker/Dockerfile

# Generate a database migration based on the schema
dummy write --spec "SQL migration to add an index on orders.user_id and orders.created_at" \
            --context migrations/001_init.sql \
            --target migrations/004_order_indexes.sql

# Scaffold a new module matching the project's patterns
dummy write --spec "REST handler module for /api/v1/products — list, get, create, delete" \
            --context src/handlers/users.rs \
            --target src/handlers/products.rs
```

Then review the output and make surgical edits. Do not regenerate — just fix what is wrong.

---

### dummy chat — documentation workflow

After a long session, extract the conversation and delegate documentation updates:

```sh
# Step 1: extract readable text from the session log
dummy chat --input <session.jsonl> --output /tmp/chat.txt

# Step 2: ask the cheap model what docs need updating
dummy read --paths /tmp/chat.txt docs/CHANGELOG.md docs/ARCHITECTURE.md \
           --question "Based on this session, what exact changes should I make to these docs?"

# Step 3: apply the suggested edits with the Edit tool
```

Find session JSONL files at:
- Windows: `%APPDATA%\Claude\projects\<project-hash>\<session-id>.jsonl`
- macOS/Linux: `~/.claude/projects/<project-hash>/<session-id>.jsonl`

---

### When NOT to delegate

Do not call `dummy` for:

- **Tasks under ~2000 tokens** — delegation overhead is not worth it for small reads
- **Architectural decisions** — which approach, which abstraction, which tradeoff
- **Debugging** — root cause analysis, logic errors, subtle state bugs
- **Security-sensitive code** — auth, crypto, input validation, permissions
- **Anything requiring careful reasoning** — if it needs thought, it stays with Claude
- **Edits to specific lines** — read the file directly when you need exact context for an edit
- **One-off lookups** — `grep` or reading a small file is faster than a round-trip to the API
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
