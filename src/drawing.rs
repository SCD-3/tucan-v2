use imageproc::point::Point;
use std::f32::consts::PI;
use hexx::{layout::HexLayout, orientation::HexOrientation, Vec2, storage::HexStore};
use hexx::Hex;
use imageproc::drawing::Canvas;

#[derive(Clone, Copy)]
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub hex_radius: f32,

    pub hexmap_offset: Vec2
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

#[must_use]
pub fn get_pos(pos: Hex, image_config: ImageConfig) -> Vec2 {
    let layout = HexLayout { 
        origin: image_config.hexmap_offset, 
        orientation: HexOrientation::Pointy,
        scale: Vec2::splat(image_config.hex_radius) };
    layout.hex_to_world_pos(pos)
}

pub trait DrawHexMap<T>: Sized + HexStore<T> {

    type ColorSpace;

    fn draw<C: Canvas<Pixel = Self::ColorSpace>>(&self, img: &mut C, image_config: ImageConfig) {
        for (hex, element) in self.iter() {
            self.draw_element(img, hex, element, image_config);
        }
    }

    fn draw_element<C: Canvas<Pixel = Self::ColorSpace>>(&self, img: &mut C, hex: Hex, value: &T, image_config: ImageConfig);


}