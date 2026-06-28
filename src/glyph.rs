use crate::number::round_float;
use crate::svg_path::SvgPathBuilder;
use ttf_parser::OutlineBuilder;

/// The height of the glyphs in the SVG output.
pub const GLYPH_HEIGHT: f32 = 100.0;

pub struct GlyphConfig<'a> {
    pub face: &'a ttf_parser::Face<'a>,
    pub angle: Option<f32>,
}

/// Represents information about a glyph.
pub struct GlyphInfo {
    /// X-coordinate offset from the pen position at the top-left of the glyph.
    pub x_offset: f32,
    /// Y-coordinate offset from the pen position at the top-left of the glyph.
    pub y_offset: f32,
    /// Width to advance the pen position after rendering this glyph.
    pub feed_width: f32,
    /// SVG symbol representation of the glyph.
    pub symbol: String,
}

/// Creates a mapping of characters to their corresponding glyph information for the given font face.
pub fn create_glyph_map(
    config: &GlyphConfig,
    characters: &[char],
) -> std::collections::BTreeMap<char, GlyphInfo> {
    let mut glyph_map = std::collections::BTreeMap::new();
    for &ch in characters {
        if let Some(glyph_info) = create_glyph_info(config, ch) {
            glyph_map.insert(ch, glyph_info);
        }
    }
    glyph_map
}

/// Creates glyph information for a specific character based on the provided font face.
pub fn create_glyph_info(config: &GlyphConfig, ch: char) -> Option<GlyphInfo> {
    match ch {
        ' ' => create_space_glyph_info(config),
        '\n' => create_enter_glyph_info(config),
        _ => create_normal_glyph_info(config, ch),
    }
}

/// Creates glyph information for a normal character (not space or enter) based on the provided font face.
fn create_normal_glyph_info(config: &GlyphConfig, ch: char) -> Option<GlyphInfo> {
    let face = config.face;
    let glyph_id = face.glyph_index(ch)?;
    let scale = GLYPH_HEIGHT / face.height() as f32;

    let ascender_raw = face.ascender() as f32;

    let path_data = {
        let mut builder = SvgPathBuilder::new(scale, ascender_raw);
        face.outline_glyph(glyph_id, &mut builder);
        builder.path_data
    };

    let bbox_raw = face.glyph_bounding_box(glyph_id)?;
    let x_offset = bbox_raw.x_min as f32 * scale;
    let y_offset = (ascender_raw - bbox_raw.y_max as f32) * scale;

    let symbol = create_symbol_path(
        ch,
        x_offset,
        y_offset,
        (bbox_raw.x_max as f32 - bbox_raw.x_min as f32) * scale,
        (bbox_raw.y_max as f32 - bbox_raw.y_min as f32) * scale,
        &path_data,
    );

    // Note that the width of the bbox may exceed the advance width.
    let feed_width = face.glyph_hor_advance(glyph_id)? as f32 * scale;

    let glyph_info = GlyphInfo {
        feed_width,
        symbol,
        x_offset,
        y_offset,
    };
    Some(glyph_info)
}

/// Creates glyph information for the space character based on the provided font face.
fn create_space_glyph_info(config: &GlyphConfig) -> Option<GlyphInfo> {
    let face = config.face;
    let glyph_id = face.glyph_index(' ')?;
    let scale = GLYPH_HEIGHT / face.height() as f32;

    let width_raw = face.glyph_hor_advance(glyph_id)? as f32;
    let ascender_raw = face.ascender() as f32;
    let stroke_width_raw = ascender_raw * 0.05;
    let height_raw = ascender_raw * 0.7; // 40 - 100%

    let tangent_of_angle = config
        .angle
        .unwrap_or(face.italic_angle())
        .clamp(-45.0, 45.0)
        .to_radians()
        .tan();
    let tangent_of_angle = if tangent_of_angle.is_finite() {
        tangent_of_angle
    } else {
        0.0
    };
    let path_data = {
        let mut builder = SvgPathBuilder::new(scale, ascender_raw);
        // Draw the outer rectangle.
        builder.move_to(0.0, 0.0);
        builder.line_to(width_raw * 0.8, 0.0);
        builder.line_to(width_raw * 0.8 - tangent_of_angle * height_raw, height_raw);
        builder.line_to(0.0 - tangent_of_angle * height_raw, height_raw);
        builder.close();
        // Draw the inner rectangle.
        builder.move_to(
            stroke_width_raw - tangent_of_angle * stroke_width_raw,
            stroke_width_raw,
        );
        builder.line_to(
            stroke_width_raw - tangent_of_angle * (height_raw - stroke_width_raw),
            height_raw - stroke_width_raw,
        );
        builder.line_to(
            width_raw * 0.8 - stroke_width_raw - tangent_of_angle * (height_raw - stroke_width_raw),
            height_raw - stroke_width_raw,
        );
        builder.line_to(
            width_raw * 0.8 - stroke_width_raw - tangent_of_angle * stroke_width_raw,
            stroke_width_raw,
        );
        builder.close();
        builder.path_data
    };

    let width = width_raw * scale;
    let height = GLYPH_HEIGHT; // as face.height() * scale;
    let box_left = (-tangent_of_angle * GLYPH_HEIGHT).min(0.0);
    let box_width = width + (tangent_of_angle * GLYPH_HEIGHT).abs();
    let symbol = create_symbol_path(' ', box_left, 0.0, box_width, height, &path_data);

    let glyph_info = GlyphInfo {
        feed_width: width,
        symbol,
        x_offset: box_left,
        y_offset: 0.0,
    };
    Some(glyph_info)
}

