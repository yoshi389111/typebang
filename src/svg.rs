use crate::font;
use crate::glyph;
use crate::number::round_float;

/// The time to wait before starting the first line of text.
const TIME_START: f32 = 500.0; // [ms]
/// The time to wait after displaying a line of text before starting the next line.
const TIME_BETWEEN_LINES: f32 = 1000.0; // [ms]

/// The time duration for displaying a character quickly.
const TIME_QUICK_PUNCH: f32 = 200.0; // [ms]
/// The time duration for displaying a normal character.
const TIME_KEY_PUNCH: f32 = 400.0; // [ms]
/// The time duration for displaying a newline character.
const TIME_ENTER_KEY: f32 = 500.0; // [ms]

/// The time duration for displaying a character with emphasis effect.
const TIME_EMPHASIS_EFFECT: f32 = 1000.0; // [ms]
/// The time duration for the "jolt" effect when displaying a character.
const TIME_JOLT_EFFECT: f32 = 100.0; // [ms]

/// Configuration for generating SVG output.
pub struct Config {
    pub font_name: String,

    pub fg_color: String,
    pub bg_color: String,
    pub strong_color: String,

    pub messages: Vec<String>,
}

/// Generates an SVG representation of the provided messages using the specified font and colors.
pub fn generate(out: &mut impl std::fmt::Write, config: &Config) -> anyhow::Result<()> {
    const GLYPH_HEIGHT: f32 = glyph::GLYPH_HEIGHT;

    const MARGIN_TOP: f32 = GLYPH_HEIGHT * 0.33;
    const MARGIN_BOTTOM: f32 = MARGIN_TOP;
    const MARGIN_LEFT: f32 = GLYPH_HEIGHT * 0.33;
    const MARGIN_RIGHT: f32 = MARGIN_LEFT;

    let Config {
        font_name,
        fg_color,
        bg_color,
        strong_color,
        messages,
    } = config;

    let characters = extract_unique_characters(messages);
    let glyph_map =
        font::load_and_run(font_name, |face| glyph::create_glyph_map(face, &characters))?;

    let maximum_message_width = calculate_max_message_width(&glyph_map, messages);
    let canvas_width = round_float(MARGIN_LEFT + maximum_message_width + MARGIN_RIGHT);
    let canvas_height = round_float(MARGIN_TOP + GLYPH_HEIGHT + MARGIN_BOTTOM);

    // SVG header
    writeln!(
        out,
        "<svg \
            xmlns=\"http://www.w3.org/2000/svg\" \
            width=\"{canvas_width}\" \
            height=\"{canvas_height}\" \
            viewBox=\"0 0 {canvas_width} {canvas_height}\" \
        >",
    )?;

    // define symbols for each glyph
    writeln!(out, "<defs>")?;
    for glyph_info in glyph_map.values() {
        writeln!(out, "{}", glyph_info.symbol)?;
    }
    writeln!(out, "</defs>")?;

    // background rectangle
    writeln!(
        out,
        "<rect \
            x=\"0\" \
            y=\"0\" \
            width=\"{canvas_width}\" \
            height=\"{canvas_height}\" \
            fill=\"{bg_color}\" \
        />"
    )?;

    // render each message using the defined symbols
    for (i, message) in messages.iter().enumerate() {
        let char_info_list = create_char_info_list(message, &glyph_map);

        let total_time =
            char_info_list.iter().map(|info| info.wait).sum::<f32>() + TIME_BETWEEN_LINES;

        let trigger = if i == 0 {
            let last_number = messages.len() - 1;
            &format!("{TIME_START}ms;line{last_number}.end")
        } else {
            let prev_number = i - 1;
            &format!("line{prev_number}.end")
        };
        let line_begin_trigger = &format!("line{i}.begin");
        let timing_last = create_timing(total_time - TIME_BETWEEN_LINES, total_time);

        writeln!(out, "<g opacity=\"0\">")?;
        writeln!(
            out,
            "<animate \
                attributeName=\"opacity\" \
                id=\"line{i}\" \
                begin=\"{trigger}\" \
                values=\"1;1;0\" \
                dur=\"{total_time}ms\" \
                keyTimes=\"0;{timing_last};1\" \
            />"
        )?;
        writeln!(
            out,
            "<animateTransform \
                attributeName=\"transform\" \
                type=\"translate\" \
                begin=\"{line_begin_trigger}\" \
                values=\"0,0;0,0;0,-{GLYPH_HEIGHT}\" \
                dur=\"{total_time}ms\" \
                keyTimes=\"0;{timing_last};1\" \
            />"
        )?;
        writeln!(out, "<g>")?;

        let jolt_values = create_jolt_values(&char_info_list);
        let jolt_key_times = create_jolt_key_times(&char_info_list, total_time);
        writeln!(
            out,
            "<animateTransform \
                attributeName=\"transform\" \
                type=\"translate\" \
                begin=\"{line_begin_trigger}\" \
                values=\"{jolt_values}\" \
                dur=\"{total_time}ms\" \
                keyTimes=\"{jolt_key_times}\" \
            />"
        )?;

        let mut current_time = 0.0;
        for char_info in char_info_list.iter() {
            let glyph_number = u32::from(char_info.ch);
            let x = round_float(MARGIN_LEFT + char_info.x_offset);
            let y = round_float(MARGIN_TOP + char_info.y_offset);
            let fg = fg_color;
            let bg = bg_color;
            let st = strong_color;
            writeln!(
                out,
                "<use \
                    href=\"#glyph{glyph_number}\" \
                    x=\"{x}\" \
                    y=\"{y}\" \
                    fill=\"{fg}\" \
                >"
            )?;
            let timing_start = create_timing(current_time, total_time);
            let timing_strong_end = create_timing(current_time + TIME_EMPHASIS_EFFECT, total_time);
            writeln!(
                out,
                "<animate \
                    attributeName=\"fill\" \
                    begin=\"{line_begin_trigger}\" \
                    values=\"{bg};{bg};{st};{fg};{fg}\" \
                    dur=\"{total_time}ms\" \
                    keyTimes=\"0;{timing_start};{timing_start};{timing_strong_end};1\" \
                />"
            )?;
            if glyph_number <= 0x20 {
                writeln!(
                    out,
                    "<animate \
                        attributeName=\"opacity\" \
                        begin=\"{line_begin_trigger}\" \
                        values=\"0;0;1;0;0\" \
                        dur=\"{total_time}ms\" \
                        keyTimes=\"0;{timing_start};{timing_start};{timing_strong_end};1\" \
                    />"
                )?;
            } else {
                writeln!(
                    out,
                    "<animate \
                        attributeName=\"stroke\" \
                        begin=\"{line_begin_trigger}\" \
                        values=\"{bg};{bg};{st};{fg};{fg}\" \
                        dur=\"{total_time}ms\" \
                        keyTimes=\"0;{timing_start};{timing_start};{timing_strong_end};1\" \
                    />"
                )?;
                writeln!(
                    out,
                    "<animate \
                        attributeName=\"stroke-width\" \
                        begin=\"{line_begin_trigger}\" \
                        values=\"0;0;2;0;0\" \
                        dur=\"{total_time}ms\" \
                        keyTimes=\"0;{timing_start};{timing_start};{timing_strong_end};1\" \
                    />"
                )?;
            }
            writeln!(out, "</use>")?;
            current_time += char_info.wait;
        }
        writeln!(out, "</g>")?;
        writeln!(out, "</g>")?;
    }

    writeln!(out, "</svg>")?;
    Ok(())
}

