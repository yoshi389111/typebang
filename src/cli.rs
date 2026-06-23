use anyhow::Context;

/// Runs the main logic of the program.
pub fn run(args: &crate::Args) -> anyhow::Result<()> {
    if args.list_fonts {
        print_font_families()
    } else {
        output_svg(args)
    }
}

/// Generates the SVG output based on the provided arguments.
fn output_svg(args: &crate::Args) -> anyhow::Result<()> {
    let messages = if is_used_stdin(&args.messages) {
        read_from_stdin()?
    } else {
        split_message(&args.messages)
    };

    let io_writer: Box<dyn std::io::Write> = if let Some(output_path) = &args.output {
        let file = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create output file '{}'", output_path))?;
        Box::new(file)
    } else {
        Box::new(std::io::stdout())
    };
    let mut out = typebang::io::Utf8Writer::new(io_writer);

    let config = typebang::svg::Config {
        font_name: args.font.clone(),
        fg_color: args.fg_color.clone(),
        bg_color: args.bg_color.clone(),
        strong_color: args.strong_color.clone(),
        messages,
    };

    // Generate the SVG content
    typebang::svg::generate(&mut out, &config)
}

/// Outputs the list of available font families to stdout.
fn print_font_families() -> anyhow::Result<()> {
    let mut out = typebang::io::Utf8Writer::new(std::io::stdout());
    typebang::font_list::print_font_families(&mut out)
}

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
