#[macro_use]
mod tiles;
mod hexmap;
mod drawing;

use std::error::Error;
use std::net;
use std::process::Command;

use hexx::Vec2;
use image::{ImageBuffer, Rgba};
use rand::prelude::*;

use hexmap::{
    TileMap_rawShapeGen,
    TileMap_shape,
    TileMap_props,
    TileMap_templates,
    TileMap,
    MapSize,
};
use crate::drawing::{DrawHexMap, ImageConfig};

static GUI_HTML: &[u8] = include_bytes!(r"..\vol\templates\gui.html");
static BACKGROUND_BIG: &[u8] = std::include_bytes!(r"..\vol\assets\background_big.png");
static BACKGROUND_SMALL: &[u8] = std::include_bytes!(r"..\vol\assets\background_small.png");


const GENERATION_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);

const WIDTH: u32 = 2480;
const HEIGHT: u32 = 1748;

const HEX_RADIUS_BIG: f32 = 70.00;
const HEX_RADIUS_SMALL: f32 = 70.00;

const HEXMAP_OFFSET: Vec2 = Vec2 { x: 850.0, y: 874.0 };


/// Attempts to generate a tile map shape, templates, and props using the provided random number generator and map size.
/// 
/// # Arguments
/// * `rng` - A mutable reference to the random number generator.
/// * `size` - The size of the tile map.
/// 
/// # Returns
/// A result containing the generated shape, templates, and props, or an error message.
fn try_gen<R: Rng>(rng: &mut R, size: MapSize) -> Result<TileMap, String> {
    let raw = TileMap_rawShapeGen::new(size, rng)?;
    let shape = TileMap_shape::new(raw)?;

    let props = TileMap_props::new(rng, &shape)?;
    let templates = TileMap_templates::new(rng, &shape, &props)?;
    TileMap::new(templates, props)
}

fn render(map: TileMap, image_config: ImageConfig) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let mut image = image::load_from_memory(
        match_size!(map.size(), BACKGROUND_BIG, BACKGROUND_SMALL)
        )?
        .into_rgba8();

    map.draw(&mut image, image_config);

    return Ok(image);
}


/// Renders a print image by duplicating the provided image vertically.
/// 
/// # Arguments
/// * `image_bytes` - A slice of bytes representing the original image.
/// 
/// # Returns
/// A result containing the rendered print image as a vector of bytes, or an error message.
fn render_print(map: TileMap, image_config: ImageConfig) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let image = render(map, image_config)?;
    let mut new_image = ImageBuffer::new(image.width(), image.height());
    image::imageops::overlay(&mut new_image, &image, 0, 0);
    image::imageops::overlay(&mut new_image, &image, 0, image.height() as i64);

    Ok(new_image)
}

/// Opens the default web browser with the specified URL.
/// 
/// # Arguments
/// * `url` - A string slice representing the URL to open.
/// 
/// # Returns
/// A result indicating success or failure of the operation.
fn open_browser(url: &str) -> std::io::Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
    } else {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
    }
}

fn main() -> Result<(), Box<dyn Error>> {


    Ok(())
}
