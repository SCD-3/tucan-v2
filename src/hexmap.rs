use std::ops::{Index, IndexMut};
use hexx::{Hex, storage::{HexStore, HexagonalMap}};
use image::Rgb;
use imageproc::drawing::{Canvas, draw_polygon_mut};
use rand::prelude::*;
use crate::{drawing::*, tiles::*};

pub const SMALL_MAP_SIZE: u8 = 73;
pub const BIG_MAP_SIZE:   u8 = 104;

const MIN_HEX_NEIGHBORS: u8 = 2;

const VILLAGE_COUNT: u8 = 10;
const VILLAGE_OFFSET: u8 = 3;
const MIN_PROP_DISTANEC: usize = 2;

/// Matching for pattern `Option<T>::Some(a)` or `Option<T>::None`
/// 
/// `a` is ment to be "falsely" state, which can be equivalent to no state
macro_rules! empty {
    ($a:pat) => {
        Some($a)|None
    };
}

/// RGB colors.
///
/// For the purpose of color conversion, as well as blending, the implementation of `Pixel`
/// assumes an `sRGB` color space of its data.
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        Rgb::<u8>([$r, $g, $b])
    };
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum MapSize {
    Small = SMALL_MAP_SIZE,
    Big   = BIG_MAP_SIZE
}

pub enum RawTileState {
    FreeToTake,
    Taken,
    Unknown
}

#[allow(non_camel_case_types)]
pub struct TileMap_rawShapeGen {
    map: HexagonalMap<RawTileState>,
    size: MapSize
}
impl TileMap_rawShapeGen {

    #[must_use]
    pub fn new<R: Rng>(size: MapSize, rng: &mut R) -> Self {
        let mut map = HexagonalMap::new(Hex::ZERO, size as u32, |_| RawTileState::Unknown);
        map[Hex::ZERO] = RawTileState::FreeToTake;

        let mut map = Self { map, size };

        for i in 0..size as u8 {
            map.do_a_run(rng, i < MIN_HEX_NEIGHBORS);
        };
        map
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &RawTileState)> {
        self.map.iter()
    }

    #[inline(always)]
    #[must_use]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }

    fn do_a_run<R: Rng>(&mut self, rng: &mut R, overule_min_neighbors: bool) {
        // println!("{}", self.iter().filter(|(_, state)| matches!(state, RawTileState::FreeToTake)).collect::<Vec<_>>().len());

        let (hex, _) = self.iter()
        .filter(|(hex, state)| matches!(state, RawTileState::FreeToTake) && (overule_min_neighbors || self.count_neighbors(*hex) >= MIN_HEX_NEIGHBORS))
        .choose(rng)
        .expect("We ran out of tiles. Congrats");

        self[hex] = RawTileState::Taken;
        for i in hex.all_neighbors() {
            if matches!(self.get(i), Some(RawTileState::Unknown)) {
                self[i] = RawTileState::FreeToTake;
            }
        }
    }

    #[must_use]
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


#[allow(non_camel_case_types)]
pub struct TileMap_shape {
    map: HexagonalMap<bool>,
    size: MapSize
}
impl TileMap_shape {
    
    #[must_use]
    pub fn new(from: TileMap_rawShapeGen) -> Self {
        let map = HexagonalMap::new(Hex::ZERO, from.size_u32(), |h| matches!(from[h], RawTileState::Taken));
        Self { map, size: from.size }
    }
    
    #[must_use]
    pub fn has_no_holes(&self) -> bool {
        for (hex, state) in self.iter() {
            if !*state
                && hex.all_neighbors().iter().all(|s| matches!(self.get(*s), Some(&true))) {
                    return false;
                }
            
        }
        true
    }


    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &bool)> {
        self.map.iter()
    }

    #[inline(always)]
    #[must_use]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }

    #[must_use]
    fn find_edge_single(&self) -> Hex {
        let mut pos = Hex::ZERO;
        while !matches!(self.get(Hex::new(pos.x+1, pos.y)), empty!(false)) {
            pos.x += 1;
        };
        pos
    }

    #[must_use]
    pub fn find_edges(&self) -> Vec<Hex> {
        let mut pos = self.find_edge_single();
        let mut edge = vec![pos];
        let mut last_state = self.get(pos);
        
        loop {
            for i in pos.all_neighbors() {
                match (last_state, self.get(i)) {
                    (empty!(false), Some(true)) => {last_state = self.get(i); pos = i; edge.push(pos); break;}
                    (empty!(false), empty!(false)) => continue,
                    (Some(true), Some(true)) => continue,
                    (Some(true), empty!(false)) => last_state = self.get(i)
                }
            }
            // println!("Added {:?} First element is {:?} Are they equal? {}", edge.last(), edge.first(), edge.last() == edge.first());
            if edge.last() == edge.first() {
                edge.pop();
                break edge
            }
        }
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
                draw_polygon_mut(img, &points, if hex == Hex::ZERO {Rgb([255, 0, 0])} else {Rgb([255, 255, 255])})
            }
        }
    }
}


#[allow(non_camel_case_types)]
pub struct TileMap_templates {
    map: HexagonalMap<Option<TileTemplate>>,
    size: MapSize
}
impl TileMap_templates {

