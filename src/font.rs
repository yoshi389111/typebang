use anyhow::Context;

pub fn get_font_id(db: &fontdb::Database, font_name: &str) -> Option<fontdb::ID> {
    let families = match font_name.to_lowercase().as_str() {
        "sans-serif" => vec![
            fontdb::Family::SansSerif,
            fontdb::Family::Name("Noto Sans"),
            fontdb::Family::Name("DejaVu Sans"),
            fontdb::Family::Name("Arial"),
            fontdb::Family::Name("Segoe UI"),
        ],
        "serif" => vec![
            fontdb::Family::Serif,
            fontdb::Family::Name("Noto Serif"),
            fontdb::Family::Name("DejaVu Serif"),
            fontdb::Family::Name("Times New Roman"),
        ],
        "monospace" => vec![
            fontdb::Family::Monospace,
            fontdb::Family::Name("Noto Sans Mono"),
            fontdb::Family::Name("DejaVu Sans Mono"),
            fontdb::Family::Name("Consolas"),
            fontdb::Family::Name("Courier New"),
            fontdb::Family::Name("Lucida Console"),
        ],
        _ => vec![fontdb::Family::Name(font_name)],
    };

    let query = fontdb::Query {
        families: &families,
        ..Default::default()
    };

    db.query(&query)
}

/// This module provides functionality for loading fonts and executing operations on them.
pub fn load_and_run<T>(
    font_name: &str,
    f: impl for<'a> FnOnce(&ttf_parser::Face<'_>) -> T,
) -> anyhow::Result<T> {
    // If the font name looks like a file path, try to load it directly as a font file.
    if font_name.contains('/') || font_name.contains('\\') {
        let font_data = std::fs::read(font_name)
            .with_context(|| format!("Failed to read font file for '{}'", font_name))?;
        let face = ttf_parser::Face::parse(&font_data, 0)
            .with_context(|| format!("Failed to parse font file for '{}'", font_name))?;
        return Ok(f(&face));
    }

    let db = {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db
    };

    let id = get_font_id(&db, font_name)
        .ok_or_else(|| anyhow::anyhow!("Font '{}' not found", font_name))?;

    db.with_face_data(id, |data, index| {
        let face = ttf_parser::Face::parse(data, index)
            .with_context(|| format!("Failed to parse font data for '{}'", font_name))?;
        Ok(f(&face))
    })
    .ok_or_else(|| anyhow::anyhow!("Failed to parse font data for '{}'", font_name))?
}
