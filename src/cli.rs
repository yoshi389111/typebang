use anyhow::Context;

/// Runs the main logic of the program.
pub fn run(args: &crate::Args) -> anyhow::Result<()> {
    if args.list_fonts {
        print_font_families()
    } else if args.list_faces {
        print_font_faces(args)
    } else {
        generate_svg(args)
    }
}

/// Generates the SVG output based on the provided arguments.
fn generate_svg(args: &crate::Args) -> anyhow::Result<()> {
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

    let font_filter = create_font_filter(&args.font, &args.font_face);

    let config = typebang::svg::Config {
        font_filter,

        fg_color: args.fg_color.clone(),
        bg_color: args.bg_color.clone(),
        strong_color: args.strong_color.clone(),

        angle: args.angle.map(|a| a as f32),

        messages,
    };

    // Generate the SVG content
    typebang::svg::generate(&mut out, &config)
}

/// Outputs the list of available font families to stdout.
fn print_font_families() -> anyhow::Result<()> {
    let families = typebang::font::get_families()?;

    println!("Generic families:");
    println!("  sans-serif");
    println!("  serif");
    println!("  monospace");
    println!();

    println!("Installed font families:");

    if families.is_empty() {
        println!("  No fonts found.");
    } else {
        for family in families {
            println!("  {family}");
        }
    }
    Ok(())
}

fn print_font_faces(args: &crate::Args) -> anyhow::Result<()> {
    let font_name = &args.font;
    let font_filter = create_font_filter(font_name, &None);
    let post_script_names = typebang::font::get_font_faces(&font_filter)?;

    println!("Font faces for '{}':", font_name);
    if post_script_names.is_empty() {
        println!("  No fonts found.");
    } else {
        for post_script_name in post_script_names {
            println!("  {post_script_name}");
        }
    }
    Ok(())
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

fn create_font_filter(
    font_path_or_name: &str,
    font_face: &Option<String>,
) -> typebang::font::Filter {
    if let Some(path) = get_valid_path(font_path_or_name) {
        typebang::font::Filter {
            source: typebang::font::Source::File(path),
            name: font_face.clone(),
        }
    } else {
        typebang::font::Filter {
            source: typebang::font::Source::System,
            name: Some(font_path_or_name.to_string()),
        }
    }
}

fn get_valid_path(path: &str) -> Option<std::path::PathBuf> {
    if !path.contains('/') && !path.contains('\\') {
        return None;
    }

    Some(std::path::Path::new(path).to_path_buf())
}
