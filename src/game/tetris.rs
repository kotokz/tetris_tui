use std::{collections::HashSet, mem};

use super::shape::{Cell, Piece, Shape};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

pub trait Tetris {
    fn tick(&mut self);
    fn rotate(&mut self);
    fn get(&self, cell: Cell) -> Option<Shape>;
    fn shift(&mut self, direction: Direction);
    fn alive(&self) -> bool;
    fn board_size(&self) -> (i32, i32);
}
pub struct TetrisBoard {
    width: i32,
    height: i32,
    current_piece: Piece,
    landed_pieces: Vec<Piece>,
    alive: bool,
}

impl Tetris for TetrisBoard {
    fn board_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn tick(&mut self) {
        if !self.alive {
            return;
        }

        let advanced_piece = &self.current_piece + Cell(0, 1);

        if self.is_out_of_bounds(&advanced_piece) || self.is_colliding(&advanced_piece) {
            let landed_piece = mem::replace(
                &mut self.current_piece,
                &Piece::random_piece() + Cell((self.width - 1) / 2, 0),
            );

            self.landed_pieces.push(landed_piece);
            self.remove_full_lines();

            if self.is_colliding(&self.current_piece) {
                self.alive = false;
            }
        } else {
            self.current_piece = advanced_piece;
        }
    }

    fn get(&self, cell: Cell) -> Option<Shape> {
        if self.current_piece.has_position(cell) {
            Some(self.current_piece.shape())
        } else {
            self.landed_pieces
                .iter()
                .find(|piece| piece.has_position(cell))
                .map(|piece| piece.shape())
        }
    }

    fn shift(&mut self, direction: Direction) {
        if !self.alive {
            return;
        }

        let shifted_piece = &self.current_piece
            + match direction {
                Direction::Left => Cell(-1, 0),
                Direction::Right => Cell(1, 0),
            };

        if !self.is_out_of_bounds(&shifted_piece) && !self.is_colliding(&shifted_piece) {
            self.current_piece = shifted_piece;
        }
    }

    fn rotate(&mut self) {
        if !self.alive {
            return;
        }

        let rotated_piece = self.current_piece.rotate();

        if !self.is_out_of_bounds(&rotated_piece) && !self.is_colliding(&rotated_piece) {
            self.current_piece = rotated_piece;
        }
    }

    fn alive(&self) -> bool {
        self.alive
    }
}

impl TetrisBoard {
    fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            current_piece: &Piece::random_piece() + Cell((width - 1) / 2, 0),
            landed_pieces: vec![],
            alive: true,
        }
    }

    pub fn new_default() -> Self {
        Self::new(10, 20)
    }

    fn is_out_of_bounds(&self, piece: &Piece) -> bool {
        !piece
            .iter_positions()
            .all(|cell| 0 <= cell.0 && cell.0 < self.width && 0 <= cell.1 && cell.1 < self.height)
    }

    fn is_colliding(&self, piece: &Piece) -> bool {
        self.landed_pieces
            .iter()
            .any(|cell| cell.collides_with(piece))
    }

    fn is_line_full(&self, y: i32) -> bool {
        self.landed_pieces
            .iter()
            .flat_map(|piece| piece.iter_positions())
            .filter(|cell| cell.1 == y)
            .collect::<HashSet<_>>()
            .len() as i32
            == self.width
    }

    fn remove_line(&mut self, y: i32) {
        for piece in self.landed_pieces.iter_mut() {
            piece.remove_cell(y);
        }
    }

    fn remove_full_lines(&mut self) {
        for y in 0..self.height {
            if self.is_line_full(y) {
                self.remove_line(y);
            }
        }
    }
}