/// Creates glyph information for the enter character based on the provided font face.
fn create_enter_glyph_info(config: &GlyphConfig) -> Option<GlyphInfo> {
    let face = config.face;
    let scale = GLYPH_HEIGHT / face.height() as f32;

    let ascender_raw = face.ascender() as f32;
    let stroke_width_raw = ascender_raw * 0.05;
    let width_raw = stroke_width_raw * 8.0;
    let height_raw = ascender_raw * 0.7; // 40 - 100%

    let tangent_of_angle = config
        .angle
        .unwrap_or(face.italic_angle())
        .clamp(-45.0, 45.0)
        .to_radians()
        .tan();
    let tangent_of_angle = if tangent_of_angle.is_finite() {
        tangent_of_angle
    } else {
        0.0
    };
    let path_data = {
        let mut builder = SvgPathBuilder::new(scale, ascender_raw);
        // Draw the outer rectangle.
        builder.move_to(0.0, 0.0);
        builder.line_to(width_raw, 0.0);
        builder.line_to(width_raw - tangent_of_angle * height_raw, height_raw);
        builder.line_to(0.0 - tangent_of_angle * height_raw, height_raw);
        builder.close();
        // Draw the inner rectangle.
        builder.move_to(
            stroke_width_raw - tangent_of_angle * stroke_width_raw,
            stroke_width_raw,
        );
        builder.line_to(
            stroke_width_raw - tangent_of_angle * (height_raw - stroke_width_raw),
            height_raw - stroke_width_raw,
        );
        builder.line_to(
            width_raw - stroke_width_raw - tangent_of_angle * (height_raw - stroke_width_raw),
            height_raw - stroke_width_raw,
        );
        builder.line_to(
            width_raw - stroke_width_raw - tangent_of_angle * stroke_width_raw,
            stroke_width_raw,
        );
        builder.close();
        // Draw the inner arrow shape.
        builder.move_to(
            stroke_width_raw * 3.5 - tangent_of_angle * stroke_width_raw * 4.0,
            stroke_width_raw * 4.0,
        );
        builder.line_to(
            stroke_width_raw * 3.5 - tangent_of_angle * stroke_width_raw * 5.0,
            stroke_width_raw * 5.0,
        );
        builder.line_to(
            stroke_width_raw * 2.0 - tangent_of_angle * stroke_width_raw * 3.5,
            stroke_width_raw * 3.5,
        );
        builder.line_to(
            stroke_width_raw * 3.5 - tangent_of_angle * stroke_width_raw * 2.0,
            stroke_width_raw * 2.0,
        );
        builder.line_to(
            stroke_width_raw * 3.5 - tangent_of_angle * stroke_width_raw * 3.0,
            stroke_width_raw * 3.0,
        );
        builder.line_to(
            width_raw - stroke_width_raw * 2.0 - tangent_of_angle * stroke_width_raw * 3.0,
            stroke_width_raw * 3.0,
        );
        builder.line_to(
            width_raw - stroke_width_raw * 2.0 - tangent_of_angle * stroke_width_raw * 6.0,
            stroke_width_raw * 6.0,
        );
        builder.line_to(
            width_raw - stroke_width_raw * 3.0 - tangent_of_angle * stroke_width_raw * 6.0,
            stroke_width_raw * 6.0,
        );
        builder.line_to(
            width_raw - stroke_width_raw * 3.0 - tangent_of_angle * stroke_width_raw * 4.0,
            stroke_width_raw * 4.0,
        );
        builder.close();
        builder.path_data
    };

    let width = width_raw * scale;
    let height = GLYPH_HEIGHT; // as face.height() * scale
    let box_left = (-tangent_of_angle * GLYPH_HEIGHT).min(0.0);
    let box_width = width + (tangent_of_angle * GLYPH_HEIGHT).abs();

    let symbol = create_symbol_path('\n', box_left, 0.0, box_width, height, &path_data);

    let glyph_info = GlyphInfo {
        feed_width: width,
        symbol,
        x_offset: box_left,
        y_offset: 0.0,
    };
    Some(glyph_info)
}

/// Creates an SVG symbol element for a glyph with the specified parameters.
fn create_symbol_path(
    ch: char,
    box_left: f32,
    box_top: f32,
    box_width: f32,
    box_height: f32,
    path_data: &str,
) -> String {
    let glyph_number = u32::from(ch);
    let box_left = round_float(box_left);
    let box_top = round_float(box_top);
    let box_width = round_float(box_width);
    let box_height = round_float(box_height);
    format!(
        "<symbol \
            id=\"glyph{glyph_number}\" \
            viewBox=\"{box_left} {box_top} {box_width} {box_height}\" \
            width=\"{box_width}\" \
            height=\"{box_height}\" \
        >\
        <path d=\"{path_data}\"/>\
        </symbol>",
    )
}
