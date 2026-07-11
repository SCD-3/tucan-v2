use std::ops::{Index, IndexMut};
use hexx::{Hex, storage::{HexStore, HexagonalMap}};
use image::Rgb;
use imageproc::drawing::{Canvas, draw_polygon_mut};
use rand::prelude::*;
use crate::{drawing::*, tiles::*};

type Result<T> = core::result::Result<T, String>;

pub const BIG_MAP_SIZE:   u8 = 104;
pub const SMALL_MAP_SIZE: u8 =  73;

const HEXMAP_RADIUS_BIG:   u32 = 6;
const HEXMAP_RADIUS_SMALL: u32 = 5;

const MIN_HEX_NEIGHBORS: u8 = 2;

const VILLAGE_COUNT: u8 = 10;
// const VILLAGE_OFFSET: u8 = 3;

const MIN_PROP_DISTANCE: u32 = 2;

const ARTIFACT_COUNT_BIG:           usize = 15;
const ARTIFACT_COUNT_SMALL:         usize = 10; // 10
const ARTIFACT_COUNT_PER_ART_BIG:   usize =  3;
const ARTIFACT_COUNT_PER_ART_SMALL: usize =  2;

/// Matching for pattern `Option<T>::Some(a)|Option<T>::None`.
/// 
/// `a` is ment to be "falsely" state, which can be equivalent to no state.
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

/// First parameter is map_size
/// 
/// Second is if big
/// 
/// Third is if small
#[macro_export]
macro_rules! match_size {
    ($name:expr, $big:expr, $small:expr) => {
        match $name {
            MapSize::Big => $big,
            MapSize::Small => $small
        }
    };
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum MapSize {
    Small = SMALL_MAP_SIZE,
    Big   = BIG_MAP_SIZE
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RawTileState {
    FreeToTake,
    Taken,
    Unknown
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct TileMap_rawShapeGen {
    map: HexagonalMap<RawTileState>,
    size: MapSize
}
impl TileMap_rawShapeGen {
    
    pub fn new<R: Rng>(size: MapSize, rng: &mut R) -> Result<Self> {
        let mut map = HexagonalMap::new(
            Hex::ZERO, 
            match_size!(size, HEXMAP_RADIUS_BIG, HEXMAP_RADIUS_SMALL), 
            |_| RawTileState::Unknown);
        
        map[Hex::ZERO] = RawTileState::FreeToTake;

        let mut map = Self { map, size };

        for i in 0..size as u8 {
            map.do_a_run(rng, i < MIN_HEX_NEIGHBORS);
        };
        // println!("{:?} {}", size, match_size!(size, HEXMAP_RADIUS_BIG, HEXMAP_RADIUS_SMALL));
        Ok(map)
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
impl HexStore<RawTileState> for TileMap_rawShapeGen {
    
    fn get(&self, hex: hexx::Hex) -> Option<&RawTileState> {
        self.map.get(hex)
    }

    fn get_mut(&mut self, hex: hexx::Hex) -> Option<&mut RawTileState> {
        self.map.get_mut(hex)
    }

    fn iter<'s>(&'s self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s RawTileState)>
    where
        RawTileState: 's
    {
        self.map.iter()
    }

    fn iter_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s mut RawTileState)>
    where
        RawTileState: 's
    {
        self.map.iter_mut()
    }

    fn values<'s>(&'s self) -> impl ExactSizeIterator<Item = &'s RawTileState>
    where
        RawTileState: 's
    {
        self.map.values()
    }

    fn values_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = &'s mut RawTileState>
    where
        RawTileState: 's
    {
        self.map.values_mut()
    }

}


#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct TileMap_shape {
    map: HexagonalMap<bool>,
    size: MapSize
}
impl TileMap_shape {
    
    pub fn new(from: TileMap_rawShapeGen) -> Result<Self> {
        let map = HexagonalMap::new(Hex::ZERO, match_size!(from.size, HEXMAP_RADIUS_BIG, HEXMAP_RADIUS_SMALL), |h| matches!(from[h], RawTileState::Taken));
        let out = Self { map, size: from.size };
        if out.has_no_holes() {
            Ok(out)
        }
        else {
            Err("found 1 size holes in the shape")?
        }
    }
    
    #[must_use]
    fn has_no_holes(&self) -> bool {
        for (hex, state) in self.iter() {
            if !*state
                && hex.all_neighbors().iter().all(|s| matches!(self.get(*s), Some(&true))) {
                    return false;
                }
            
        }
        true
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
impl HexStore<bool> for TileMap_shape {
    
    fn get(&self, hex: hexx::Hex) -> Option<&bool> {
        self.map.get(hex)
    }

    fn get_mut(&mut self, hex: hexx::Hex) -> Option<&mut bool> {
        self.map.get_mut(hex)
    }

    fn iter<'s>(&'s self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s bool)>
    where
        bool: 's
    {
        self.map.iter()
    }

    fn iter_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s mut bool)>
    where
        bool: 's
    {
        self.map.iter_mut()
    }

    fn values<'s>(&'s self) -> impl ExactSizeIterator<Item = &'s bool>
    where
        bool: 's
    {
        self.map.values()
    }

    fn values_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = &'s mut bool>
    where
        bool: 's
    {
        self.map.values_mut()
    }

}

