use anyhow::{Context, Result};

use crate::cli::WriteArgs;
use crate::provider::Provider;
use crate::ui;

pub async fn run(provider: Provider, args: WriteArgs, model: &str) -> Result<()> {
    let mut user_msg = String::new();

    if let Some(ctx_path) = &args.context {
        let content = std::fs::read_to_string(ctx_path)
            .with_context(|| format!("Cannot read context file {ctx_path}"))?;
        user_msg.push_str(&format!(
            "<reference path=\"{ctx_path}\">\n{content}\n</reference>\n\n"
        ));
    }
    user_msg.push_str(&format!("Write: {}", args.spec));

    ui::generating(model, &args.target);

    let mut output = provider
        .complete(
            "Generate clean, idiomatic code matching the style of any reference provided. \
             No explanations. No markdown fences. Output ONLY the file contents.",
            vec![user_msg],
            args.max_tokens,
        )
        .await?;

    // Strip accidental markdown fences some models add despite instructions
    if output.starts_with("```") {
        if let Some((_, rest)) = output.split_once('\n') {
            output = rest
                .rsplit_once("```")
                .map(|(body, _)| body.to_string())
                .unwrap_or_else(|| rest.to_string());
        }
    }

    std::fs::write(&args.target, &output)
        .with_context(|| format!("Cannot write {}", args.target))?;

    ui::written(&args.target, output.len());
    Ok(())
}