/// Extracts unique characters from the provided messages and returns them as a vector of characters.
fn extract_unique_characters(messages: &[String]) -> Vec<char> {
    let unique_chars = messages
        .iter()
        .flat_map(|message| message.chars())
        .collect::<std::collections::HashSet<_>>();
    unique_chars.into_iter().collect()
}

/// Calculates the maximum width of the provided messages based on the glyph information.
fn calculate_max_message_width(
    glyph_map: &std::collections::BTreeMap<char, glyph::GlyphInfo>,
    messages: &[String],
) -> f32 {
    messages
        .iter()
        .map(|message| calculate_message_width(glyph_map, message))
        .fold(0.0, f32::max)
}

/// Calculates the width of a single message based on the glyph information.
fn calculate_message_width(
    glyph_map: &std::collections::BTreeMap<char, glyph::GlyphInfo>,
    message: &str,
) -> f32 {
    message
        .chars()
        .flat_map(|ch| glyph_map.get(&ch))
        .map(|glyph_info| glyph_info.feed_width)
        .sum()
}

/// Represents information about a character, including its offsets and wait time.
struct CharInfo {
    ch: char,
    x_offset: f32,
    y_offset: f32,
    wait: f32, // [ms]
}

/// Creates a list of character information for the provided message based on the glyph map.
fn create_char_info_list(
    message: &str,
    glyph_map: &std::collections::BTreeMap<char, glyph::GlyphInfo>,
) -> Vec<CharInfo> {
    let mut char_info_list = Vec::new();
    let mut pen_x = 0.0;
    let mut prev_ch = None;
    for ch in message.chars() {
        let Some(glyph_info) = glyph_map.get(&ch) else {
            // Skip characters that don't have a corresponding glyph
            continue;
        };

        char_info_list.push(CharInfo {
            ch,
            x_offset: pen_x + glyph_info.x_offset,
            y_offset: glyph_info.y_offset,
            wait: decide_wait_time(ch, prev_ch),
        });

        pen_x += glyph_info.feed_width;
        prev_ch = Some(ch);
    }
    char_info_list
}

