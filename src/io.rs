/// A wrapper around a `std::io::Write` that implements `std::fmt::Write` for writing UTF-8 strings.
pub struct Utf8Writer<W: std::io::Write> {
    io_writer: W,
}

impl<W: std::io::Write> Utf8Writer<W> {
    /// Creates a new `Utf8Writer` that wraps the given `std::io::Write`.
    pub fn new(io_writer: W) -> Self {
        Self { io_writer }
    }
}

/// A wrapper around a `std::io::Write` that implements `std::fmt::Write` for writing UTF-8 strings.
impl<W: std::io::Write> std::fmt::Write for Utf8Writer<W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.io_writer
            .write_all(s.as_bytes())
            .map_err(|_| std::fmt::Error)
    }
}
