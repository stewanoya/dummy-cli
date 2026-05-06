use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "dummy",
    about = "Delegate bulk I/O tasks to cheap LLM providers to save expensive reasoning tokens",
    long_about = "dummy routes read/write tasks to inexpensive LLMs (DeepSeek, Kimi, Ollama, etc.)\n\
                  so Claude can focus on reasoning. All providers use the OpenAI-compatible API.\n\
                  \n\
                  QUICK START (env vars):\n\
                  \n\
                    set DUMMY_API_KEY=sk-...\n\
                    set DUMMY_BASE_URL=https://api.deepseek.com/v1\n\
                    set DUMMY_MODEL=deepseek-chat\n\
                  \n\
                  CONFIG FILE (%APPDATA%/dummy/config.toml or ~/.config/dummy/config.toml):\n\
                  \n\
                    default_provider = \"deepseek\"\n\
                  \n\
                    [providers.deepseek]\n\
                    api_key  = \"sk-...\"\n\
                    base_url = \"https://api.deepseek.com/v1\"\n\
                    model    = \"deepseek-chat\"\n\
                  \n\
                    [providers.kimi]\n\
                    api_key  = \"...\"\n\
                    base_url = \"https://api.moonshot.ai/v1\"\n\
                    model    = \"kimi-k2.5\"\n\
                  \n\
                    [providers.ollama]\n\
                    api_key  = \"ollama\"\n\
                    base_url = \"http://localhost:11434/v1\"\n\
                    model    = \"llama3\"\n\
                  \n\
                  ENV VAR OVERRIDES (always win over config file):\n\
                    DUMMY_API_KEY, DUMMY_BASE_URL, DUMMY_MODEL, DUMMY_PROVIDER",
    version
)]
pub struct Cli {
    /// Provider name from config file (or set DUMMY_PROVIDER env var)
    #[arg(long, global = true, env = "DUMMY_PROVIDER", value_name = "NAME")]
    pub provider: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Read files and answer a question — cheaper than Claude reading files directly
    Read(ReadArgs),
    /// Generate code or docs to a file — Claude reviews and edits the result
    Write(WriteArgs),
    /// Extract human-readable chat from a Claude Code JSONL session transcript
    Chat(ChatArgs),
    /// Show the resolved provider config (for debugging)
    Config,
}

#[derive(Args)]
#[command(
    long_about = "Sends file contents to a cheap LLM and returns a structured answer.\n\
                  Use when Claude would otherwise read 3+ files or any single file >400 lines.\n\
                  Output goes to stdout so Claude reads it instead of the raw files.\n\
                  \n\
                  EXAMPLES:\n\
                  \n\
                    dummy read --paths src/main.rs src/lib.rs --question \"What does the auth flow do?\"\n\
                    dummy read --paths schema.sql --question \"List all foreign key relationships\"\n\
                    dummy read --paths Cargo.toml --question \"What version is reqwest?\"\n\
                  \n\
                  NOTE: For thinking models (deepseek-reasoner, kimi-k2.5), reasoning tokens\n\
                  count toward --max-tokens. Increase to 16384 if responses are cut off."
)]
pub struct ReadArgs {
    /// One or more files to read and analyze
    #[arg(long, required = true, num_args = 1.., value_name = "FILE")]
    pub paths: Vec<String>,

    /// Question to answer about the files
    #[arg(long, required = true)]
    pub question: String,

    /// Max response tokens — increase for thinking models (they use tokens for internal reasoning)
    #[arg(long, default_value = "8192", value_name = "N")]
    pub max_tokens: u32,
}

#[derive(Args)]
#[command(
    long_about = "Asks a cheap LLM to generate boilerplate, tests, configs, or documentation.\n\
                  Writes directly to --target. Claude then reviews and makes surgical edits.\n\
                  Best for repetitive or templated content where structure is predictable.\n\
                  \n\
                  EXAMPLES:\n\
                  \n\
                    dummy write --spec \"unit tests for all public functions\" \\\n\
                            --context src/lib.rs --target tests/lib_test.rs\n\
                  \n\
                    dummy write --spec \"OpenAPI 3.0 spec for the routes in main.rs\" \\\n\
                            --context src/main.rs --target docs/api.yaml\n\
                  \n\
                    dummy write --spec \"SQL migration to add index on users.email\" \\\n\
                            --context migrations/001_init.sql --target migrations/002_idx.sql\n\
                  \n\
                  NOTE: For thinking models, increase --max-tokens to 32768 for large outputs."
)]
pub struct WriteArgs {
    /// What to generate — be specific about format, language, and scope
    #[arg(long, required = true)]
    pub spec: String,

    /// Output file to write
    #[arg(long, required = true, value_name = "FILE")]
    pub target: String,

    /// Reference file to match style, imports, and conventions
    #[arg(long, value_name = "FILE")]
    pub context: Option<String>,

    /// Max response tokens — increase for thinking models or large output files
    #[arg(long, default_value = "16384", value_name = "N")]
    pub max_tokens: u32,
}

#[derive(Args)]
#[command(
    long_about = "Strips tool calls, system prompts, thinking blocks, and metadata from a\n\
                  Claude Code JSONL session log. Outputs only human/assistant text.\n\
                  No LLM call — runs locally and instantly.\n\
                  \n\
                  EXAMPLES:\n\
                  \n\
                    dummy chat --input session.jsonl\n\
                    dummy chat --input session.jsonl --output /tmp/chat.txt\n\
                  \n\
                  TYPICAL WORKFLOW (documentation updates):\n\
                  \n\
                    dummy chat --input session.jsonl --output /tmp/chat.txt\n\
                    dummy read --paths /tmp/chat.txt docs/CHANGELOG.md \\\n\
                               --question \"What changelog entries should I add?\""
)]
pub struct ChatArgs {
    /// Claude Code JSONL transcript file
    #[arg(long, required = true, value_name = "FILE")]
    pub input: String,

    /// Write output to file instead of stdout
    #[arg(long, short, value_name = "FILE")]
    pub output: Option<String>,
}
