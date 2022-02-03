use super::{Number, Space, Coord, Board};

#[derive(Copy, Clone)]
struct AllowedNumbers(u16);

impl AllowedNumbers {
    fn all() -> Self {
        // trailing zero b/c numbers start at one
        Self(0b111_111_111_0)
    }

    fn get_mask(num: Number) -> u16 {
        1 << num.get()
    }

    fn disallow(&mut self, num: Number) {
        self.0 &= !Self::get_mask(num);
    }

    fn allowed(&self) -> AllowedNumbersIterator {
        AllowedNumbersIterator(self.0)
    }
}

struct AllowedNumbersIterator(u16);

impl AllowedNumbersIterator {
    fn are_none_allowed(&self) -> bool {
        self.0 == 0
    }

    fn is_one_allowed(&self) -> bool {
        self.0 != 0 && self.0 & (self.0 - 1) == 0
    }
}

impl Iterator for AllowedNumbersIterator {
    type Item = Number;
    
    fn next(&mut self) -> Option<Number> {
        if self.0 == 0 {
            return None;
        }
        let lowest_set_bit = self.0.trailing_zeros();
        // clear lowest bit
        self.0 &= self.0 - 1;
        // guaranteed non-zero, so this is never none
        Number::safe_new(lowest_set_bit as u8)
        // or do:
        // Some(Number::new(lowest_set_bit as u8))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.count_ones() as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for AllowedNumbersIterator {}

impl std::ops::BitAnd for AllowedNumbers {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

struct CoordBitsetIterator(u128);

impl Iterator for CoordBitsetIterator {
    type Item = Coord;
    
    fn next(&mut self) -> Option<Coord> {
        if self.0 == 0 {
            return None;
        }
        let lowest_set_bit = self.0.trailing_zeros();
        // clear lowest bit
        self.0 &= self.0 - 1;
        Some(Coord(lowest_set_bit as u8))
    }
}

#[derive(Copy, Clone)]
struct BookkeptBoard {
    board: Board,
    open_spaces: u128,
    updated_allows: u128,
    col_allowed: [AllowedNumbers; 9],
    row_allowed: [AllowedNumbers; 9],
    box_allowed: [AllowedNumbers; 9],
}

impl BookkeptBoard {
    fn new() -> Self {
        Self {
            board: Board::new(),
            open_spaces: (1 << 81) - 1,
            updated_allows: 0,
            col_allowed: [AllowedNumbers::all(); 9],
            row_allowed: [AllowedNumbers::all(); 9],
            box_allowed: [AllowedNumbers::all(); 9]
        }
    }
    
    fn from_board(board: Board) -> Self {
        let mut res = Self::new();
        for coord in Coord::all() {
            if let Space::Full(num) = board[coord] {
                res.fill(coord, num);
            }
        }
        res
    }

    fn fill(&mut self, coord: Coord, num: Number) {
        assert!(self.board[coord] == Space::Empty);
            
        self.board[coord] = Space::Full(num);
            
        self.open_spaces &= !(1 << coord.0);
        
        self.add_col_updates(coord);
        self.add_row_updates(coord);
        self.add_box_updates(coord);
        self.updated_allows &= self.open_spaces;

        self.col_allowed[coord.x() as usize].disallow(num);
        self.row_allowed[coord.y() as usize].disallow(num);
        self.box_allowed[coord.square() as usize].disallow(num);
    }

    fn add_col_updates(&mut self, coord: Coord) {
        const COL_MASK: u128 = 
            0b000_000_001 << 72 |
            0b000_000_001 << 63 |
            0b000_000_001 << 54 |
            0b000_000_001 << 45 |
            0b000_000_001 << 36 |
            0b000_000_001 << 27 |
            0b000_000_001 << 18 |
            0b000_000_001 << 9  |
            0b000_000_001;
        // let this_col_mask = COL_MASK << coord.x();
        const COL_MASKS: [u128; 9] = [COL_MASK, COL_MASK << 1, COL_MASK << 2, COL_MASK << 3, COL_MASK << 4, COL_MASK << 5, COL_MASK << 6, COL_MASK << 7, COL_MASK << 8];
        let this_col_mask = COL_MASKS[coord.x() as usize];
        self.updated_allows |= this_col_mask;
    }
    
    fn add_row_updates(&mut self, coord: Coord) {
        const ROW_MASK: u128 = 0b111_111_111;
        // let this_row_mask = ROW_MASK << (coord.y() * 9);
        const ROW_MASKS: [u128; 9] = [ROW_MASK, ROW_MASK << 9, ROW_MASK << 18, ROW_MASK << 27, ROW_MASK << 36, ROW_MASK << 45, ROW_MASK << 54, ROW_MASK << 63, ROW_MASK << 72];
        let this_row_mask = ROW_MASKS[coord.y() as usize];
        self.updated_allows |= this_row_mask;
    }

    fn add_box_updates(&mut self, coord: Coord) {
        const BOX_MASK: u128 = 
            0b000_000_111 << 18 |
            0b000_000_111 << 9  |
            0b000_000_111;
        let shift = 3 * (coord.x() / 3) + 27 * (coord.y() / 3);
        let this_box_mask = BOX_MASK << shift;
        self.updated_allows |= this_box_mask;
    }

    fn open_spaces(&self) -> impl Iterator<Item=Coord> {
        CoordBitsetIterator(self.open_spaces)
    }

    fn any_updates(&self) -> bool {
        self.updated_allows != 0
    }
    
    fn take_updates(&mut self) -> impl Iterator<Item=Coord> {
        let updated = self.updated_allows;
        self.updated_allows = 0;
        CoordBitsetIterator(updated)
    }

    fn allowed(&self, coord: Coord) -> AllowedNumbersIterator {
        let mut res = self.col_allowed[coord.x() as usize];
        res = res & self.row_allowed[coord.y() as usize];
        res = res & self.box_allowed[coord.square() as usize];
        res.allowed()
    }
}

impl std::ops::Index<Coord> for BookkeptBoard {
    type Output = Space;
    fn index(&self, index: Coord) -> &Space {
        &self.board[index]
    }
}

fn make_forced_choices(mut board: BookkeptBoard) -> Option<BookkeptBoard> {
    while board.any_updates() {
        for update in board.take_updates() {
            let mut allowed = board.allowed(update);
            if allowed.are_none_allowed() {
                return None;
            } else if allowed.is_one_allowed() {
                let num = allowed.next().unwrap();
                board.fill(update, num);
            }
        }
    }
    // checked the whole board, didn't fill any squares
    // so we're done
    return Some(board);
}

fn pick_open_space(board: &BookkeptBoard) -> Option<Coord> {
    // i want to pick one with the least possibilities
    // but not sure how to do it fast
    board.open_spaces().next()
}


fn solve_helper(mut board: BookkeptBoard) -> Option<BookkeptBoard> {
    board = make_forced_choices(board)?;
    let open_space = match pick_open_space(&board) {
        // no open spaces means we're done
        None => return Some(board),
        Some(space) => space
    };
    let allowed = board.allowed(open_space);

    for possibility in allowed {
        let mut possible_board = board;
        possible_board.fill(open_space, possibility);
        if let Some(solved) = solve_helper(possible_board) {
            return Some(solved);
        }
    }
    return None;
}

pub fn solve(board: Board) -> Option<Board> {
    let board = BookkeptBoard::from_board(board);
    let res = solve_helper(board)?;
    Some(res.board)
}