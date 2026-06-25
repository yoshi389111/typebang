use clap::Parser;
mod cli;

#[derive(clap::Parser)]
#[command(version, about)]
pub struct Args {
    /// Font name to use for rendering text
    #[arg(short, long, default_value = "monospace")]
    font: String,
    /// Font face to use for rendering text (only applicable when font is a file path)
    #[arg(long)]
    font_face: Option<String>,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,

    /// Foreground color to use for rendering text
    #[arg(long, env = "TYPEBANG_FG_COLOR", default_value = "black")]
    fg_color: String,
    /// Background color to use for rendering text
    #[arg(long, env = "TYPEBANG_BG_COLOR", default_value = "white")]
    bg_color: String,
    /// Strong color to use for rendering text
    #[arg(long, env = "TYPEBANG_STRONG_COLOR", default_value = "#ff2000")]
    strong_color: String,

    /// List available fonts
    #[arg(long, default_value_t = false)]
    list_fonts: bool,

    /// List available faces
    #[arg(long, default_value_t = false)]
    list_faces: bool,

    /// Text messages to render
    messages: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    cli::run(&args)
}
