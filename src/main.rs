use std::io::BufRead;

mod sudoku;
use sudoku::{Board, Coord, Space, Number, solve};

fn from_file(p: &str) -> Board {
    let f = std::fs::File::open(p).unwrap();
    let f = std::io::BufReader::new(f);
    let mut b = Board::new();
    
    let mut y = 0;
    for line in f.lines() {
        let line = line.unwrap();
        if line == "---+---+---" {
            continue;
        }

        let mut x = 0;
        for char in line.chars() {
            if char == '|' {
                continue;
            }
            if char != ' ' {
                let n = char.to_digit(10).unwrap() as u8;
                b[Coord::new(x, y)] = Space::Full(Number::new(n));
            }
            x += 1;
        }
        y += 1;
    }
    return b
}

fn main() {
    let board = from_file("test1.txt");
    println!("{}", &board);
    let start = std::time::Instant::now();
    let mut board2 = Board::new();
    for _ in 0..10000 {
        board2 = solve(board).unwrap();
    }
    println!("{}", &board2);
    println!("{:?}", start.elapsed());
}
