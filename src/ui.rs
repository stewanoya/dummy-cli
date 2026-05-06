use colored::Colorize;

/// Usage stats line printed to stderr after every LLM call.
pub fn stats(model: &str, prompt: u32, completion: u32, finish: &str) {
    eprintln!(
        "\n  {}  {}  {}",
        format!("{} in / {} out", prompt, completion).cyan(),
        model.dimmed(),
        finish.dimmed()
    );
}

/// Progress hint printed before an API call.
pub fn querying(model: &str, detail: &str) {
    eprintln!("  {}  {}", model.bold(), detail.dimmed());
    eprintln!();
}

/// Progress hint before a write/generate call.
pub fn generating(model: &str, target: &str) {
    eprintln!("  {}  {}", model.bold(), format!("→ {target}").dimmed());
    eprintln!();
}

/// Success line after writing a file.
pub fn written(path: &str, bytes: usize) {
    eprintln!(
        "  {}  {}",
        path.bold(),
        format!("({bytes} bytes)").dimmed()
    );
}

/// Key/value row for the config command (stdout).
pub fn kv(key: &str, value: &str) {
    println!("  {:<12}  {}", key.dimmed(), value.bold());
}

/// Key/value row with a parenthetical note.
pub fn kv_note(key: &str, value: &str, note: &str) {
    println!(
        "  {:<12}  {}  {}",
        key.dimmed(),
        value.bold(),
        format!("({note})").dimmed()
    );
}
