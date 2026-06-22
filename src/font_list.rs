pub fn print_font_families(out: &mut impl std::fmt::Write) -> anyhow::Result<()> {
    let db = {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db
    };

    let families = db
        .faces()
        .flat_map(|face| &face.families)
        .map(|(name, _)| name)
        .collect::<std::collections::BTreeSet<_>>();

    writeln!(out, "Generic families:")?;
    writeln!(out, "  sans-serif")?;
    writeln!(out, "  serif")?;
    writeln!(out, "  monospace")?;
    writeln!(out)?;

    writeln!(out, "Installed font families:")?;

    if families.is_empty() {
        writeln!(out, "  No fonts found.")?;
    } else {
        for family in families {
            writeln!(out, "  {family}")?;
        }
    }
    Ok(())
}
