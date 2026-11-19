use std::fmt;

use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    U,
    R,
    F,
    D,
    L,
    B,
}

use Face::{B, D, F, L, R, U};
const FACES: [Face; 6] = [U, R, F, D, L, B];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Turn {
    Clockwise,
    Half,
    Counterclockwise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move(Face, Turn);

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = match self.1 {
            Turn::Clockwise => "",
            Turn::Half => "2",
            Turn::Counterclockwise => "'",
        };
        write!(f, "{:?}{suffix}", self.0)
    }
}

pub fn scramble(rng: &mut impl Rng) -> [Move; 25] {
    let turns = [Turn::Clockwise, Turn::Half, Turn::Counterclockwise];
    let mut previous = None;
    std::array::from_fn(|_| {
        let index = match previous {
            None => rng.random_range(0..FACES.len()),
            Some(excluded) => {
                let index = rng.random_range(0..FACES.len() - 1);
                index + usize::from(index >= excluded)
            }
        };
        previous = Some(index);
        Move(FACES[index], turns[rng.random_range(0..turns.len())])
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cube([[Face; 9]; 6]);

type Strip = (Face, [usize; 3]);

impl Cube {
    pub fn from_moves(moves: &[Move]) -> Self {
        let mut cube = Self(FACES.map(|face| [face; 9]));
        for &Move(face, turn) in moves {
            let quarters = match turn {
                Turn::Clockwise => 1,
                Turn::Half => 2,
                Turn::Counterclockwise => 3,
            };
            for _ in 0..quarters {
                cube.turn_clockwise(face);
            }
        }
        cube
    }

    pub fn face(&self, face: Face) -> &[Face; 9] {
        &self.0[face as usize]
    }

    fn turn_clockwise(&mut self, face: Face) {
        // Facelets are numbered left to right, top to bottom, as seen from outside the cube.
        // Each clockwise quarter-turn sends a neighboring strip to the next one in its cycle.
        let cycle: [Strip; 4] = match face {
            U => [
                (F, [0, 1, 2]),
                (L, [0, 1, 2]),
                (B, [0, 1, 2]),
                (R, [0, 1, 2]),
            ],
            R => [
                (U, [2, 5, 8]),
                (B, [6, 3, 0]),
                (D, [2, 5, 8]),
                (F, [2, 5, 8]),
            ],
            F => [
                (U, [6, 7, 8]),
                (R, [0, 3, 6]),
                (D, [2, 1, 0]),
                (L, [8, 5, 2]),
            ],
            D => [
                (F, [6, 7, 8]),
                (R, [6, 7, 8]),
                (B, [6, 7, 8]),
                (L, [6, 7, 8]),
            ],
            L => [
                (U, [0, 3, 6]),
                (F, [0, 3, 6]),
                (D, [0, 3, 6]),
                (B, [8, 5, 2]),
            ],
            B => [
                (U, [0, 1, 2]),
                (L, [6, 3, 0]),
                (D, [8, 7, 6]),
                (R, [2, 5, 8]),
            ],
        };
        let before = self.0;
        self.0[face as usize] = [6, 3, 0, 7, 4, 1, 8, 5, 2].map(|i| before[face as usize][i]);
        for ((source, from), (target, to)) in
            cycle.into_iter().zip(cycle.into_iter().cycle().skip(1))
        {
            for (source_index, target_index) in from.into_iter().zip(to) {
                self.0[target as usize][target_index] = before[source as usize][source_index];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;

    #[test]
    fn generated_scrambles_have_valid_notation_and_never_repeat_a_face() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..100 {
            let moves = scramble(&mut rng);
            assert_eq!(moves.len(), 25);
            assert!(moves.windows(2).all(|pair| pair[0].0 != pair[1].0));
            for turn in moves {
                let notation = turn.to_string();
                assert!("URFDLB".contains(&notation[..1]));
                assert!(matches!(&notation[1..], "" | "'" | "2"));
            }
        }
    }

    #[test]
    fn a_clockwise_front_turn_matches_standard_cube_notation() {
        let cube = Cube::from_moves(&[Move(F, Turn::Clockwise)]);
        assert_eq!(*cube.face(U), [U, U, U, U, U, U, L, L, L]);
        assert_eq!(*cube.face(R), [U, R, R, U, R, R, U, R, R]);
        assert_eq!(*cube.face(D), [R, R, R, D, D, D, D, D, D]);
        assert_eq!(*cube.face(L), [L, L, D, L, L, D, L, L, D]);
        assert_eq!(*cube.face(F), [F; 9]);
        assert_eq!(*cube.face(B), [B; 9]);
    }

    #[test]
    fn a_mixed_scramble_matches_the_original_cube_net() {
        use Turn::{Clockwise as Cw, Counterclockwise as Ccw, Half};

        // Reference facelets from cuub a946358, read in URFDLB order.
        let cube = Cube::from_moves(&[
            Move(R, Cw),
            Move(U, Cw),
            Move(R, Ccw),
            Move(U, Ccw),
            Move(F, Half),
            Move(D, Cw),
            Move(B, Ccw),
            Move(L, Half),
        ]);
        let expected = [
            [D, L, B, D, U, F, F, D, D],
            [L, R, U, L, R, U, D, F, L],
            [L, F, F, R, F, F, R, L, R],
            [B, D, F, U, D, U, R, R, U],
            [B, B, U, B, L, D, U, L, D],
            [R, B, L, R, B, U, B, B, F],
        ];
        assert_eq!(cube.0, expected);
    }

    #[test]
    fn a_scramble_and_its_inverse_restore_the_cube() {
        let moves = scramble(&mut StdRng::seed_from_u64(42));
        let scrambled = Cube::from_moves(&moves);
        let solved = Cube::from_moves(&[]);
        assert_ne!(scrambled, solved);
        for face in FACES {
            assert_eq!(scrambled.face(face)[4], face);
            assert_eq!(
                scrambled.0.iter().flatten().filter(|&&x| x == face).count(),
                9
            );
        }
        let inverse = moves.iter().rev().map(|&Move(face, turn)| {
            Move(
                face,
                match turn {
                    Turn::Clockwise => Turn::Counterclockwise,
                    Turn::Half => Turn::Half,
                    Turn::Counterclockwise => Turn::Clockwise,
                },
            )
        });
        assert_eq!(
            Cube::from_moves(&moves.into_iter().chain(inverse).collect::<Vec<_>>()),
            solved
        );
    }
}
