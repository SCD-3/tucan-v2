#[macro_use]
mod tiles;
mod hexmap;
mod drawing;
mod network_handler;

use std::error::Error;
use std::{io::prelude::*, net::TcpListener};
use std::process::Command;

use hexx::{Vec2, Hex};
use image::{ImageBuffer, Rgba, Pixel};
use rand::prelude::*;
use rand::random_range;
use ab_glyph::FontArc;
use imageproc::drawing::{Canvas, draw_polygon_mut, draw_filled_circle_mut, draw_text_mut};

use crate::hexmap::{
    TileMap_rawShapeGen,
    TileMap_shape,
    TileMap_props,
    TileMap_templates,
    TileMap,
    MapSize,
};
use crate::drawing::*;
use crate::network_handler::*;
use crate::tiles::*;

static GUI_HTML: &str = include_str!(r"..\vol\templates\gui.html" );
static GUI_JS: &str   = include_str!(r"..\vol\templates\script.js");
static GUI_CSS: &str  = include_str!(r"..\vol\templates\style.css");

static FAVICON: &[u8] = include_bytes!(r"..\vol\templates\favicon.ico");

static BACKGROUND_BIG: &[u8] = include_bytes!(r"..\vol\assets\background_big.png");
static BACKGROUND_SMALL: &[u8] = include_bytes!(r"..\vol\assets\background_small.png");

const FONT_SCALE: f32 = 100.0;
const FONT_OFFSET: Vec2 = Vec2::new(-5.0, -50.0);
static FONT: &[u8] = include_bytes!(r"..\vol\assets\Comic Sans MS.ttf");

const PROP_RADIUS_MULTI: f32 = 0.60;
const IMAGE_PROP_OFFSET_X: u32 = 40;
const IMAGE_PROP_OFFSET_Y: u32 = 35;

static OK: &str = "HTTP/1.1 200 OK";

// const WIDTH: u32 = 2480;
// const HEIGHT: u32 = 1748;

const HEX_RADIUS_BIG: f32 = 70.00;
const HEX_RADIUS_SMALL: f32 = 70.00;

const HEXMAP_OFFSET: Vec2 = Vec2 { x: 850.0, y: 874.0 };


impl DrawHexMap<Tile> for TileMap {
    type ColorSpace = Rgba<u8>;

    fn draw_element<C: Canvas<Pixel = Self::ColorSpace>>(&self, img: &mut C, hex: Hex, value: &Tile, image_config: ImageConfig) {
        let pos = get_pos(hex, image_config);
        if let Tile { template: Some(template), prop: prop_option } = *value {
            let points = get_hex_points(pos, image_config.radius);
            draw_polygon_mut(img, &points, template.color());

            if let PropOption::Some(prop) = prop_option {
                if let Prop::Village(n) = prop {
                    draw_filled_circle_mut(img, (pos.x as i32, pos.y as i32), (image_config.radius * PROP_RADIUS_MULTI) as i32, rgba!(210, 105, 30));
                    
                    let font = FontArc::try_from_slice(FONT).expect("invalid font");
                    let pos = pos + FONT_OFFSET;
                    draw_text_mut(img, rgba!(0, 0, 0), pos.x as i32, pos.y as i32, FONT_SCALE, &font, &n.to_string());
                }
                else {
                    let prop_image = prop.get_image();
                    for (x, y, pixel) in prop_image.enumerate_pixels() {
                        if pixel.alpha() > 0 {
                            img.draw_pixel(
                                pos.x as u32 + x - IMAGE_PROP_OFFSET_X, 
                                pos.y as u32 + y - IMAGE_PROP_OFFSET_Y, 
                                *pixel);
                        }
                    }
                }
            }
        }
    }
}

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

fn generate<R: Rng>(rng: &mut R, size: MapSize) -> TileMap {
    loop {
        let res = try_gen(rng, size);
        if res.is_err() {
            eprintln!("{}", res.err().unwrap())
        }
        else {
            return res.unwrap();
        }
    }
}

fn parse_seed(source: &str) -> u64 {
    u64::from_str_radix(source, 16).expect(&format!("failed to parse string {source}"))
}

