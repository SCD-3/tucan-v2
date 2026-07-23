use std::fmt::Display;
use image::{ImageBuffer, Rgba};

static MONOLITH_IMAGE:     &[u8] = include_bytes!(r"..\vol\assets\artifacts\Obelisk.png");
static BOOK_IMAGE:         &[u8] = include_bytes!(r"..\vol\assets\artifacts\Book.png");
static BIRD_IMAGE:         &[u8] = include_bytes!(r"..\vol\assets\artifacts\Bird.png");
static WEIRD_MONKEY_IMAGE: &[u8] = include_bytes!(r"..\vol\assets\artifacts\WeirdMonkey.png");
static DRAGON_IMAGE:       &[u8] = include_bytes!(r"..\vol\assets\artifacts\Dragon.png");

const IMAGE_PROP_SIZE: u32 = 80;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PropOption {
    Some(Prop),
    CanHave,
    NotAllowed
}
impl PropOption {
    
    #[must_use]
    #[inline(always)]
    pub const fn has_prop(self) -> bool {
        matches!(self, PropOption::Some(_))
    }

    #[must_use]
    #[inline(always)]
    pub const fn can_have_prop(self) -> bool {
        matches!(self, PropOption::CanHave)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_allowed(self) -> bool {
        !matches!(self, PropOption::NotAllowed)
    }

    #[must_use]
    pub const fn unwrap(self) -> Option<Prop> {
        match self {
            PropOption::Some(prop) => Some(prop),
            PropOption::CanHave => None,
            PropOption::NotAllowed => panic!("called `PropOption::unwrap()` on a `PropOption::NotAllowed` value")
        }
    }

    pub fn allow_prop(&mut self) {
        if self.has_prop() {
            panic!("place taken; can't allow prop")
        }
        else {
            *self = Self::CanHave
        }
    }

    pub fn give_prop(&mut self, prop: Prop, force: bool) {
        if !force && !self.can_have_prop() {
            panic!("can't place prop {prop:?}")
        }
        else {
            *self = PropOption::Some(prop);
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct TileTemplate {
    name: &'static str,

    color: Rgba<u8>,
    amount_small: usize,
    amount_big: usize,

    primary_art: Prop,
    secondary_art: Option<Prop>
}
impl TileTemplate {

    #[inline(always)]
    #[must_use]
    pub const fn color(&self) -> Rgba<u8> {
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

    #[inline(always)]
    #[must_use]
    pub const fn primary_art(&self) -> Prop {
        self.primary_art
    }

    #[inline(always)]
    #[must_use]
    pub const fn secondary_art(&self) -> Option<Prop> {
        self.secondary_art
    }
}

impl Display for TileTemplate {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Tile {
    pub template: Option<TileTemplate>,
    pub prop: PropOption

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
impl Prop {

    #[must_use]
    pub fn get_template(self) -> TileTemplate {
        for tempalte in TEMPLATES {
            if tempalte.primary_art == self || tempalte.secondary_art == Some(self) {
                return tempalte;
            }
        }
        panic!("did not find template for prop {self:?}")
    }

    #[must_use]
    pub fn get_color(self) -> Rgba<u8> {
            match self {
                Prop::Village(_)  => Rgba([255, 0  , 255, 255]),
                Prop::Monolith    => Rgba([50 , 50 , 50 , 255]),
                Prop::Book        => Rgba([100, 50 , 0  , 255]),
                Prop::Bird        => Rgba([0  , 200, 0  , 255]),
                Prop::WeirdMonkey => Rgba([100, 0  , 100, 255]),
                Prop::Dragon      => Rgba([50 , 255, 255, 255]),
            }
    }

    #[must_use]
    pub fn get_image(self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let bytes = match self {
            Prop::Village(_)  => panic!("Prop::Village does not have assiociated image"),
            Prop::Monolith    => MONOLITH_IMAGE,
            Prop::Book        => BOOK_IMAGE,
            Prop::Bird        => BIRD_IMAGE,
            Prop::WeirdMonkey => WEIRD_MONKEY_IMAGE,
            Prop::Dragon      => DRAGON_IMAGE,
        };
        let image = image::load_from_memory(bytes)
            .unwrap_or_else(|_| panic!("failure while loading image for prop {self:?}"))
            .into_rgba8();
        image::imageops::resize(&image, IMAGE_PROP_SIZE, IMAGE_PROP_SIZE, image::imageops::FilterType::Nearest)
    }
}

pub const TEMPLATES: [TileTemplate; 4] = [SAND, FOREST, MOUNTAIN, WATER];

pub const SAND:     TileTemplate = TileTemplate { name: "SAND"    , color: Rgba([203, 189, 147, 255]), amount_small: 24, amount_big: 34, primary_art: Prop::Monolith   , secondary_art: None             };
pub const FOREST:   TileTemplate = TileTemplate { name: "FOREST"  , color: Rgba([46 , 111, 64 , 255]), amount_small: 20, amount_big: 29, primary_art: Prop::Bird       , secondary_art: Some(Prop::Book) };
pub const MOUNTAIN: TileTemplate = TileTemplate { name: "MOUNTAIN", color: Rgba([140, 140, 140, 255]), amount_small: 16, amount_big: 24, primary_art: Prop::WeirdMonkey, secondary_art: None             };
pub const WATER:    TileTemplate = TileTemplate { name: "WATER"   , color: Rgba([46 , 108, 216, 255]), amount_small: 13, amount_big: 17, primary_art: Prop::Dragon     , secondary_art: None             };