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
    let mut img1: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(2480, 1754);
    let mut img2: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(2480, 1754);
    let image_config = ImageConfig { height: img1.height(), width: img1.width(), hex_radius: 60.0 };
    
    loop {
        let raw = TileMap_rawShapeGen::new(MapSize::Big, &mut rng);
        let shape = TileMap_shape::new(raw);

        if shape.has_no_holes() {
            let templates = TileMap_templates::new(&mut rng, &shape)?;
            let props = TileMap_props::new(&mut rng, &shape)?;
            
            shape.draw(&mut img1, image_config);
            img1.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_shape.png")?;

            props.draw(&mut img2, image_config);
            img2.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_props.png")?;

            break;
        }
    }

    Ok(())
}
