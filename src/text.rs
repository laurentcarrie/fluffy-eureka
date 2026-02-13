use font_kit::source::SystemSource;
use ttf_parser::OutlineBuilder;

struct SvgPathBuilder {
    path: String,
    x_offset: f64,
}

impl SvgPathBuilder {
    fn new(x_offset: f64) -> Self {
        Self {
            path: String::new(),
            x_offset,
        }
    }
}

impl OutlineBuilder for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = x as f64 + self.x_offset;
        let y = -(y as f64);
        self.path.push_str(&format!("M {x} {y} "));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = x as f64 + self.x_offset;
        let y = -(y as f64);
        self.path.push_str(&format!("L {x} {y} "));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let x1 = x1 as f64 + self.x_offset;
        let y1 = -(y1 as f64);
        let x = x as f64 + self.x_offset;
        let y = -(y as f64);
        self.path.push_str(&format!("Q {x1} {y1} {x} {y} "));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let x1 = x1 as f64 + self.x_offset;
        let y1 = -(y1 as f64);
        let x2 = x2 as f64 + self.x_offset;
        let y2 = -(y2 as f64);
        let x = x as f64 + self.x_offset;
        let y = -(y as f64);
        self.path
            .push_str(&format!("C {x1} {y1} {x2} {y2} {x} {y} "));
    }

    fn close(&mut self) {
        self.path.push_str("Z ");
    }
}

pub fn svg_path_of_text(text: &str, font_name: &str) -> String {
    let font = SystemSource::new()
        .select_by_postscript_name(font_name)
        .unwrap_or_else(|_| panic!("font not found: {font_name}"))
        .load()
        .unwrap_or_else(|_| panic!("failed to load font: {font_name}"));

    let font_data = font.copy_font_data().expect("failed to copy font data");
    let face = ttf_parser::Face::parse(&font_data, 0).expect("failed to parse font");

    let mut path = String::new();
    let mut x: f64 = 0.0;

    for ch in text.chars() {
        let glyph_id = match face.glyph_index(ch) {
            Some(id) => id,
            None => continue,
        };

        let mut builder = SvgPathBuilder::new(x);
        face.outline_glyph(glyph_id, &mut builder);
        path.push_str(&builder.path);

        if let Some(advance) = face.glyph_hor_advance(glyph_id) {
            x += advance as f64;
        }
    }

    path.trim_end().to_string()
}
