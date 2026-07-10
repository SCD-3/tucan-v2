mod tiles;
mod hexmap;
mod drawing;

use std::error::Error;
use hexx::Vec2;
use rand::{Rng, rng};
use image::*;


use hexmap::*;
use crate::drawing::{DrawHexMap, ImageConfig};

const ERROR_TIMEOUT_LIMIT: u8 = 20;

const WIDTH: u32 = 2480;
const HEIGHT: u32 = 1748;

const HEX_RADIUS_BIG: f32 = 70.00;
const HEX_RADIUS_SMALL: f32 = 70.00;

const HEXMAP_OFFSET: Vec2 = Vec2 { x: 850.0, y: 874.0 };

fn try_gen<R: Rng>(rng: &mut R, size: MapSize) -> Result<(TileMap_shape, TileMap_templates, TileMap_props), String> {
    let raw = TileMap_rawShapeGen::new(size, rng)?;
    let shape = TileMap_shape::new(raw)?;

    let props = TileMap_props::new(rng, &shape)?;
    let templates = TileMap_templates::new(rng, &shape, &props)?;
    Ok((shape, templates, props))
}

fn main() -> Result<(), Box<dyn Error>> {
    let size = MapSize::Big;

    let mut rng = rng();
    let mut image_shape: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(WIDTH, HEIGHT);
    let mut image_props: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(WIDTH, HEIGHT);
    let mut image_templates: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(WIDTH, HEIGHT);
    let image_config = ImageConfig { 
        height: HEIGHT, 
        width: WIDTH, 
        hex_radius: match_size!(size, HEX_RADIUS_BIG, HEX_RADIUS_SMALL),
        hexmap_offset: HEXMAP_OFFSET
    };
    
    let mut error_counter = 0;
    let (shape, templates, props) = loop {
        if error_counter > ERROR_TIMEOUT_LIMIT {panic!("timeout during generation")}
        let res= try_gen(&mut rng, size);
        if let Ok(value) = res {break value} 
        else {error_counter += 1; println!("{}", res.err().unwrap())}
    };
    
    
    shape.draw(&mut image_shape, image_config);
    image_shape.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_shape.png")?;

    props.draw(&mut image_props, image_config);
    image_props.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_props.png")?;

    templates.draw(&mut image_templates, image_config);
    image_templates.save(r"C:\Users\piotr\Documents\code_projects\rust\tucan-v2\image_templates.png")?;

    Ok(())
}
