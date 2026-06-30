use imageproc::point::Point;
use std::f32::consts::PI;
use hexx::{layout::HexLayout, Vec2, orientation::HexOrientation};
use hexx::Hex;
use imageproc::drawing::Canvas;

#[derive(Clone, Copy)]
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub hex_radius: f32
}

#[must_use]
pub fn get_hex_points(center: Vec2, radius: f32) -> Vec<Point<i32>> {
    (0..6)
        .map(|i| {
            let angle = PI / 3.0 * i as f32;
            Point::new(
                (center.x + radius * f32::cos(angle+PI/6.0) ).round() as i32,
                (center.y + radius * f32::sin(angle+PI/6.0) ).round() as i32,
            )
        })
        .collect()
}

pub trait DrawHexMap {

    type ColorSpace;

    fn draw<C: Canvas<Pixel = Self::ColorSpace>>(self, img: &mut C, image_config: ImageConfig);

    #[must_use]
    fn get_pos(pos: Hex, image_config: ImageConfig) -> Vec2 {
        let layout = HexLayout { 
            origin: Vec2 { x: image_config.width as f32/2.0, y: image_config.height as f32 / 2.0 }, 
            orientation: HexOrientation::Pointy,
            scale: Vec2::splat(image_config.hex_radius) };
        layout.hex_to_world_pos(pos)
    }
}