impl DrawHexMap<bool> for TileMap_shape {
    type ColorSpace = Rgb<u8>;

    fn draw_element<C: Canvas<Pixel = Self::ColorSpace>>(&self, img: &mut C, hex: Hex, value: &bool, image_config: ImageConfig) {
        let pos = get_pos(hex, image_config);
        let points = get_hex_points(pos, image_config.hex_radius);
        if *value {
            draw_polygon_mut(img, &points, if hex == Hex::ZERO {Rgb([255, 0, 0])} else {Rgb([255, 255, 255])})
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct TileMap_templates {
    map: HexagonalMap<Option<TileTemplate>>,
    size: MapSize
}
impl TileMap_templates {

    pub fn new<R: Rng>(rng: &mut R, shape: &TileMap_shape, props: &TileMap_props) -> Result<Self> {
        assert_eq!(shape.size, props.size, "Both sub-mapes must be same size");

        let mut tiles = Self::prepare_random_tiles(rng, shape.size);
        // println!("{}", from.iter().filter(|(_, a)| **a).count());

        let mut out = Self { map: HexagonalMap::new(
            Hex::ZERO, 
            match_size!(shape.size, HEXMAP_RADIUS_BIG, HEXMAP_RADIUS_SMALL), 
            |h| if shape[h] {
                Some(tiles.next().expect("invalid number of tiles in `TileMap_templates::prepare_random_tiles`"))
            } 
            else {
                None
            }
        ), size: shape.size };
        drop(tiles); // we no longer need that iterator, but we do need rng from it

        out.fix_tiles_on_artifacts(rng, props)?;
        Ok(out)
    }

    fn fix_tiles_on_artifacts<R: Rng>(&mut self, rng: &mut R, props: &TileMap_props) -> Result<()>{
        
        fn needs_a_switch(template: Option<TileTemplate>, prop: PropOption) -> bool {
            if matches!(prop, PropOption::Some(Prop::Village(_))) {
                return false;
            }

            if let Some(template) = template && prop.has_prop() {
                // let template = template.unwrap();
                let prop = prop.unwrap().unwrap();
                if prop.get_template() != template {
                    // println!("replace {prop:?} {template}");
                    true
                }
                else {
                    // println!("don't replace {prop:?} {template}");
                    false
                }
            }
            else if !prop.has_prop() {
                false
            }
            else {
                panic!("prop on empty tile, {prop:?}")
            }
        }

        fn first_tile_of_template(template_map: &TileMap_templates, tiles: &mut Vec<Hex>, template: TileTemplate) -> Result<Hex> {
            for (i, hex) in tiles.iter().enumerate() {
                if template_map[*hex].is_some() && template_map[*hex].unwrap() == template {
                    let hex = *hex;
                    tiles.remove(i);
                    return Ok(hex);
                }
            }
            Err(format!("did not find template {template:?}"))
        }

        let mut tiles_no_artifacts: Vec<Hex> = props
        .iter()
        .filter_map(|(h, p)| 
            if let PropOption::NotAllowed = p {
                Some(h)
            } 
            else {
                None
            }
        )
        .collect();
        let mut free_tiles = Vec::new(); // tiles we got from removing tiles under artifacts
        let mut removed_tiles = Vec::new(); // tiles from where we removed tiles

        for ((hex, template), (_, prop)) in Iterator::zip(self.clone().iter(), props.iter()) {
            if needs_a_switch(*template, *prop) {
                let target_template = prop.unwrap().unwrap().get_template(); // what prop wants
                let removed_hex = first_tile_of_template(self, &mut tiles_no_artifacts, target_template)?;
                self[hex] = Some(target_template);
                free_tiles.push(template.unwrap());
                removed_tiles.push(removed_hex);
            }
        };

        free_tiles.shuffle(rng);
        for hex in removed_tiles {
            self[hex] = free_tiles.pop();
        }

        Ok(())
    }

    fn prepare_random_tiles<R: Rng>(rng: &mut R, map_size: MapSize) -> impl Iterator<Item = TileTemplate> {
        let mut out = Vec::new();
        for template in [SAND, FOREST, MOUNTAIN, WATER] {
            out.extend(vec![template; match_size!(map_size, template.amount_big(), template.amount_small())]);
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
impl HexStore<Option<TileTemplate>> for TileMap_templates {

    fn get(&self, hex: hexx::Hex) -> Option<&Option<TileTemplate>> {
        self.map.get(hex)
    }

    fn get_mut(&mut self, hex: hexx::Hex) -> Option<&mut Option<TileTemplate>> {
        self.map.get_mut(hex)
    }

    fn iter<'s>(&'s self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s Option<TileTemplate>)>
    where
        Option<TileTemplate>: 's
    {
        self.map.iter()
    }

    fn iter_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s mut Option<TileTemplate>)>
    where
        Option<TileTemplate>: 's
    {
        self.map.iter_mut()
    }

    fn values<'s>(&'s self) -> impl ExactSizeIterator<Item = &'s Option<TileTemplate>>
    where
        Option<TileTemplate>: 's
    {
        self.map.values()
    }

    fn values_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = &'s mut Option<TileTemplate>>
    where
        Option<TileTemplate>: 's
    {
        self.map.values_mut()
    }
    
}

impl DrawHexMap<Option<TileTemplate>> for TileMap_templates {
    type ColorSpace = Rgb<u8>;

    fn draw_element<C: Canvas<Pixel = Self::ColorSpace>>(&self, img: &mut C, hex: Hex, value: &Option<TileTemplate>, image_config: ImageConfig) {
        let pos = get_pos(hex, image_config);
        let points = get_hex_points(pos, image_config.hex_radius);
        if let Some(template) = value { draw_polygon_mut(img, &points, if hex == Hex::ZERO {rgb!(255, 0, 0)} else {template.color()}) }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct TileMap_props {
    map: HexagonalMap<PropOption>,
    size: MapSize
}
impl TileMap_props {

    pub fn new<R: Rng>(rng: &mut R, from: &TileMap_shape) -> Result<Self> {
        let mut output = Self {
            map: HexagonalMap::new(
                Hex::ZERO, 
                match_size!(from.size, HEXMAP_RADIUS_BIG, HEXMAP_RADIUS_SMALL), 
                |_| PropOption::NotAllowed), 
            size: from.size
        };
        output.place_villages(from)?;
        if output.has_invalid_villages() {
            Err("found villages placed too close to each others")?;
        }
        output.place_prop_places(from)?;
        // let clone = output.clone();
        // let spots = clone.iter().filter_map(|a| if let (h, PropOption::CanHave) = a {Some(h)} else {None});
        // for (prop, place) in Iterator::zip(Self::prepare_random_props(rng,from.size), spots) {
        //     output[place].give_prop(prop, false);
        // }
        let mut props = Self::prepare_random_props(rng,from.size);
        output.map
        .iter_mut()
        .for_each(|(_, prop)| 
            if let PropOption::CanHave = *prop 
                {prop.give_prop(props.next().expect("ran out of random props"), false);});

        Ok(output)
    }

    fn place_villages(&mut self, shape: &TileMap_shape) -> Result<()> {
        let edge = shape.find_edges();
        let edge_len = edge.len();
        let distance = edge_len as f64 / VILLAGE_COUNT as f64;
        let mut i = 0f64;
        let mut village_pos: Vec<Hex> = Vec::new();
        for _ in 0..VILLAGE_COUNT {
            village_pos.push(*edge.get(i.round() as usize).ok_or(format!("out of edge. Index {i}. Edge_len {edge_len}"))?);
            i += distance;
        }
        assert_eq!(village_pos.len(), VILLAGE_COUNT as usize, "invalid number of villages. Expected {VILLAGE_COUNT}, got {}.", village_pos.len());

        for (id, hex) in village_pos.iter().enumerate() {
            self[*hex].give_prop(Prop::Village(id as u8 + 1), true);
        };
        Ok(())
    }

    fn has_invalid_villages(&self) -> bool {
        self.iter().any(|(hex, prop)| {
            match prop {
                PropOption::Some(Prop::Village(_)) => {
                    hex
                    .all_neighbors()
                    .iter()
                    .any(
                        |h| 
                        matches!(self.get(*h), Some(PropOption::Some(Prop::Village(_))))
                    )
                }
                _ => false
            }
        })
    }

    fn place_prop_places(&mut self, shape: &TileMap_shape) -> Result<()> {
        macro_rules! prepare_new_ring {
            ($size:expr) => {
                Hex::ZERO.ring($size).step_by(MIN_PROP_DISTANCE as usize)
            };
        }

        let mut placed_props = 1;
        let mut ring_distance = MIN_PROP_DISTANCE;
        let mut ring = prepare_new_ring!(ring_distance);
        self[Hex::ZERO].allow_prop();
        loop {
            if placed_props == match_size!(self.size, ARTIFACT_COUNT_BIG, ARTIFACT_COUNT_SMALL) {
                break;
            }
            match ring.next() {
                Some(hex) => {if 
                        *shape.get(hex).ok_or(format!("ring index out of bounds. Hex: {hex:?} ring size: {ring_distance}"))? && 
                        !self.get(hex).ok_or(format!("ring index out of bounds. Hex: {hex:?} ring size: {ring_distance}"))?.is_allowed() 
                        && !hex.all_neighbors().iter()
                            .any(|h| self.get(*h).unwrap_or(&PropOption::NotAllowed).has_prop()) 
                    {
                    
                    placed_props += 1; self[hex].allow_prop(); 
                    // println!("Allowing at {hex:?}. Prop number: {placed_props}")
                    }
                },
                None => {ring_distance += MIN_PROP_DISTANCE; ring = prepare_new_ring!(ring_distance); 
                    // println!("Expanding ring. New radius: {ring_distance}")
                }
            }
        }
        Ok(())
    }

    fn prepare_random_props<R: Rng>(rng: &mut R, map_size: MapSize) -> impl Iterator<Item = Prop> {
        let mut out = Vec::new();
        let artifact_count_per_art = match_size!(map_size, ARTIFACT_COUNT_PER_ART_BIG, ARTIFACT_COUNT_PER_ART_SMALL);
        for prop in [SAND.primary_art(), FOREST.primary_art(), MOUNTAIN.primary_art(), WATER.primary_art()] {
            out.extend(vec![prop; artifact_count_per_art]);
        }
        // for prop in [SAND.secondary_art(), FOREST.secondary_art(), MOUNTAIN.secondary_art(), WATER.secondary_art()] {
        //     if let Some(item) = prop {out.extend(vec![item; artifact_count_per_art])};
        // }
        for item in [SAND.secondary_art(), FOREST.secondary_art(), MOUNTAIN.secondary_art(), WATER.secondary_art()]
            .into_iter()
            .flatten() 
        {out.extend(vec![item; artifact_count_per_art])}

        out.shuffle(rng);
        out.into_iter()
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
impl HexStore<PropOption> for TileMap_props {
    
    fn get(&self, hex: hexx::Hex) -> Option<&PropOption> {
        self.map.get(hex)
    }

    fn get_mut(&mut self, hex: hexx::Hex) -> Option<&mut PropOption> {
        self.map.get_mut(hex)
    }

    fn iter<'s>(&'s self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s PropOption)>
    where
        Tile: 's
    {
        self.map.iter()
    }

    fn iter_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s mut PropOption)>
    where
        Tile: 's
    {
        self.map.iter_mut()
    }

    fn values<'s>(&'s self) -> impl ExactSizeIterator<Item = &'s PropOption>
    where
        Tile: 's
    {
        self.map.values()
    }

    fn values_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = &'s mut PropOption>
    where
        Tile: 's
    {
        self.map.values_mut()
    }
    
}

impl DrawHexMap<PropOption> for TileMap_props {
    type ColorSpace = Rgb<u8>;

    fn draw_element<C: Canvas<Pixel = Self::ColorSpace>>(&self, img: &mut C, hex: Hex, value: &PropOption, image_config: ImageConfig) {
        let pos = get_pos(hex, image_config);
        let points = get_hex_points(pos, image_config.hex_radius);
        let color = if false {
            // hex == Hex::ZERO
            rgb!(255, 0, 0)
        }
        else {
            match value {
                PropOption::Some(Prop::Village(_))  => rgb!(255, 0  , 255),
                PropOption::Some(Prop::Monolith)    => rgb!(50 , 50 , 50 ),
                PropOption::Some(Prop::Book)        => rgb!(100, 50 , 0  ),
                PropOption::Some(Prop::Bird)        => rgb!(0  , 200, 0  ),
                PropOption::Some(Prop::WeirdMonkey) => rgb!(100, 0  , 100),
                PropOption::Some(Prop::Dragon)      => rgb!(50 , 255, 255),

                PropOption::CanHave => panic!("attempted to draw empty artifact slot at {hex:?}"),
                PropOption::NotAllowed => rgb!(0, 0, 0)
            }
        };
        draw_polygon_mut(img, &points, color)
    }
}



pub struct TileMap {
    map: HexagonalMap<Tile>,
    size: MapSize
}
impl TileMap {

    pub fn new(templates: TileMap_templates, props: TileMap_props) -> Result<Self> {
        assert_eq!(templates.size, props.size, "Both sub-mapes must be same size");
        let size = templates.size;

        Ok(
            Self { 
                map: HexagonalMap::new(
                    Hex::ZERO, 
                    match_size!(size, HEXMAP_RADIUS_BIG, HEXMAP_RADIUS_SMALL), 
                    |pos| {
                        Tile { 
                            template: templates[pos], 
                            prop: props[pos]
                        }
                    }
                ), 
                size: templates.size 
            }
        )
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &Tile)> {
        self.map.iter()
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
impl HexStore<Tile> for TileMap {
    
    fn get(&self, hex: hexx::Hex) -> Option<&Tile> {
        self.map.get(hex)
    }

    fn get_mut(&mut self, hex: hexx::Hex) -> Option<&mut Tile> {
        self.map.get_mut(hex)
    }

    fn iter<'s>(&'s self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s Tile)>
    where
        Tile: 's
    {
        self.map.iter()
    }

    fn iter_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = (hexx::Hex, &'s mut Tile)>
    where
        Tile: 's
    {
        self.map.iter_mut()
    }

    fn values<'s>(&'s self) -> impl ExactSizeIterator<Item = &'s Tile>
    where
        Tile: 's
    {
        self.map.values()
    }

    fn values_mut<'s>(&'s mut self) -> impl ExactSizeIterator<Item = &'s mut Tile>
    where
        Tile: 's
    {
        self.map.values_mut()
    }
    
}

impl DrawHexMap<Tile> for TileMap {
    type ColorSpace = Rgb<u8>;

    fn draw_element<C: Canvas<Pixel = Self::ColorSpace>>(&self, img: &mut C, hex: Hex, value: &Tile, image_config: ImageConfig) {
        todo!()
    }
}