use crate::number::round_float;
use crate::svg_path::SvgPathBuilder;
use ttf_parser::OutlineBuilder;

pub const GLYPH_HEIGHT: f32 = 100.0;

pub struct GlyphInfo {
    pub x_offset: f32,
    pub y_offset: f32,
    pub width: f32,
    pub symbol: String,
}

fn create_normal_glyph_info(face: &ttf_parser::Face, ch: char) -> Option<GlyphInfo> {
    let glyph_id = face.glyph_index(ch)?;
    let height = GLYPH_HEIGHT;
    let scale = height / face.height() as f32;
    let ascender = face.ascender() as f32;

    let path_data = {
        let mut builder = SvgPathBuilder::new(scale, ascender);
        face.outline_glyph(glyph_id, &mut builder);
        builder.path_data
    };

    let width = face.glyph_hor_advance(glyph_id)? as f32 * scale;
    let bbox = face.glyph_bounding_box(glyph_id)?;
    let x_offset = bbox.x_min as f32 * scale;
    let y_offset = (ascender - bbox.y_max as f32) * scale;

    let box_left = round_float(x_offset);
    let box_top = round_float(y_offset);
    let box_width = round_float((bbox.x_max as f32 - bbox.x_min as f32) * scale);
    let box_height = round_float((bbox.y_max as f32 - bbox.y_min as f32) * scale);

    let glyph_number = u32::from(ch);

    let symbol = format!(
        "<symbol \
            id=\"glyph{glyph_number}\" \
            viewBox=\"{box_left} {box_top} {box_width} {box_height}\" \
            width=\"{box_width}\" \
            height=\"{box_height}\" \
        >\
        <path d=\"{path_data}\"/>\
        </symbol>",
    );

    let glyph_info = GlyphInfo {
        width,
        symbol,
        x_offset,
        y_offset,
    };
    Some(glyph_info)
}

fn create_space_glyph_info(face: &ttf_parser::Face) -> Option<GlyphInfo> {
    let glyph_id = face.glyph_index(' ')?;
    let height = GLYPH_HEIGHT;
    let scale = height / face.height() as f32;
    let width_advance = face.glyph_hor_advance(glyph_id)? as f32;
    let ascender = face.ascender() as f32;

    let path_data = {
        let stroke_width_horizontal = width_advance * 0.125;
        let stroke_width_vertical = ascender * 0.125;
        let stroke_width = stroke_width_horizontal.min(stroke_width_vertical);
        let mut builder = SvgPathBuilder::new(scale, ascender);
        builder.move_to(stroke_width, 0.0);
        builder.line_to(width_advance - stroke_width, 0.0);
        builder.line_to(width_advance - stroke_width, stroke_width * 2.0);
        builder.line_to(width_advance - stroke_width * 2.0, stroke_width * 2.0);
        builder.line_to(width_advance - stroke_width * 2.0, stroke_width);
        builder.line_to(stroke_width * 2.0, stroke_width);
        builder.line_to(stroke_width * 2.0, stroke_width * 2.0);
        builder.line_to(stroke_width, stroke_width * 2.0);
        builder.close();
        builder.path_data
    };

    let glyph_number = u32::from(' ');
    let width = width_advance * scale;

    let symbol = format!(
        "<symbol id=\"glyph{}\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\"><path d=\"{}\" /></symbol>",
        glyph_number,
        round_float(width),
        round_float(height),
        round_float(width),
        round_float(height),
        path_data
    );

    let glyph_info = GlyphInfo {
        width,
        symbol,
        x_offset: 0.0,
        y_offset: 0.0,
    };
    Some(glyph_info)
}

fn create_enter_glyph_info(face: &ttf_parser::Face) -> Option<GlyphInfo> {
    let glyph_id = face.glyph_index('n')?;
    let height = GLYPH_HEIGHT;
    let scale = height / face.height() as f32;
    let width_advance = face.glyph_hor_advance(glyph_id)? as f32;

    let path_data = {
        let ascender = face.ascender() as f32;
        let box_height = ascender * 0.7;
        let stroke_width_horizontal = width_advance * 0.125;
        let stroke_width_vertical = box_height * 0.125;
        let stroke_width = stroke_width_horizontal.min(stroke_width_vertical);
        let mut builder = SvgPathBuilder::new(scale, ascender);
        builder.move_to(0.0, 0.0);
        builder.line_to(width_advance, 0.0);
        builder.line_to(width_advance, box_height);
        builder.line_to(0.0, box_height);
        builder.close();
        builder.move_to(stroke_width, stroke_width);
        builder.line_to(stroke_width, box_height - stroke_width);
        builder.line_to(width_advance - stroke_width, box_height - stroke_width);
        builder.line_to(width_advance - stroke_width, stroke_width);
        builder.close();
        builder.move_to(stroke_width * 3.5, stroke_width * 4.0);
        builder.line_to(stroke_width * 3.5, stroke_width * 5.0);
        builder.line_to(stroke_width * 2.0, stroke_width * 3.5);
        builder.line_to(stroke_width * 3.5, stroke_width * 2.0);
        builder.line_to(stroke_width * 3.5, stroke_width * 3.0);
        builder.line_to(width_advance - stroke_width * 2.0, stroke_width * 3.0);
        builder.line_to(width_advance - stroke_width * 2.0, stroke_width * 6.0);
        builder.line_to(width_advance - stroke_width * 3.0, stroke_width * 6.0);
        builder.line_to(width_advance - stroke_width * 3.0, stroke_width * 4.0);
        builder.close();
        builder.path_data
    };

    let width = width_advance * scale;
    let symbol = format!(
        "<symbol id=\"glyph{}\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\"><path d=\"{}\" /></symbol>",
        u32::from('\n'),
        round_float(width),
        round_float(height),
        round_float(width),
        round_float(height),
        path_data
    );

    let glyph_info = GlyphInfo {
        width,
        symbol,
        x_offset: 0.0,
        y_offset: 0.0,
    };
    Some(glyph_info)
}

pub fn create_glyph_info(face: &ttf_parser::Face, ch: char) -> Option<GlyphInfo> {
    match ch {
        ' ' => create_space_glyph_info(face),
        '\n' => create_enter_glyph_info(face),
        _ => create_normal_glyph_info(face, ch),
    }
}
