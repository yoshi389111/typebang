use crate::font;
use crate::glyph;
use crate::number::round_float;

/// The height of the glyphs in the SVG viewBox.
const GLYPH_HEIGHT: f32 = glyph::GLYPH_HEIGHT;
const MARGIN_TOP: f32 = GLYPH_HEIGHT * 0.33;
const MARGIN_RIGHT: f32 = GLYPH_HEIGHT * 0.33;
const MARGIN_LEFT: f32 = MARGIN_RIGHT;
const MARGIN_BOTTOM: f32 = MARGIN_TOP;

const WAIT_TOP: f32 = 0.0; // [ms]
const WAIT_LAST: f32 = 1000.0; // [ms]
const WAIT_STRONG: f32 = 1000.0; // [ms]
const WAIT_CLACK: f32 = 100.0; // [ms]

fn extract_unique_characters(messages: &[String]) -> Vec<char> {
    let mut unique_chars = std::collections::HashSet::new();
    for message in messages {
        for ch in message.chars() {
            unique_chars.insert(ch);
        }
    }
    unique_chars.into_iter().collect()
}

fn calculate_message_width(
    glyph_map: &std::collections::BTreeMap<char, glyph::GlyphInfo>,
    message: &str,
) -> f32 {
    message
        .chars()
        .flat_map(|ch| glyph_map.get(&ch))
        .map(|glyph_info| glyph_info.width)
        .sum()
}

fn calculate_max_message_width(
    glyph_map: &std::collections::BTreeMap<char, glyph::GlyphInfo>,
    messages: &[String],
) -> f32 {
    messages
        .iter()
        .map(|message| calculate_message_width(glyph_map, message))
        .fold(0.0, f32::max)
}

fn create_glyph_map(
    face: &ttf_parser::Face,
    characters: &[char],
) -> std::collections::BTreeMap<char, glyph::GlyphInfo> {
    let mut glyph_map = std::collections::BTreeMap::new();
    for &ch in characters {
        if let Some(glyph_info) = glyph::create_glyph_info(face, ch) {
            glyph_map.insert(ch, glyph_info);
        }
    }
    glyph_map
}

fn decide_wait_time(ch: char, prev_ch: Option<char>) -> f32 {
    const TIME_QUICK: f32 = 200.0; // [ms]
    const TIME_PUNCH: f32 = 400.0; // [ms]
    const TIME_ENTER: f32 = 500.0; // [ms]
    if ch == '\n' {
        TIME_ENTER
    } else if let Some(prev) = prev_ch
        && prev.is_ascii_alphabetic()
        && ch.is_ascii_alphabetic()
        && !(prev.is_ascii_lowercase() && ch.is_ascii_uppercase())
    {
        TIME_QUICK + rand::random_range(0.0..200.0)
    } else {
        TIME_PUNCH + rand::random_range(0.0..100.0)
    }
}

struct CharInfo {
    ch: char,
    x_offset: f32,
    y_offset: f32,
    wait: f32, // [ms]
}

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

        pen_x += glyph_info.width;
        prev_ch = Some(ch);
    }
    char_info_list
}