    #[must_use]
    pub fn new<R: Rng>(rng: &mut R, from: &TileMap_shape) -> Self {
        let mut tiles = Self::prepare_random_tiles(rng, from.size);
        // println!("{}", from.iter().filter(|(_, a)| **a).count());

        Self { map: HexagonalMap::new(
            Hex::ZERO, 
            from.size_u32(), 
            |h| if from[h] {Some(tiles.next().unwrap())} else {None}), size: from.size }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &Option<TileTemplate>)> {
        self.map.iter()
    }

    #[inline(always)]
    #[must_use]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }

    pub fn prepare_random_tiles<R: Rng>(rng: &mut R, map_size: MapSize) -> impl Iterator<Item = TileTemplate> {
        let mut out = Vec::new();
        match map_size {
            MapSize::Small => {
                for template in [SAND, FOREST, MOUNTAIN, WATER] {
                    out.extend(vec![template; template.amount_small()]);
                }
            },
            MapSize::Big => {
                for template in [SAND, FOREST, MOUNTAIN, WATER] {
                    out.extend(vec![template; template.amount_big()]);
                }
            }
        }
        out.shuffle(rng);
        out.into_iter()
    }
}

impl Index<Hex> for TileMap_templates {
    type Output = Option<TileTemplate>;
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
impl DrawHexMap for TileMap_templates {
    type ColorSpace = Rgb<u8>;

    fn draw<C: Canvas<Pixel = Self::ColorSpace>>(self, img: &mut C, image_config: ImageConfig) {
        for (hex, state) in self.iter() {
            let pos = Self::get_pos(hex, image_config);
            let points = get_hex_points(pos, image_config.hex_radius);
            if let Some(template) = state { draw_polygon_mut(img, &points, if hex == Hex::ZERO {Rgb([255, 0, 0])} else {template.color()}) }
        }
    }
}


#[allow(non_camel_case_types)]
pub struct TileMap_props {
    map: HexagonalMap<PropOption>,
    size: MapSize
}
impl TileMap_props {

    #[must_use]
    pub fn new<R: Rng>(rng: &mut R, from: &TileMap_shape) -> Self {
        let mut output = Self {
            map: HexagonalMap::new(Hex::ZERO, from.size_u32(), |_| PropOption::CanHave), 
            size: from.size
        };
        output.place_villages(from);

        output
    }

    fn trim_tiles_too_close(&mut self) {
        todo!()
    }

    fn place_villages(&mut self, shape: &TileMap_shape) {
        let edge = shape.find_edges();
        let edge_len = edge.len();
        let distance = edge_len / VILLAGE_COUNT as usize;
        let mut village_pos: Vec<Hex> = Vec::new();
        let mut edge_iter = edge.iter();
        for _ in 0..VILLAGE_COUNT {
            for _ in 0..distance-1 {edge_iter.next();};
            village_pos.push(*edge_iter.next().unwrap());
        }
        if village_pos.len() != VILLAGE_COUNT as usize {
            panic!("invalid number of villages. Expected {VILLAGE_COUNT}, got {}.", village_pos.len())
        }

        for (id, hex) in village_pos.iter().enumerate() {
            let value = self.get(*hex);
            if value.is_none() || !value.unwrap().can_have_prop() {
                panic!("attempted to place village at invalid position, at hex {hex:?}, with value {value:?}")
            }
            else {
                self[*hex].give_prop(Prop::Village(id as u8 + 1));
            }
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &PropOption)> {
        self.map.iter()
    }

    #[inline(always)]
    #[must_use]
    pub fn size_u32(&self) -> u32 {
        self.size as u32
    }
}

impl Index<Hex> for TileMap_props {
    type Output = PropOption;
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
impl DrawHexMap for TileMap_props {
    type ColorSpace = Rgb<u8>;

    fn draw<C: Canvas<Pixel = Self::ColorSpace>>(self, img: &mut C, image_config: ImageConfig) {
        for (hex, state) in self.iter() {
            let pos = Self::get_pos(hex, image_config);
            let points = get_hex_points(pos, image_config.hex_radius);
            let color;
            if hex == Hex::ZERO {
                color = rgb!(255, 0, 0);
            }
            else if state.is_some() {
                color = match state.unwrap() {
                    Some(Prop::Village(_)) => rgb!(255, 0, 255),
                    Some(_) => rgb!(0, 255, 0),
                    None => rgb!(255, 255, 0)
                }
            }
            else {
                color = rgb!(0, 0, 0);
            }

            draw_polygon_mut(img, &points, color)
        }
    }
}



pub struct TileMap {
    map: HexagonalMap<Tile>,
    size: MapSize
}
impl TileMap {

    #[must_use]
    pub fn new(templates: TileMap_templates, props: TileMap_props) -> Self {
        if templates.size != props.size {
            panic!("Both sub-mapes must be same size")
        }

        Self { map: HexagonalMap::new(Hex::ZERO, templates.size_u32(), |pos| {Tile { template: templates[pos], prop: props[pos]}}), size: templates.size }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &Tile)> {
        self.map.iter()
    }

    #[inline(always)]
    #[must_use]
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
    #[must_use]
    fn get(&self, index: Idx) -> Option<&Self::Output>;
}