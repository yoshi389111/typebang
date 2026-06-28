use anyhow::Context;

/// Font source, either from a file or the system's installed fonts.
pub enum Source {
    /// Represents a font loaded from a file.
    File(std::path::PathBuf),
    /// Represents a font loaded from the system's installed fonts.
    System,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::File(path) => write!(f, "{}", path.display()),
            Source::System => write!(f, "System font"),
        }
    }
}

/// Font filter used to specify the source and name of a font when querying the font database.
pub struct Filter {
    pub source: Source,
    pub name: Option<String>,
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} of {}", name, self.source),
            None => write!(f, "Any font of {}", self.source),
        }
    }
}

/// Loads the font database and runs the provided closure with the font face corresponding to the given filter.
pub fn load_and_run<T>(
    filter: &Filter,
    f: impl for<'a> FnOnce(&ttf_parser::Face<'_>) -> T,
) -> anyhow::Result<T> {
    let db = create_font_db(&filter.source)?;

    let id = match &filter.name {
        Some(font_name) => get_font_id(&db, font_name)
            .ok_or_else(|| anyhow::anyhow!("Font '{}' not found", filter))?,

        None => db
            .faces()
            .next()
            .map(|face| face.id)
            .ok_or_else(|| anyhow::anyhow!("No fonts available in '{}'", filter.source))?,
    };

    db.with_face_data(id, |data, index| {
        let face = ttf_parser::Face::parse(data, index)
            .with_context(|| format!("Failed to parse font data for {}", filter))?;
        Ok(f(&face))
    })
    .ok_or_else(|| anyhow::anyhow!("Font data for {} not available", filter,))?
}

/// Returns the font ID for the given font name, if it exists.
fn get_font_id(db: &fontdb::Database, font_name: &str) -> Option<fontdb::ID> {
    if let Some(id) = db
        .faces()
        .find(|face| face.post_script_name == font_name)
        .map(|face| face.id)
    {
        return Some(id);
    }

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

/// Retrieves the list of available font families from the system's font database.
pub fn get_families() -> anyhow::Result<Vec<String>> {
    let db = create_font_db(&Source::System)?;
    let families = db
        .faces()
        .flat_map(|face| &face.families)
        .map(|(name, _)| name)
        .collect::<std::collections::BTreeSet<_>>();
    Ok(families.into_iter().cloned().collect())
}

/// Retrieves the list of font faces (PostScript names) that match the given filter.
pub fn get_font_faces(filter: &Filter) -> anyhow::Result<Vec<String>> {
    let db = create_font_db(&filter.source)?;
    let post_script_names = db
        .faces()
        .filter(|&face| is_target_family(&filter.name, face))
        .map(|face| face.post_script_name.clone())
        .collect::<Vec<_>>();
    Ok(post_script_names)
}

/// Creates a font database based on the specified source.
fn create_font_db(source: &Source) -> anyhow::Result<fontdb::Database> {
    let mut db = fontdb::Database::new();
    match source {
        Source::File(path) => db
            .load_font_file(path)
            .with_context(|| format!("Failed to load font file '{:?}'", path))?,
        Source::System => db.load_system_fonts(),
    }
    Ok(db)
}

/// Checks if the given face info matches the target family.
fn is_target_family(family: &Option<String>, face_info: &fontdb::FaceInfo) -> bool {
    match family {
        Some(name) => face_info
            .families
            .iter()
            .any(|(family_name, _)| family_name == name),
        None => true,
    }
}
