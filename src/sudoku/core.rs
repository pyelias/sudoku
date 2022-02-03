use std::num::NonZeroU8;

#[derive(Copy, Clone, PartialEq)]
pub struct Number(NonZeroU8);

impl Number {
    pub fn safe_new(n: u8) -> Option<Self> {
        Some(Self(NonZeroU8::new(n)?))
    }
    
    pub fn new(n: u8) -> Self {
        Self::safe_new(n).unwrap()
    }

    pub fn get(&self) -> u8 {
        self.0.get()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Space {
    Empty,
    Full(Number),
}

impl std::fmt::Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, " "),
            Self::Full(num) => write!(f, "{}", num.0.get())
        }
    }
}

#[derive(Copy, Clone)]
pub struct Coord(pub(in super) u8);

impl Coord {
    pub fn new(x: u8, y: u8) -> Self {
        Self(y * 9 + x)   
    }

    pub fn all() -> impl Iterator<Item=Self> {
        (0..81).map(Coord)
    }

    pub fn x(&self) -> u8 {
        self.0 % 9
    }
    
    pub fn y(&self) -> u8 {
        self.0 / 9
    }

    pub fn square(&self) -> u8 {
        3 * (self.y() / 3) + (self.x() / 3)
    }
}

#[derive(Copy, Clone)]
pub struct Board([Space; 81]);

impl Board {
    pub fn new() -> Self {
        Self([Space::Empty; 81])
    }
}

impl std::ops::Index<Coord> for Board {
    type Output = Space;
    fn index(&self, index: Coord) -> &Space {
        &self.0[index.0 as usize]
    }
}

impl std::ops::IndexMut<Coord> for Board {
    fn index_mut(&mut self, index: Coord) -> &mut Space {
        &mut self.0[index.0 as usize]
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                write!(f, "{}", self[Coord::new(x, y)])?;
                if x == 2 || x == 5 {
                    write!(f, "|")?;
                }
            }
            if y != 8 {
                write!(f, "\n")?;
            }
            if y == 2 || y == 5 {
                write!(f, "---+---+---\n")?;
            }
        }
        return Ok(());
    }
}