fn create_clack_values(char_info_list: &[CharInfo]) -> String {
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

fn create_clack_key_times(char_info_list: &[CharInfo], total_time: f32) -> String {
    let mut key_times = String::new();
    key_times += "0;";
    let mut current_time = WAIT_TOP;
    for char_info in char_info_list {
        let t1 = round_float(current_time / total_time);
        let t2 = round_float((current_time + WAIT_CLACK) / total_time);
        key_times += &format!("{t1};{t1};{t2};");
        current_time += char_info.wait;
    }
    key_times += "1";
    key_times
}

pub fn generate(
    out: &mut impl std::fmt::Write,
    messages: &[String],
    font_name: &str,
    fg_color: &str,
    bg_color: &str,
    strong_color: &str,
) -> anyhow::Result<()> {
    let characters = extract_unique_characters(messages);
    let glyph_map = font::load_and_run(font_name, |face| create_glyph_map(face, &characters))?;

    let maximum_message_width = calculate_max_message_width(&glyph_map, messages);
    let height = GLYPH_HEIGHT;
    let canvas_width = round_float(MARGIN_LEFT + maximum_message_width + MARGIN_RIGHT);
    let canvas_height = round_float(MARGIN_TOP + height + MARGIN_BOTTOM);

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
            char_info_list.iter().map(|info| info.wait).sum::<f32>() + WAIT_TOP + WAIT_LAST;

        let trigger = if i == 0 {
            &format!("500ms;line{}.end", messages.len() - 1)
        } else {
            &format!("line{}.end", i - 1)
        };
        let base_trigger = &format!("line{i}.begin");
        let t3 = round_float((total_time - WAIT_LAST) / total_time);

        writeln!(out, "<g opacity=\"0\">")?;
        writeln!(
            out,
            "<animate \
                attributeName=\"opacity\" \
                id=\"line{i}\" \
                begin=\"{trigger}\" \
                values=\"1;1;0\" \
                dur=\"{total_time}ms\" \
                keyTimes=\"0;{t3};1\" \
            />"
        )?;
        writeln!(
            out,
            "<animateTransform \
                attributeName=\"transform\" \
                begin=\"{base_trigger}\" \
                type=\"translate\" \
                values=\"0,0;0,0;0,-{GLYPH_HEIGHT}\" \
                dur=\"{total_time}ms\" \
                keyTimes=\"0;{t3};1\" \
            />"
        )?;
        writeln!(out, "<g>")?;

        let clack_values = create_clack_values(&char_info_list);
        let clack_key_times = create_clack_key_times(&char_info_list, total_time);
        writeln!(
            out,
            "<animateTransform \
                attributeName=\"transform\" \
                begin=\"{base_trigger}\" \
                type=\"translate\" \
                values=\"{clack_values}\" \
                dur=\"{total_time}ms\" \
                keyTimes=\"{clack_key_times}\" \
            />"
        )?;

        let mut current_time = WAIT_TOP;
        for (j, char_info) in char_info_list.iter().enumerate() {
            let glyph_number = u32::from(char_info.ch);
            let x = round_float(MARGIN_LEFT + char_info.x_offset);
            let y = round_float(MARGIN_TOP + char_info.y_offset);
            let fg = fg_color;
            let bg = bg_color;
            let st = strong_color;
            writeln!(
                out,
                "<use \
                    id=\"l{i}c{j}\" \
                    href=\"#glyph{glyph_number}\" \
                    x=\"{x}\" \
                    y=\"{y}\" \
                    fill=\"{fg}\" \
                >"
            )?;
            let t1 = round_float(current_time / total_time);
            let t2 = round_float((current_time + WAIT_STRONG) / total_time);
            writeln!(
                out,
                "<animate \
                    attributeName=\"fill\" \
                    begin=\"{base_trigger}\" \
                    values=\"{bg};{bg};{st};{fg};{fg}\" \
                    dur=\"{total_time}ms\" \
                    keyTimes=\"0;{t1};{t1};{t2};1\" \
                />"
            )?;
            if glyph_number <= 0x20 {
                writeln!(
                    out,
                    "<animate \
                        attributeName=\"opacity\" \
                        begin=\"{base_trigger}\" \
                        values=\"0;0;1;0;0\" \
                        dur=\"{total_time}ms\" \
                        keyTimes=\"0;{t1};{t1};{t2};1\" \
                    />"
                )?;
            } else {
                writeln!(
                    out,
                    "<animate \
                        attributeName=\"stroke\" \
                        begin=\"{base_trigger}\" \
                        values=\"{bg};{bg};{st};{fg};{fg}\" \
                        dur=\"{total_time}ms\" \
                        keyTimes=\"0;{t1};{t1};{t2};1\" \
                    />"
                )?;
                writeln!(
                    out,
                    "<animate \
                        attributeName=\"stroke-width\" \
                        begin=\"{base_trigger}\" \
                        values=\"0;0;0.7;0;0\" \
                        dur=\"{total_time}ms\" \
                        keyTimes=\"0;{t1};{t1};{t2};1\" \
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
