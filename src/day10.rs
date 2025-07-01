use std::collections::{HashMap, HashSet};

use crate::util::{Coordinate, Matrix, COORDINATE_OFFSETS_NESW};

struct EvaluationState {
    reachable: HashMap<Coordinate, HashSet<Coordinate>>,
    trailheads: HashSet<Coordinate>,
    n_trails: usize,
}

impl EvaluationState {
    pub fn new() -> Self {
        EvaluationState {
            reachable: HashMap::<Coordinate, HashSet<Coordinate>>::new(),
            trailheads: HashSet::<Coordinate>::new(),
            n_trails: 0,
        }
    }
}

/// Recursively construct inclining paths from a starting position.
///
/// For a more optimal solution, we might need to check if a visited coordinate
/// does not have any known solutions.
///
/// if let Some(hashed) = state.reachable.get(current_coord).cloned() {
///    state.trailheads.insert(trail[0]);
///    state
///        .reachable
///        .entry(trail[0])
///        .and_modify(|peaks| {
///            peaks.extend(hashed.iter());
///         })
///         .or_insert(hashed);
///     return;
/// }
fn evaluate_coordinate(
    current_coord: &Coordinate,
    current_val: u8,
    trail: &mut Vec<Coordinate>,
    matrix: &Matrix<u8>,
    bounds: &[&Coordinate; 2],
    state: &mut EvaluationState,
) {
    for offset in COORDINATE_OFFSETS_NESW {
        let neighbor_coord = *current_coord + offset;
        if !neighbor_coord.is_in(bounds[0], bounds[1]) {
            continue;
        }
        let neighbor_val = matrix[neighbor_coord.r as usize][neighbor_coord.c as usize];
        if neighbor_val != current_val + 1 {
            continue;
        }
        trail.push(neighbor_coord);
        if trail.len() == 10 {
            state.n_trails += 1;
            state.trailheads.insert(trail[0]);
            for coord in trail.iter() {
                state
                    .reachable
                    .entry(*coord)
                    .and_modify(|peaks| {
                        peaks.insert(neighbor_coord);
                    })
                    .or_insert(HashSet::from([neighbor_coord]));
            }
        } else {
            evaluate_coordinate(&neighbor_coord, neighbor_val, trail, matrix, bounds, state);
        }
        trail.pop();
    }
}

/// Loop over all coordinates and recursively construct inclining paths from all
/// 0-height starting positions.
fn solve(matrix: &Matrix<u8>) -> EvaluationState {
    let mut state = EvaluationState::new();
    let bounds = [
        &Coordinate::new(0, 0),
        &Coordinate::new(matrix.shape()[0] as isize, matrix.shape()[1] as isize),
    ];
    let mut trail = Vec::<Coordinate>::with_capacity(10);
    for row in 0..matrix.shape()[0] {
        for col in 0..matrix.shape()[1] {
            let current_val = matrix[row][col];
            if current_val != 0 {
                continue;
            }
            let current_coord = Coordinate::new(row as isize, col as isize);
            // We explore a new trail from this position.
            trail.clear();
            trail.push(current_coord);
            evaluate_coordinate(
                &current_coord,
                current_val,
                &mut trail,
                matrix,
                &bounds,
                &mut state,
            );
        }
    }
    state
}

pub fn parse_input(input: &str) -> Matrix<u8> {
    let mut data = vec![];
    for line in input.lines() {
        let mut row = Vec::with_capacity(line.len());
        for byte in line.bytes() {
            // Digit 0 is represented by 0x30.
            row.push(byte - 0x30);
        }
        data.push(row);
    }
    Matrix::new(data)
}

/// Compute the sum of all trailhead scores.
/// Any element in the matrix is a trailhead if:
/// - it has the value 0.
/// - it has at least 1 continuous path from itself to a 9-value element. Moves
///   can only occur in the four cardinal directions North, East, South and West.
///   The score of a trailhead equals the number of acceptable paths.
pub fn part_1(matrix: &Matrix<u8>) -> usize {
    let state = solve(matrix);
    state
        .trailheads
        .iter()
        .filter_map(|coord| state.reachable.get(coord))
        .map(|peaks: &HashSet<Coordinate>| peaks.len())
        .sum()
}

/// Compute the sum of all distinct trails that depart from a trailhead.
/// Any element in the matrix is a trailhead if:
/// - it has the value 0.
/// - it has at least 1 continuous path from itself to a 9-value element. Moves
///   can only occur in the four cardinal directions North, East, South and West.
///   The score of a trailhead equals the number of acceptable paths.
pub fn part_2(matrix: &Matrix<u8>) -> usize {
    solve(matrix).n_trails
}

#[cfg(test)]
mod tests {
    use super::{parse_input, part_1, part_2};
    use crate::util::{read_file_to_string, Matrix};
    const INPUT: &str = "89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            &parse_input(INPUT),
            &Matrix::new(vec![
                vec![8, 9, 0, 1, 0, 1, 2, 3],
                vec![7, 8, 1, 2, 1, 8, 7, 4],
                vec![8, 7, 4, 3, 0, 9, 6, 5],
                vec![9, 6, 5, 4, 9, 8, 7, 4],
                vec![4, 5, 6, 7, 8, 9, 0, 3],
                vec![3, 2, 0, 1, 9, 0, 1, 2],
                vec![0, 1, 3, 2, 9, 8, 0, 1],
                vec![1, 0, 4, 5, 6, 7, 3, 2],
            ])
        )
    }

    #[test]
    fn test_part_1_small() {
        // Scores of trailheads in reading order.
        // 5, 6, 5, 3, 1, 3, 5, 3, 5 expected
        assert_eq!(part_1(&parse_input(INPUT)), 36)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&parse_input(&read_file_to_string("data/day10.txt"))),
            794
        );
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&parse_input(INPUT)), 81)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&parse_input(&read_file_to_string("data/day10.txt"))),
            1706
        )
    }
}