fn render(map: TileMap, image_config: ImageConfig) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let mut image = image::load_from_memory(
        match_size!(map.size(), BACKGROUND_BIG, BACKGROUND_SMALL)
        )?
        .into_rgba8();

    map.draw(&mut image, image_config);

    Ok(image)
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
    let mut new_image = ImageBuffer::new(image.width(), image.height()*2);
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
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to server");
    let full_addr = &("http://".to_string() + &listener.local_addr().unwrap().to_string());
    open_browser(full_addr)?;
    println!("Starting server at {full_addr}");
    println!("DO NOT SAVE PAGE IN BOOKMARKS. IT CAN AND WILL CHANGE AT NEXT LAUNCH OF THE PROGRAM\n");

    let size = MapSize::Big;
    let image_config = ImageConfig {
        // height: HEIGHT,
        // width: WIDTH,
        radius: match_size!(size, HEX_RADIUS_BIG, HEX_RADIUS_SMALL),
        hexmap_offset: HEXMAP_OFFSET,
    };
    let mut seed = None;
    let mut image = None;

    for stream in listener.incoming() {
        let mut stream = stream.expect("failed to read stream");
        let mut buffer = [0u8; 1024];
        stream.read(&mut buffer)?;

        let request = Request::parse(&buffer).expect("failed to parse request");

        match (request.method, request.path.as_str()) {
            (Method::GET, "/") => {
                println!("getting index");
                let response = format!("{OK}\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n", GUI_HTML.len());
                stream.write_all(response.as_bytes())?;
                stream.write_all(GUI_HTML.as_bytes())?;
            }

            (Method::GET, "/style.css") => {
                println!("getting stylesheet");
                let response = format!("{OK}\r\nContent-Length: {}\r\nContent-Type: text/css\r\n\r\n", GUI_CSS.len());
                stream.write_all(response.as_bytes())?;
                stream.write_all(GUI_CSS.as_bytes())?;
            }

            (Method::GET, "/script.js") => {
                println!("getting script");
                let response = format!("{OK}\r\nContent-Length: {}\r\nContent-Type: application/javascript\r\n\r\n", GUI_JS.len());
                stream.write_all(response.as_bytes())?;
                stream.write_all(GUI_JS.as_bytes())?;
            }

            
            (Method::GET, "/favicon.ico") => {
                println!("getting icon");
                let response = format!("{OK}\r\nContent-Length: {}\r\nContent-Type: image/x-icon\r\n\r\n", FAVICON.len());
                stream.write_all(response.as_bytes())?;
                stream.write_all(FAVICON)?;
            }

            (Method::POST, "/generate") => {
                println!("posting generate order");
                let response = OK;
                let body = request.body.trim_matches('\0');
                stream.write_all(response.as_bytes())?;
                
                if !body.is_empty() {
                    println!("seed given, using {body}");
                    seed = Some(parse_seed(body));
                }
                else {
                    println!("no seed given, using random");
                    seed = Some(random_range(0..u64::MAX));
                }
                
                let mut rng = StdRng::seed_from_u64(seed.unwrap());
                image = Some(generate(&mut rng, MapSize::Big));
            }

            (Method::GET, "/getSeed") => {
                println!("getting seed");

                if seed.is_some() {
                    let seed = format!("{:X}", seed.unwrap());
                    let response = format!("{OK}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n", seed.len());
                    stream.write_all(response.as_bytes())?;
                    stream.write_all(seed.as_bytes())?;
                }
                else {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 14\r\n\r\n")?;
                    stream.write_all(b"Seed not found")?;
                };
            }

            (Method::GET, "/getImage") => {
                println!("getting image");

                if let Some(image) = image.clone() {
                    let image = render(image, image_config)?;
                    let mut png_bytes = Vec::new();

                    image::DynamicImage::ImageRgba8(image)
                        .write_to(
                            &mut std::io::Cursor::new(&mut png_bytes),
                            image::ImageFormat::Png,
                        )?;

                    let response = format!(
                        "{OK}\r\nContent-Length: {}\r\nContent-Type: image/png\r\n\r\n",
                        png_bytes.len()
                    );

                    stream.write_all(response.as_bytes())?;
                    stream.write_all(&png_bytes)?;
                }
                else {
                    stream.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 15\r\n\r\nImage not found"
                    )?;
                };
            }

            (Method::GET, "/getPrintImage") => {
                println!("getting print image");

                if let Some(image) = image.clone() {
                    let image = render_print(image, image_config)?;
                    let mut png_bytes = Vec::new();

                    image::DynamicImage::ImageRgba8(image)
                        .write_to(
                            &mut std::io::Cursor::new(&mut png_bytes),
                            image::ImageFormat::Png,
                        )?;

                    let response = format!(
                        "{OK}\r\nContent-Length: {}\r\nContent-Type: image/png\r\n\r\n",
                        png_bytes.len()
                    );

                    stream.write_all(response.as_bytes())?;
                    stream.write_all(&png_bytes)?;
                }
                else {
                    stream.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 15\r\n\r\nImage not found"
                    )?;
                };
            }

            _ => {
                eprintln!("request not found: {request:?}");
                stream.write_all(b"HTTP/1.1 404 Not found")?;
            }
        };
    };

    Ok(())
}