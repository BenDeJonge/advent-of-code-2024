use std::collections::HashSet;

use crate::util::Matrix;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn clockwise(&self) -> Direction {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Guard {
    position: [usize; 2],
    direction: Direction,
}

impl Guard {
    pub fn rotate(&mut self) {
        self.direction = self.direction.clockwise()
    }

    pub fn peek(&self, bounds: [usize; 2]) -> Option<[usize; 2]> {
        let dest = match self.direction {
            Direction::North => [self.position[0].checked_sub(1), Some(self.position[1])],
            Direction::East => [Some(self.position[0]), self.position[1].checked_add(1)],
            Direction::South => [self.position[0].checked_add(1), Some(self.position[1])],
            Direction::West => [Some(self.position[0]), self.position[1].checked_sub(1)],
        };
        if dest[0].is_some_and(|val| val < bounds[0]) && dest[1].is_some_and(|val| val < bounds[1])
        {
            return Some([dest[0].unwrap(), dest[1].unwrap()]);
        }
        None
    }
}

const CHAR_EMPTY: char = '.';
const CHAR_OCCUPIED: char = '#';
const CHAR_GUARD: char = '^';

pub fn parse_input(input: &str) -> (Matrix<bool>, Guard) {
    let mut guard = Guard {
        position: [0, 0],
        direction: Direction::North,
    };
    let mut matrix = vec![];
    for (row, line) in input.lines().enumerate() {
        let mut vec: Vec<bool> = Vec::with_capacity(line.len());
        for (col, char) in line.chars().enumerate() {
            match char {
                CHAR_EMPTY => vec.push(false),
                CHAR_OCCUPIED => vec.push(true),
                CHAR_GUARD => {
                    vec.push(false);
                    guard.position = [row, col];
                }
                _ => unreachable!(),
            }
        }
        matrix.push(vec);
    }
    (Matrix::new(matrix), guard)
}

fn visits(matrix: &Matrix<bool>, guard: &mut Guard) -> HashSet<[usize; 2]> {
    let mut visited = HashSet::from([guard.position]);
    loop {
        if let Some(next_position) = guard.peek(matrix.shape()) {
            match matrix[next_position[0]][next_position[1]] {
                // Guard cannot move there.
                true => {
                    guard.rotate();
                }
                false => {
                    visited.insert(next_position);
                    guard.position = next_position;
                }
            }
        } else {
            return visited;
        }
    }
}

/// The number of unique squares the guard will visit.
pub fn part_1(matrix: &Matrix<bool>, guard: &mut Guard) -> usize {
    visits(matrix, guard).len()
}

/// The number of loops the guard can get stuck in by adding a single obstacle.
pub fn part_2(matrix: &mut Matrix<bool>, guard: &mut Guard) -> usize {
    let mut obstacles = 0;
    let position_original = guard.position;
    let direction_orginal = guard.direction;

    // The guard would not normally visit this position so any obstacle
    // placed there would not be encountered anyway.
    let mut visited = visits(matrix, guard);
    // The guard would notice placing an obstacle on his position.
    visited.remove(&position_original);
    let mut visited_with_obstacle = HashSet::new();
    for [row, col] in visited {
        // A valid obstacle position.
        matrix[row][col] = true;
        guard.position = position_original;
        guard.direction = direction_orginal;
        visited_with_obstacle.insert((guard.direction, guard.position));
        while let Some(next_position) = guard.peek(matrix.shape()) {
            match matrix[next_position[0]][next_position[1]] {
                // Guard cannot move there.
                true => {
                    guard.rotate();
                }
                false => {
                    guard.position = next_position;
                    // The guard is stuck in a loop.
                    if visited_with_obstacle.contains(&(guard.direction, guard.position)) {
                        obstacles += 1;
                        break;
                    } else {
                        // The guard moves to a vacant square.
                        visited_with_obstacle.insert((guard.direction, guard.position));
                    }
                }
            }
        }
        // Undoing the obstacle.
        matrix[row][col] = false;
        visited_with_obstacle.clear();
    }
    obstacles
}

#[cfg(test)]
mod tests {

    use super::{parse_input, part_1, part_2};
    use crate::{
        day06::{Direction, Guard},
        util::{read_file_to_string, Matrix},
    };
    const INPUT: &str = "....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT),
            (
                Matrix::new(vec![
                    vec![false, false, false, false, true, false, false, false, false, false],
                    vec![false, false, false, false, false, false, false, false, false, true],
                    vec![false, false, false, false, false, false, false, false, false, false],
                    vec![false, false, true, false, false, false, false, false, false, false],
                    vec![false, false, false, false, false, false, false, true, false, false],
                    vec![false, false, false, false, false, false, false, false, false, false],
                    vec![false, true, false, false, false, false, false, false, false, false],
                    vec![false, false, false, false, false, false, false, false, true, false],
                    vec![true, false, false, false, false, false, false, false, false, false],
                    vec![false, false, false, false, false, false, true, false, false, false],
                ]),
                Guard {
                    position: [6, 4],
                    direction: Direction::North
                }
            )
        )
    }

    #[test]
    fn test_part_1_small() {
        let (matrix, mut guard) = parse_input(INPUT);
        assert_eq!(part_1(&matrix, &mut guard), 41)
    }

    #[test]
    fn test_part_1_full() {
        let (matrix, mut guard) = parse_input(&read_file_to_string("data/day06.txt"));
        assert_eq!(part_1(&matrix, &mut guard), 4696)
    }

    #[test]
    fn test_part_2_small() {
        let (mut matrix, mut guard) = parse_input(INPUT);
        assert_eq!(part_2(&mut matrix, &mut guard), 6)
    }

    #[test]
    fn test_part_2_full() {
        let (mut matrix, mut guard) = parse_input(&read_file_to_string("data/day06.txt"));
        assert_eq!(part_2(&mut matrix, &mut guard), 1443)
    }
}
