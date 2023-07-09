use font_kit::loaders::freetype::Font;
use raqote::{DrawOptions, DrawTarget, Point, Source};

pub trait DrawTargetExt {
  fn draw_text_fix(
    &mut self,
    font: &Font,
    point_size: f32,
    text: &str,
    start: Point,
    src: &Source,
    options: &DrawOptions,
  );
}
struct Vector2 {
  x: f32,
  y: f32,
}

impl DrawTargetExt for DrawTarget {
  fn draw_text_fix(
    &mut self,
    font: &Font,
    point_size: f32,
    text: &str,
    start: Point,
    src: &Source,
    options: &DrawOptions,
  ) {
    let mut pos = Vector2 {
      x: start.x,
      y: start.y,
    };
    let mut ids = Vec::new();
    let mut positions = Vec::new();
    for c in text.chars() {
      let id = font.glyph_for_char(c).unwrap();
      ids.push(id);
      positions.push(Vector2 { x: pos.x, y: pos.y });
      pos.x += font.advance(id).unwrap().x() * point_size / 24. / 96. * 2.2;
    }
    let positions = positions
      .into_iter()
      .map(|p| Point::new(p.x - (pos.x - start.x) * 0.5, p.y))
      .collect::<Vec<_>>();
    self.draw_glyphs(font, point_size, &ids, &positions, src, options);
  }
}
