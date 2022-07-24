use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{collections::HashSet, ops::Add};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Shape {
    I = 0,
    O,
    T,
    J,
    L,
    S,
    Z,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Cell(pub i32, pub i32);

impl Add for Cell {
    type Output = Cell;

    fn add(self, rhs: Self) -> Self::Output {
        Cell(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl From<(i32, i32)> for Cell {
    fn from(lhs: (i32, i32)) -> Self {
        Cell(lhs.0, lhs.1)
    }
}

#[derive(Debug)]
pub struct Piece {
    shape: Shape,
    positions: HashSet<Cell>,
    pivot: Cell,
}

impl Distribution<Piece> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Piece {
        let shape = match rng.gen_range(0..=6) {
            0 => Shape::I,
            1 => Shape::O,
            2 => Shape::T,
            3 => Shape::J,
            4 => Shape::L,
            5 => Shape::S,
            _ => Shape::Z,
        };
        Piece::new(shape)
    }
}

impl Piece {
    pub fn new(shape: Shape) -> Self {
        match shape {
            Shape::I => Self::new_piece(
                Shape::I,
                [Cell(0, 0), Cell(1, 0), Cell(2, 0), Cell(3, 0)],
                Cell(1, 0),
            ),
            Shape::O => Self::new_piece(
                Shape::O,
                [Cell(0, 0), Cell(1, 0), Cell(0, 1), Cell(1, 1)],
                Cell(0, 0),
            ),
            Shape::T => Self::new_piece(
                Shape::T,
                [Cell(0, 0), Cell(1, 0), Cell(2, 0), Cell(1, 1)],
                Cell(1, 0),
            ),
            Shape::J => Self::new_piece(
                Shape::J,
                [Cell(0, 0), Cell(0, 1), Cell(0, 2), Cell(-1, 2)],
                Cell(0, 1),
            ),
            Shape::L => Self::new_piece(
                Shape::L,
                [Cell(0, 0), Cell(0, 1), Cell(0, 2), Cell(1, 2)],
                Cell(0, 1),
            ),
            Shape::S => Self::new_piece(
                Shape::S,
                [Cell(0, 0), Cell(1, 0), Cell(0, 1), Cell(-1, 1)],
                Cell(0, 0),
            ),
            Shape::Z => Self::new_piece(
                Shape::Z,
                [Cell(0, 0), Cell(-1, 0), Cell(0, 1), Cell(1, 1)],
                Cell(0, 0),
            ),
        }
    }

    fn new_piece(s: Shape, cells: [Cell; 4], p: Cell) -> Self {
        Self {
            shape: s,
            positions: cells.into_iter().collect(),
            pivot: p,
        }
    }

    pub fn random_piece() -> Self {
        rand::random()
    }

    pub fn shape(&self) -> Shape {
        self.shape
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = Cell> + '_ {
        self.positions.iter().copied()
    }

    pub fn has_position(&self, cell: Cell) -> bool {
        self.positions.contains(&cell)
    }

    pub fn collides_with(&self, other: &Piece) -> bool {
        self.positions.intersection(&other.positions).count() > 0
    }

    //https://gamedev.stackexchange.com/a/17976
    pub fn rotate(&self) -> Self {
        let Cell(a, b) = self.pivot;

        Self {
            shape: self.shape,
            positions: self
                .iter_positions()
                .map(|Cell(x, y)| Cell(a + b - y, b - a + x))
                .collect(),
            pivot: self.pivot,
        }
    }

    pub fn remove_cell(&mut self, y: i32) {
        self.positions = self
            .positions
            .iter()
            .copied()
            .filter(|pos| pos.1 != y)
            .map(|cell| {
                if cell.1 >= y {
                    cell
                } else {
                    Cell(cell.0, cell.1 + 1)
                }
            })
            .collect();
    }
}

impl Add<Cell> for &Piece {
    type Output = Piece;

    fn add(self, rhs: Cell) -> Self::Output {
        Piece {
            shape: self.shape,
            positions: self.positions.iter().map(|&pos| pos + rhs).collect(),
            pivot: self.pivot + rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let piece_i = Piece::new(Shape::I);

        assert_eq!(piece_i.shape, Shape::I);

        println!("{:#?}", piece_i);
    }

    #[test]
    fn test_random() {
        let piece_random = Piece::random_piece();
        println!("{:#?}", piece_random);
    }

    #[test]
    fn test_rotate() {
        let piece_i = Piece::new(Shape::I);

        let rotated_i = piece_i.rotate();

        assert_eq!(rotated_i.pivot, piece_i.pivot);

        assert_eq!(
            rotated_i.positions,
            [Cell(1, -1), Cell(1, 1), Cell(1, 0), Cell(1, 2)]
                .into_iter()
                .collect::<HashSet<Cell>>()
        );

        for shape in [
            Shape::I,
            Shape::J,
            Shape::L,
            Shape::O,
            Shape::S,
            Shape::T,
            Shape::Z,
        ]
        .into_iter()
        {
            let piece = Piece::new(shape);
            let roated_piece = piece.rotate().rotate().rotate().rotate();
            assert_eq!(piece.positions, roated_piece.positions);
        }
    }
}
