use anyhow::Result;

use crate::cli::{Cli, Command};
use crate::config::{config_path, Config};
use crate::provider::Provider;
use crate::ui;

mod chat;
mod read;
mod write;

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Config => {
            let path = config_path();
            let exists = path.exists();
            ui::kv_note("config", &path.display().to_string(), if exists { "found" } else { "not found" });
            println!();
            let cfg = Config::load()?;
            match cfg.resolve_provider(cli.provider.as_deref()) {
                Ok(p) => {
                    let masked = if p.api_key.len() > 8 {
                        format!("{}...", &p.api_key[..8])
                    } else {
                        "***".to_string()
                    };
                    ui::kv("api_type", &p.api_type.to_string());
                    ui::kv("base_url", p.base_url.as_deref().unwrap_or("(default)"));
                    ui::kv("model", &p.model);
                    ui::kv("api_key", &masked);
                }
                Err(e) => {
                    println!();
                    eprintln!("  no provider configured: {e}");
                }
            }
            Ok(())
        }
        Command::Read(args) => {
            let cfg = Config::load()?.resolve_provider(cli.provider.as_deref())?;
            let model = cfg.model.clone();
            let provider = Provider::from_config(cfg);
            read::run(provider, args, &model).await
        }
        Command::Write(args) => {
            let cfg = Config::load()?.resolve_provider(cli.provider.as_deref())?;
            let model = cfg.model.clone();
            let provider = Provider::from_config(cfg);
            write::run(provider, args, &model).await
        }
        Command::Chat(args) => chat::run(args),
    }
}
