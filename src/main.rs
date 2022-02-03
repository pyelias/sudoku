#[derive(Copy, Clone, PartialEq)]
pub struct Number(std::num::NonZeroU8);

impl Number {
    fn new(n: u8) -> Self {
        Self(std::num::NonZeroU8::new(n).unwrap())
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
pub struct Coord(u8);

impl Coord {
    fn new(x: u8, y: u8) -> Self {
        Self(y * 9 + x)   
    }

    fn all() -> impl Iterator<Item=Self> {
        (0..81).map(|n| Self(n))
    }

    fn x(&self) -> u8 {
        self.0 % 9
    }
    
    fn y(&self) -> u8 {
        self.0 / 9
    }

    fn square(&self) -> u8 {
        3 * (self.y() / 3) + (self.x() / 3)
    }
}

// row, column, 3x3
#[derive(Copy, Clone)]
struct Region([Space; 9]);

struct AllowedNumbers(u16);

impl AllowedNumbers {
    fn all() -> Self {
        Self(0b111_111_111)
    }

    fn get_mask(num: Number) -> u16 {
        1 << (num.0.get() - 1)
    }

    fn disallow(&mut self, num: Number) {
        self.0 &= !Self::get_mask(num);
    }

    fn is_allowed(&self, num: Number) -> bool {
        self.0 & Self::get_mask(num) != 0
    }

    fn count_allowed(&self) -> u8 {
        self.0.count_ones() as u8
    }

    fn allowed(&self) -> impl Iterator<Item=Number> + '_ {
        (1..10).map(Number::new).filter(|n| self.is_allowed(*n))
    }
}

#[derive(Copy, Clone)]
pub struct Board([Space; 81]);

impl Board {
    pub fn new() -> Self {
        Self([Space::Empty; 81])
    }

    fn get_row(&self, row: u8) -> Region {
        let mut reg = [Space::Empty; 9];
        for x in 0..9 {
            reg[x as usize] = self[Coord::new(x, row)];
        }
        Region(reg)
    }

    fn get_col(&self, col: u8) -> Region {
        let mut reg = [Space::Empty; 9];
        for y in 0..9 {
            reg[y as usize] = self[Coord::new(col, y)];
        }
        Region(reg)
    }

    fn get_square(&self, square: u8) -> Region {
        let start_x = 3 * (square % 3);
        let start_y = 3 * (square / 3);
    
        let mut reg = [Space::Empty; 9];
        for y in 0..3 {
            for x in 0..3 {
                let coord = Coord::new(start_x + x, start_y + y);
                reg[(3 * x + y) as usize] = self[coord];
            }
        }
        Region(reg)
    }
    
    fn regions(&self, coord: Coord) -> [Region; 3] {
        [
            self.get_col(coord.x()),
            self.get_row(coord.y()),
            self.get_square(coord.square())
        ]
    }

    fn allowed(&self, coord: Coord) -> AllowedNumbers {
        let mut allowed = AllowedNumbers::all();
        for reg in self.regions(coord) {
            for space in reg.0 {
                if let Space::Full(n) = space {
                    allowed.disallow(n);
                }
            }
        }
        return allowed;
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

fn make_forced_choices(mut board: Board) -> Option<Board> {
    'look_for_forces: loop {
        for coord in Coord::all() {
            if let Space::Full(_) = board[coord] {
                continue;
            }
            
            let allowed = board.allowed(coord);
            let count = allowed.count_allowed();
            if count == 0 {
                return None;
            } else if count == 1 {
                board[coord] = Space::Full(allowed.allowed().next().unwrap());
                continue 'look_for_forces;
            }
        }
        // checked the whole board, didn't fill any squares
        // so we're done
        return Some(board);
    }
}

fn pick_open_space(board: &Board) -> Option<(Coord, AllowedNumbers)> {
    let all_spaces = Coord::all();
    let open_spaces = all_spaces.filter(|c| board[*c] == Space::Empty);
    let mut spaces_and_allowed = open_spaces.map(|c| (c, board.allowed(c)));
    spaces_and_allowed.next()
    //spaces_and_allowed.min_by_key(|(_, a)| a.count_allowed())
}

pub fn solve(mut board: Board) -> Option<Board> {
    board = make_forced_choices(board)?;
    let open_space = pick_open_space(&board);
    let (space, allowed) = match open_space {
        None => return Some(board),
        Some(best) => best
    };

    for possibility in allowed.allowed() {
        let mut possible_board = board.clone();
        possible_board[space] = Space::Full(possibility);
        if let Some(solved) = solve(possible_board) {
            return Some(solved);
        }
    }
    return None;
}

fn main() {
    let mut board = Board::new();
    board[Coord::new(3, 0)] = Space::Full(Number::new(8));
    board[Coord::new(5, 0)] = Space::Full(Number::new(1));
    board[Coord::new(7, 1)] = Space::Full(Number::new(4));
    board[Coord::new(8, 1)] = Space::Full(Number::new(3));
    board[Coord::new(0, 2)] = Space::Full(Number::new(5));
    board[Coord::new(4, 3)] = Space::Full(Number::new(7));
    board[Coord::new(6, 3)] = Space::Full(Number::new(8));
    board[Coord::new(6, 4)] = Space::Full(Number::new(1));
    board[Coord::new(1, 5)] = Space::Full(Number::new(2));
    board[Coord::new(4, 5)] = Space::Full(Number::new(3));
    board[Coord::new(0, 6)] = Space::Full(Number::new(6));
    board[Coord::new(7, 6)] = Space::Full(Number::new(7));
    board[Coord::new(8, 6)] = Space::Full(Number::new(5));
    board[Coord::new(2, 7)] = Space::Full(Number::new(3));
    board[Coord::new(3, 7)] = Space::Full(Number::new(4));
    board[Coord::new(3, 8)] = Space::Full(Number::new(2));
    board[Coord::new(6, 8)] = Space::Full(Number::new(6));
    println!("{}", &board);
    board = solve(board).unwrap();
    println!("{}", &board);
}
