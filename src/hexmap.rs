use std::ops::{Index, IndexMut};

use hexx::{Hex, layout, storage::{HexStore, HexagonalMap}, layout::HexLayout, Vec2, orientation::HexOrientation};
use image::DynamicImage;
use imageproc::drawing::Canvas;
use crate::tiles::*;

pub const SMALL_MAP_SIZE: u8 = 73;
pub const BIG_MAP_SIZE:   u8 = 104;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum MapSize {
    Small = 73,
    Big   = 104
}

pub enum RawTileState {
    FreeToTake,
    Taken,
    Unknown
}

pub struct TileMap_rawShapeGen {
    map: HexagonalMap<RawTileState>,
    size: MapSize
}
impl TileMap_rawShapeGen {
    pub fn iter(&self) -> impl Iterator {
        self.map.iter()
    }

    #[inline(always)]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }
}

impl Index<Hex> for TileMap_rawShapeGen {
    type Output = RawTileState;
    fn index(&self, index: Hex) -> &Self::Output {
        &self.map[index]
    }
}
impl IndexMut<Hex> for TileMap_rawShapeGen {
    fn index_mut(&mut self, index: Hex) -> &mut Self::Output {
        &mut self.map[index]
    }
}



pub struct TileMap_shape {
    map: HexagonalMap<bool>,
    size: MapSize
}
impl TileMap_shape {
    
    pub fn new(from: TileMap_rawShapeGen) -> Self {
        let map = HexagonalMap::new(Hex::ZERO, from.size_u32(), |h| matches!(from[h], RawTileState::Taken));
        Self { map, size: from.size }
    }

    pub fn iter(&self) -> impl Iterator {
        self.map.iter()
    }

    #[inline(always)]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }
}

impl Index<Hex> for TileMap_shape {
    type Output = bool;
    fn index(&self, index: Hex) -> &Self::Output {
        &self.map[index]
    }
}
impl IndexMut<Hex> for TileMap_shape {
    fn index_mut(&mut self, index: Hex) -> &mut Self::Output {
        &mut self.map[index]
    }
}



pub struct TileMap_templates {
    map: HexagonalMap<TileTemplate>,
    size: MapSize
}
impl TileMap_templates {

    pub fn new(from: &TileMap_shape) -> Self {
        todo!()
    }

    pub fn iter(&self) -> impl Iterator {
        self.map.iter()
    }

    #[inline(always)]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }
}

impl Index<Hex> for TileMap_templates {
    type Output = TileTemplate;
    fn index(&self, index: Hex) -> &Self::Output {
        &self.map[index]
    }
}
impl IndexMut<Hex> for TileMap_templates {
    fn index_mut(&mut self, index: Hex) -> &mut Self::Output {
        &mut self.map[index]
    }
}




pub struct TileMap_props {
    map: HexagonalMap<Option<Prop>>,
    size: MapSize
}
impl TileMap_props {

    pub fn new(from: &TileMap_shape) -> Self {
        todo!()
    }

    pub fn iter(&self) -> impl Iterator {
        self.map.iter()
    }

    #[inline(always)]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }
}

impl Index<Hex> for TileMap_props {
    type Output = Option<Prop>;
    fn index(&self, index: Hex) -> &Self::Output {
        &self.map[index]
    }
}
impl IndexMut<Hex> for TileMap_props {
    fn index_mut(&mut self, index: Hex) -> &mut Self::Output {
        &mut self.map[index]
    }
}



pub struct TileMap {
    map: HexagonalMap<Tile>,
    size: MapSize
}
impl TileMap {

    pub fn new(templates: TileMap_templates, props: TileMap_props) -> Self {
        if templates.size != props.size {
            panic!("Both sub-mapes must be same size")
        }

        Self { map: HexagonalMap::new(Hex::ZERO, templates.size_u32(), |pos| {Tile { template: templates[pos], prop: props[pos]}}), size: templates.size }
    }

    pub fn iter(&self) -> impl Iterator {
        self.map.iter()
    }

    #[inline(always)]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }
}

impl Index<Hex> for TileMap {
    type Output = Tile;
    fn index(&self, index: Hex) -> &Self::Output {
        &self.map[index]
    }
}
impl IndexMut<Hex> for TileMap {
    fn index_mut(&mut self, index: Hex) -> &mut Self::Output {
        &mut self.map[index]
    }
}



pub trait DrawHexMap {
    const IMAGE_WIDTH: u32;
    const IMAGE_HEIGHT: u32;
    
    fn get_shape(&self, pos: Hex) -> DynamicImage;
    fn draw_to_image<C: Canvas>(img: &mut C);

    fn get_pos(pos: Hex, size: f32) -> Vec2 {
        let layout = HexLayout { 
            origin: Vec2 { x: Self::IMAGE_WIDTH as f32/2.0, y: Self::IMAGE_HEIGHT as f32 / 2.0 }, 
            orientation: HexOrientation::Pointy,
            scale: Vec2::splat(size) };
        layout.hex_to_world_pos(pos)
    }
}