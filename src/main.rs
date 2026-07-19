#[macro_use]
mod tiles;
mod hexmap;
mod drawing;

use std::error::Error;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::sync::Mutex;

use hexx::Vec2;
use rand::{Rng, SeedableRng, rng, rngs::StdRng};

use hexmap::{
    TileMap_rawShapeGen,
    TileMap_shape,
    TileMap_props,
    TileMap_templates,
    TileMap,
    MapSize,
};
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

fn render_image<R: Rng>(rng: &mut R, size: MapSize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut image = image::open(match_size!(size, r"src\img\background_big.png", r"src\img\background_small.png"))?
        .into_rgba8();

    let image_config = ImageConfig {
        height: HEIGHT,
        width: WIDTH,
        radius: match_size!(size, HEX_RADIUS_BIG, HEX_RADIUS_SMALL),
        hexmap_offset: HEXMAP_OFFSET,
    };

    let mut error_counter = 0;
    let (_, templates, props) = loop {
        if error_counter > ERROR_TIMEOUT_LIMIT {
            panic!("timeout during generation");
        }
        let res = try_gen(rng, size);
        if let Ok(value) = res {
            break value;
        } else {
            error_counter += 1;
            eprintln!("{}", res.err().unwrap());
        }
    };

    let map = TileMap::new(templates, props)?;
    map.draw(&mut image, image_config);

    // Stack 2 copies vertically to create A4 from A5
    let mut a4_image = image::ImageBuffer::new(WIDTH as u32, (HEIGHT * 2) as u32);
    for pixel in image.enumerate_pixels() {
        a4_image.put_pixel(pixel.0, pixel.1, pixel.2.clone());
        a4_image.put_pixel(pixel.0, pixel.1 + HEIGHT, pixel.2.clone());
    }

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    a4_image.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(buf)
}

fn render_display_image<R: Rng>(rng: &mut R, size: MapSize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut image = image::open(match_size!(size, r"src\img\background_big.png", r"src\img\background_small.png"))?
        .into_rgba8();

    let image_config = ImageConfig {
        height: HEIGHT,
        width: WIDTH,
        radius: match_size!(size, HEX_RADIUS_BIG, HEX_RADIUS_SMALL),
        hexmap_offset: HEXMAP_OFFSET,
    };

    let mut error_counter = 0;
    let (_, templates, props) = loop {
        if error_counter > ERROR_TIMEOUT_LIMIT {
            panic!("timeout during generation");
        }
        let res = try_gen(rng, size);
        if let Ok(value) = res {
            break value;
        } else {
            error_counter += 1;
            eprintln!("{}", res.err().unwrap());
        }
    };

    let map = TileMap::new(templates, props)?;
    map.draw(&mut image, image_config);

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    image.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(buf)
}

fn render_print_image(image_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let image = image::load_from_memory(image_bytes)?.into_rgba8();

    let mut print_image = image::ImageBuffer::new(WIDTH, HEIGHT * 2);
    for (x, y, pixel) in image.enumerate_pixels() {
        print_image.put_pixel(x, y, *pixel);
        print_image.put_pixel(x, y + HEIGHT, *pixel);
    }

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    print_image.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(buf)
}

fn parse_seed(query: Option<&str>) -> Option<u64> {
    query.and_then(|query| {
        query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            if key == "seed" {
                let value = value.trim();
                if value.is_empty() {
                    None
                } else {
                    u64::from_str_radix(value, 16).ok()
                }
            } else {
                None
            }
        })
    })
}

fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

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

fn handle_client(mut stream: TcpStream, image_data: &Mutex<Vec<u8>>, frontend_html: &str) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let size = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    let request_line = request.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    let (path, query) = path.split_once('?').unwrap_or((path, ""));

    if method == "GET" && (path == "/" || path == "/index.html") {
        write_response(&mut stream, "200 OK", "text/html; charset=utf-8", frontend_html.as_bytes())?;
    } else if method == "GET" && (path == "/image" || path == "/image/") {
        let image_bytes = image_data.lock().unwrap().clone();
        write_response(&mut stream, "200 OK", "image/png", image_bytes.as_slice())?;
    } else if method == "GET" && (path == "/print-image" || path == "/print-image/") {
        let image_bytes = image_data.lock().unwrap().clone();
        let print_bytes = match render_print_image(&image_bytes) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("Print image generation failed: {}", err);
                let body = b"Internal Server Error";
                write_response(&mut stream, "500 Internal Server Error", "text/plain; charset=utf-8", body)?;
                return Ok(());
            }
        };

        write_response(&mut stream, "200 OK", "image/png", print_bytes.as_slice())?;
    } else if method == "GET" && path == "/generate" {
        let seed = parse_seed(Some(query));
        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_rng(&mut rng()),
        };

        let image_bytes = match render_display_image(&mut rng, MapSize::Big) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("Generation failed: {}", err);
                let body = b"Internal Server Error";
                write_response(&mut stream, "500 Internal Server Error", "text/plain; charset=utf-8", body)?;
                return Ok(());
            }
        };

        {
            let mut current_image = image_data.lock().unwrap();
            *current_image = image_bytes;
        }

        write_response(&mut stream, "200 OK", "text/plain; charset=utf-8", b"OK")?;
    } else {
        let body = b"Not Found";
        write_response(&mut stream, "404 Not Found", "text/plain; charset=utf-8", body)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let size = MapSize::Big;
    let frontend_html = fs::read_to_string("src/frontend.html")?;

    let image_data = Mutex::new(Vec::<u8>::new());

    let mut rng = rng();
    let initial_image = render_display_image(&mut rng, size)?;
    {
        let mut current_image = image_data.lock().unwrap();
        *current_image = initial_image;
    }

    let listener = TcpListener::bind("127.0.0.1:5500")?;
    if let Err(err) = open_browser("http://127.0.0.1:5500/") {
        eprintln!("Failed to open browser: {}", err);
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_client(stream, &image_data, &frontend_html) {
                    eprintln!("Client error: {}", e);
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
