mod tiles;
mod hexmap;
mod drawing;

use std::error::Error;

use hexmap::*;
use rand::rng;
use image::*;

use crate::drawing::{DrawHexMap, ImageConfig};

fn main() -> Result<(), Box<dyn Error>>{
    let mut rng = rng();
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(2000, 1000);
    let image_config = ImageConfig { height: img.height(), width: img.width(), hex_radius: 15.0 };
    
    loop {
        let raw = TileMap_rawShapeGen::new(MapSize::Big, &mut rng);
        let shape = TileMap_shape::new(raw);
        let templates = TileMap_templates::new(&mut rng, &shape);

        if shape.has_no_holes() {
            templates.draw(&mut img, image_config);
            img.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image.png")?;
            break;
        }
    }
    Ok(())
}