/// Determines the wait time for a character based on its type and the previous character.
fn decide_wait_time(ch: char, prev_ch: Option<char>) -> f32 {
    if ch == '\n' {
        TIME_ENTER_KEY
    } else if let Some(prev) = prev_ch
        && prev.is_ascii_alphabetic()
        && ch.is_ascii_alphabetic()
        && !(prev.is_ascii_lowercase() && ch.is_ascii_uppercase())
    {
        TIME_QUICK_PUNCH + rand::random_range(0.0..200.0)
    } else {
        TIME_KEY_PUNCH + rand::random_range(0.0..100.0)
    }
}

/// Creates a normalized timing value for a target time relative to the total time.
fn create_timing(target_time: f32, total_time: f32) -> String {
    round_float(target_time / total_time)
}

/// Creates a string representing the key times for the "jolt" effect based on the character information and total time.
fn create_jolt_key_times(char_info_list: &[CharInfo], total_time: f32) -> String {
    let mut key_times = String::new();
    key_times += "0;";
    let mut current_time = 0.0;
    for char_info in char_info_list {
        let time_jolt_start = round_float(current_time / total_time);
        let time_jolt_end = round_float((current_time + TIME_JOLT_EFFECT) / total_time);
        key_times += &format!("{time_jolt_start};{time_jolt_start};{time_jolt_end};");
        current_time += char_info.wait;
    }
    key_times += "1";
    key_times
}

/// Creates a string representing the values for the "jolt" effect based on the character information.
fn create_jolt_values(char_info_list: &[CharInfo]) -> String {
    let mut values = String::new();
    values += "0 0;";
    for char_info in char_info_list {
        let ch = char_info.ch;
        let (dx, dy) = if ch == '\n' {
            (-9.0, 10.0)
        } else {
            let dx = (rand::random_range(-1..=1) + rand::random_range(-1..=1)) as f32 * 2.0;
            let dy = rand::random_range(3..6) as f32;
            (dx, dy)
        };
        let dx = round_float(dx);
        let dy = round_float(dy);
        values += &format!("0 0;{dx} {dy};0 0;");
    }
    values += "0 0";
    values
}
