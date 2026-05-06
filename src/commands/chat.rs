use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value;

use crate::cli::ChatArgs;

struct Msg {
    role: String,
    timestamp: String,
    text: String,
}

pub fn run(args: ChatArgs) -> Result<()> {
    let src = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Cannot read {}", args.input))?;

    let msgs = parse(&src);

    match args.output {
        Some(ref path) => {
            let content = render_plain(&msgs);
            std::fs::write(path, &content)
                .with_context(|| format!("Cannot write {path}"))?;
            eprintln!(
                "  {}  {}",
                path.bold(),
                format!("({} messages)", msgs.len()).dimmed()
            );
        }
        None => render_terminal(&msgs),
    }

    Ok(())
}

fn parse(src: &str) -> Vec<Msg> {
    let mut msgs = Vec::new();

    for line in src.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        let msg_type = msg["type"].as_str().unwrap_or("");
        if msg_type != "user" && msg_type != "assistant" {
            continue;
        }

        let inner = &msg["message"];
        let role = inner["role"].as_str().unwrap_or(msg_type).to_string();
        let content = &inner["content"];
        let timestamp = msg["timestamp"].as_str().unwrap_or("").to_string();

        let texts: Vec<&str> = match content {
            Value::String(s) => {
                let t = s.trim();
                if t.is_empty() {
                    continue;
                }
                vec![t]
            }
            Value::Array(blocks) => blocks
                .iter()
                .filter(|b| b["type"].as_str() == Some("text"))
                .filter_map(|b| b["text"].as_str())
                .filter(|t| !t.trim().is_empty())
                .collect(),
            _ => continue,
        };

        if texts.is_empty() {
            continue;
        }

        msgs.push(Msg { role, timestamp, text: texts.join("\n") });
    }

    msgs
}

fn render_plain(msgs: &[Msg]) -> String {
    msgs.iter()
        .map(|m| {
            let ts = if m.timestamp.is_empty() {
                String::new()
            } else {
                format!(" ({})", m.timestamp)
            };
            format!("[{}]{}:\n{}", m.role.to_uppercase(), ts, m.text)
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

fn render_terminal(msgs: &[Msg]) {
    let sep = "─".repeat(60);

    for (i, m) in msgs.iter().enumerate() {
        if i > 0 {
            println!("\n{}\n", sep.dimmed());
        }

        // Role label
        if m.role == "user" {
            print!("{}", "USER".cyan().bold());
        } else {
            print!("{}", "ASSISTANT".green().bold());
        }

        // Timestamp (dim, on same line)
        if !m.timestamp.is_empty() {
            print!("  {}", m.timestamp.dimmed());
        }
        println!();
        println!();

        println!("{}", m.text);
    }
}
