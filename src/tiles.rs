use image::Rgb;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct TileTemplate {
    color: Rgb<u8>,
    amount_small: usize,
    amount_big: usize
}
impl TileTemplate {

    #[inline(always)]
    #[must_use]
    pub const fn color(&self) -> Rgb<u8> {
        self.color
    }

    #[inline(always)]
    #[must_use]
    pub const fn amount_small(&self) -> usize {
        self.amount_small
    }

    #[inline(always)]
    #[must_use]
    pub const fn amount_big(&self) -> usize {
        self.amount_big
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Tile {
    pub template: Option<TileTemplate>,
    pub prop: Option<Prop>

}
impl Tile {
    
    #[inline(always)]
    #[must_use]
    pub fn taken(&self) -> bool {
        self.template.is_some()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Prop {
    Village(u8),
    Monolith,
    Book,
    Bird,
    WeirdMonkey,
    Dragon
}



pub const SAND:     TileTemplate = TileTemplate { color: Rgb([203, 189, 147]), amount_small: 24, amount_big: 34 };
pub const FOREST:   TileTemplate = TileTemplate { color: Rgb([46 , 111, 64 ]), amount_small: 20, amount_big: 29 };
pub const MOUNTAIN: TileTemplate = TileTemplate { color: Rgb([140, 140, 140]), amount_small: 16, amount_big: 24 };
pub const WATER:    TileTemplate = TileTemplate { color: Rgb([46 , 108, 216]), amount_small: 13, amount_big: 17 };