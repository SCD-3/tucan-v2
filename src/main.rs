mod tiles;
mod hexmap;
mod drawing;

use std::error::Error;

use rand::{Rng, rng};
use hexmap::*;
use image::*;

use crate::drawing::{DrawHexMap, ImageConfig};

const WIDTH: u32 = 2480;
const HEIGHT: u32 = 1754;

fn try_gen<R: Rng>(rng: &mut R) -> Result<(TileMap_shape, TileMap_templates, TileMap_props), String> {
    let raw = TileMap_rawShapeGen::new(MapSize::Big, rng)?;
    let shape = TileMap_shape::new(raw)?;

    let props = TileMap_props::new(rng, &shape)?;
    let templates = TileMap_templates::new(rng, &shape, &props)?;
    Ok((shape, templates, props))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = rng();
    let mut img1: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(WIDTH, HEIGHT);
    let mut img2: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(WIDTH, HEIGHT);
    let mut img3: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(WIDTH, HEIGHT);
    let image_config = ImageConfig { height: HEIGHT, width: WIDTH, hex_radius: 60.0 };
    
    let (shape, templates, props) = loop {
        let res= try_gen(&mut rng);
        if let Ok(value) = res {break value} 
        else {println!("{}", res.err().unwrap())}
    };
    
    
    shape.draw(&mut img1, image_config);
    img1.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_shape.png")?;

    props.draw(&mut img2, image_config);
    img2.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_props.png")?;

    templates.draw(&mut img3, image_config);
    img3.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_templates.png")?;

    Ok(())
}
