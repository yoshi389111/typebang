use anyhow::Context;

/// Checks if the messages indicate that input should be read from stdin.
fn is_used_stdin(messages: &[String]) -> bool {
    messages.is_empty() || (messages.len() == 1 && messages[0] == "-")
}

/// Reads lines from stdin and returns them as a vector of strings.
fn read_from_stdin() -> anyhow::Result<Vec<String>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut input = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.context("Failed to read line from stdin")?;
        if !line.trim().is_empty() {
            input.push(line + "\n");
        }
    }
    Ok(input)
}

/// Splits messages into lines and appends a newline to each line.
fn split_message(messages: &[String]) -> Vec<String> {
    messages
        .iter()
        .flat_map(|message| message.split('\n'))
        .map(|s| s.to_string() + "\n")
        .collect::<Vec<String>>()
}

pub fn run(args: &crate::Args) -> anyhow::Result<()> {
    if args.list_fonts {
        let mut out = typebang::io::Utf8Writer::new(std::io::stdout());
        typebang::font_list::print_font_families(&mut out)?;
        return Ok(());
    }

    let messages = if is_used_stdin(&args.messages) {
        read_from_stdin()?
    } else {
        split_message(&args.messages)
    };

    let io_writer: Box<dyn std::io::Write> = if let Some(output_path) = &args.output {
        Box::new(std::fs::File::create(output_path)?)
    } else {
        Box::new(std::io::stdout())
    };
    let mut out = typebang::io::Utf8Writer::new(io_writer);

    // Generate the SVG content
    typebang::svg::generate(
        &mut out,
        &messages,
        &args.font,
        &args.fg_color,
        &args.bg_color,
        &args.strong_color,
    )
}
