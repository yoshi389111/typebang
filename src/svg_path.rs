use crate::number::round_float;

/// Represents a builder for constructing SVG path data from font outlines.
pub struct SvgPathBuilder {
    scale: f32,
    ascender: f32,
    start_x: String,
    start_y: String,
    current_x: String,
    current_y: String,
    pub path_data: String,
}

impl SvgPathBuilder {
    pub fn new(scale: f32, ascender: f32) -> Self {
        SvgPathBuilder {
            scale,
            ascender,
            start_x: "0".to_string(),
            start_y: "0".to_string(),
            current_x: "0".to_string(),
            current_y: "0".to_string(),
            path_data: String::new(),
        }
    }

    fn convert_x(&self, x: f32) -> String {
        round_float(x * self.scale)
    }

    fn convert_y(&self, y: f32) -> String {
        round_float((self.ascender - y) * self.scale)
    }
}

impl ttf_parser::OutlineBuilder for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = self.convert_x(x);
        let y = self.convert_y(y);
        self.path_data += &format!("M{x} {y}");
        self.start_x = x.clone();
        self.start_y = y.clone();
        self.current_x = x;
        self.current_y = y;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = self.convert_x(x);
        let y = self.convert_y(y);
        if self.current_x == x && self.current_y == y {
            // Skip if the line is degenerate (no movement)
        } else if self.current_x == x {
            self.path_data += &format!("V{y}");
        } else if self.current_y == y {
            self.path_data += &format!("H{x}");
        } else {
            self.path_data += &format!("L{x} {y}");
        }
        self.current_x = x;
        self.current_y = y;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let x1 = self.convert_x(x1);
        let y1 = self.convert_y(y1);
        let x = self.convert_x(x);
        let y = self.convert_y(y);
        self.path_data += &format!("Q{x1} {y1} {x} {y}");
        self.current_x = x;
        self.current_y = y;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let x1 = self.convert_x(x1);
        let y1 = self.convert_y(y1);
        let x2 = self.convert_x(x2);
        let y2 = self.convert_y(y2);
        let x = self.convert_x(x);
        let y = self.convert_y(y);
        self.path_data += &format!("C{x1} {y1} {x2} {y2} {x} {y}");
        self.current_x = x;
        self.current_y = y;
    }

    fn close(&mut self) {
        self.current_x = self.start_x.clone();
        self.current_y = self.start_y.clone();
        self.path_data += "Z";
    }
}
