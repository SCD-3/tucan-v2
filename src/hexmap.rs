use std::{ops::{Index, IndexMut}, process::Output};

use hexx::{Hex, storage::{HexStore, HexagonalMap}, layout::HexLayout, Vec2, orientation::HexOrientation};
use image::DynamicImage;
use imageproc::drawing::Canvas;
use rand::prelude::*;
use crate::tiles::*;

pub const SMALL_MAP_SIZE: u8 = 73;
pub const BIG_MAP_SIZE:   u8 = 104;

const MIN_NEIGHBORS: u8 = 2;

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
    pub fn new<R: Rng>(size: MapSize, rng: &mut R) -> Self {
        let mut map = HexagonalMap::new(Hex::ZERO, size as u32, |_| RawTileState::Unknown);
        map[Hex::ZERO] = RawTileState::FreeToTake;

        let mut map = Self { map, size };

        for _ in 0..size as u8 {
            map.do_a_run(rng);
        };
        map
    }

    pub fn iter(&self) -> impl Iterator {
        self.map.iter()
    }

    #[inline(always)]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }

    fn do_a_run<R: Rng>(&mut self, rng: &mut R) {
        let (hex, _) = self.map.iter().filter(|(hex, state)| matches!(state, RawTileState::FreeToTake) && self.count_neighbors(*hex) >= MIN_NEIGHBORS).choose(rng).expect("We ran out of tiles. Congrats");
        self[hex] = RawTileState::Taken;
        for i in hex.all_neighbors() {
            if matches!(self.get(i), Some(RawTileState::Unknown)) {
                self[i] = RawTileState::FreeToTake;
            }
        }
    }

    fn count_neighbors(&self, hex: Hex) -> u8 {
        let mut counted = 0;
        for i in hex.all_neighbors() {
            if matches!(self.get(i), Some(RawTileState::Taken)) {
                counted += 1;
            }
        };
        counted
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
impl Get<Hex> for TileMap_rawShapeGen {
    fn get(&self, index: Hex) -> Option<&Self::Output> {
        self.map.get(index)
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
impl Get<Hex> for TileMap_shape {
    fn get(&self, index: Hex) -> Option<&Self::Output> {
        self.map.get(index)
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
impl Get<Hex> for TileMap_templates {
    fn get(&self, index: Hex) -> Option<&Self::Output> {
        self.map.get(index)
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
impl Get<Hex> for TileMap_props {
    fn get(&self, index: Hex) -> Option<&Self::Output> {
        self.map.get(index)
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
impl Get<Hex> for TileMap {
    fn get(&self, index: Hex) -> Option<&Self::Output> {
        self.map.get(index)
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

trait Get<Idx: ?Sized>: Index<Idx> {
    fn get(&self, index: Idx) -> Option<&Self::Output>;
}