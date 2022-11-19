#![allow(unused)]

use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use std::fmt;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
enum Tile {
    X = 0,
    O = 1,
    Empty = 2,
}

impl Tile {
    fn opponent(&self) -> Tile {

        match self {
            Self::X => Self::O,
            Self::O => Self::X,
            Self::Empty => panic!("Called opponent on empty tile"),
        }
    }

    fn opponent_iter() -> TileOpponentIter {
        TileOpponentIter::new()
    }
}

struct TileOpponentIter {
    cur: Tile,
}

impl TileOpponentIter {
    fn new() -> Self {
        Self { cur: Tile::Empty }
    }
}

impl Iterator for TileOpponentIter {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = match self.cur {
            Tile::Empty => Some(Tile::X),
            Tile::X => Some(Tile::O),
            Tile::O => None,
        };

        self.cur = match self.cur {
            Tile::Empty => Tile::X,
            Tile::X | Tile::O => Tile::O,
        };

        ret
    }
}

impl From<u8> for Tile {
    fn from(state: u8) -> Self {
        FromPrimitive::from_u8(state).expect("Invalid state")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2 {
    x: isize,
    y: isize,
}

impl Vec2 {
    pub fn new(x: isize, y: isize) -> Vec2 {
        Vec2 { x, y }
    }

    fn is_in_board(&self) -> bool {
        (0..8).contains(&self.x) && (0..8).contains(&self.y)
    }

    fn board_iter() -> Vec2BoardIter {
        Vec2BoardIter { cur: Vec2::new(0, 0) }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", b'a' + self.x as u8, self.y)
    }
}

struct Vec2BoardIter {
    cur: Vec2,
}

impl Iterator for Vec2BoardIter {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == Vec2::new(7, 7) {
            return None;
        }

        if self.cur.y == 7 {
            self.cur.y = 0;
            self.cur.x += 1;
        }
        else {
            self.cur.y += 1;
        }

        Some(self.cur)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    state: u128,
}

impl Board {
    pub fn empty() -> Board {
        Board { state: 0 }
    }

    pub fn new() -> Board {
        let mut board = Board::empty();

        board.set(Vec2::new(3, 3), Tile::O);
        board.set(Vec2::new(3, 4), Tile::X);
        board.set(Vec2::new(4, 3), Tile::X);
        board.set(Vec2::new(4, 4), Tile::O);

        board
    }

    fn get_raw_place(&self, place: usize) -> Tile {
        Tile::from(((self.state >> place) & 0b11) as u8)
    }

    fn set_raw_place(&mut self, place: usize, tile: Tile) {
        self.state &= !(0b11 << place);
        self.state |= (tile as u128) << place;
    }

    fn raw_place(place: Vec2) -> usize {
        (place.x * 8 + place.y) as usize
    }

    fn get(&self, pos: Vec2) -> Tile {
        self.get_raw_place(Self::raw_place(pos))
    }

    fn set(&mut self, pos: Vec2, tile: Tile) {
        self.set_raw_place(Self::raw_place(pos), tile);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    board: Board,
    next_player: Tile,
}

impl Pos {
    pub fn new() -> Pos {
        Pos { board: Board::new(), next_player: Tile::X }
    }

    pub fn place(&mut self, place: Vec2) -> bool {
        self.board.set(place, self.next_player);

        let mut flipped = false;

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let step = Vec2::new(dx, dy);
                let mut cur = place + step;

                while cur.is_in_board() && self.board.get(cur) == self.next_player.opponent() {
                    cur += step;
                }

                if cur.is_in_board() && self.board.get(cur) == self.next_player {
                    loop {
                        cur -= step;

                        if cur == place {
                            break;
                        }

                        self.board.set(cur, self.next_player);
                        flipped = true;
                    }
                }
            }
        }

        self.next_player = self.next_player.opponent();

        flipped
    }

    pub fn is_game_over(&self) -> bool {
        for place in Vec2::board_iter() {
            if self.board.get(place) != Tile::Empty {
                continue;
            }
            
            for player in Tile::opponent_iter() {
                let mut tester = self.clone();
                tester.next_player = player;

                if tester.place(place) {
                    return false;
                }
            }
        }

        true
    }

    pub fn valid_moves(&self) -> Vec<Vec2> {
        let mut ret = Vec::new();

        for place in Vec2::board_iter() {
            if self.board.get(place) != Tile::Empty {
                continue;
            }

            if self.clone().place(place) {
                ret.push(place);
            }
        }
        
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_place_simple() {
        let mut a = Pos::new();
        a.place(Vec2::new(3, 2));
        
        let mut b = Pos::new();
        b.board.set(Vec2::new(3, 2), Tile::X);
        b.board.set(Vec2::new(3, 3), Tile::X);
        b.next_player = Tile::O;

        assert_eq!(a, b);
    }
}
