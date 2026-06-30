use std::ops::{Index, IndexMut};

use hexx::{Hex, storage::{HexStore, HexagonalMap}};
use image::Rgb;
use imageproc::drawing::{Canvas, draw_polygon_mut};
use rand::prelude::*;
use crate::{tiles::*, drawing::*};

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

        for i in 0..size as u8 {
            map.do_a_run(rng, i < MIN_NEIGHBORS);
        };
        map
    }

    pub fn iter(&self) -> impl Iterator<Item = (Hex, &RawTileState)> {
        self.map.iter()
    }

    #[inline(always)]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }

    fn do_a_run<R: Rng>(&mut self, rng: &mut R, overule_min_neighbors: bool) {
        // println!("{}", self.iter().filter(|(_, state)| matches!(state, RawTileState::FreeToTake)).collect::<Vec<_>>().len());

        let (hex, _) = self.map.iter()
        .filter(|(hex, state)| matches!(state, RawTileState::FreeToTake) && (overule_min_neighbors || self.count_neighbors(*hex) >= MIN_NEIGHBORS))
        .choose(rng)
        .expect("We ran out of tiles. Congrats");

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
    
    pub fn has_no_holes(&self) -> bool {
        for (hex, state) in self.iter() {
            if !*state {
                if hex.all_neighbors().iter().all(|s| matches!(self.get(*s), Some(&true))) {
                    return false;
                }
            }
            
        }
        true
    }


    pub fn iter(&self) -> impl Iterator<Item = (Hex, &bool)> {
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
impl DrawHexMap for TileMap_shape {
    type ColorSpace = Rgb<u8>;

    fn draw<C: Canvas<Pixel = Self::ColorSpace>>(self, img: &mut C, image_config: ImageConfig) {
        for (hex, state) in self.iter() {
            let pos = Self::get_pos(hex, image_config);
            let points = get_hex_points(pos, image_config.hex_radius);
            if *state {
                draw_polygon_mut(img, &points, Rgb([255, 255, 255]))
            }
        }
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

    pub fn iter(&self) -> impl Iterator<Item = (Hex, &TileTemplate)> {
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

    pub fn iter(&self) -> impl Iterator<Item = (Hex, &Option<Prop>)> {
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

    pub fn iter(&self) -> impl Iterator<Item = (Hex, &Tile)> {
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

trait Get<Idx: ?Sized>: Index<Idx> {
    fn get(&self, index: Idx) -> Option<&Self::Output>;
}