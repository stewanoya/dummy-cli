use anyhow::{Context, Result};

use crate::cli::ReadArgs;
use crate::provider::Provider;
use crate::ui;

pub async fn run(provider: Provider, args: ReadArgs, model: &str) -> Result<()> {
    let corpus: String = args
        .paths
        .iter()
        .map(|p| {
            let content = std::fs::read_to_string(p)
                .with_context(|| format!("Cannot read {p}"))?;
            Ok(format!("<file path=\"{p}\">\n{content}\n</file>"))
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n\n");

    let detail = format!(
        "{} file{}",
        args.paths.len(),
        if args.paths.len() == 1 { "" } else { "s" }
    );
    ui::querying(model, &detail);

    let answer = provider
        .complete(
            "You are a precise code/document analyst. Read the provided files and answer \
             the question concisely. Quote file paths and line numbers when relevant. \
             Output structured bullets, not prose. Keep your answer under 800 words.",
            // Corpus and question are separate messages so the file content forms a
            // stable prefix — repeated reads of the same files get a cache hit.
            vec![
                format!("<corpus>\n{corpus}\n</corpus>"),
                args.question,
            ],
            args.max_tokens,
        )
        .await?;

    println!("{answer}");
    Ok(())
}
