use std::{fmt::Debug, vec};

use crate::util::{Coordinate, Matrix};

type Coord = [usize; 2];

pub fn parse_input(input: &str) -> Matrix<char> {
    Matrix::new(input.lines().map(|line| line.chars().collect()).collect())
}

fn north(coord: Coord) -> Option<Coord> {
    coord[1].checked_sub(1).map(|c| [coord[0], c])
}
fn south(coord: Coord) -> Option<Coord> {
    coord[1].checked_add(1).map(|c| [coord[0], c])
}
fn east(coord: Coord) -> Option<Coord> {
    coord[0].checked_add(1).map(|c| [c, coord[1]])
}
fn west(coord: Coord) -> Option<Coord> {
    coord[0].checked_sub(1).map(|c| [c, coord[1]])
}

fn get_n_equal_neighbors<T: PartialEq>(coord: Coord, matrix: &Matrix<T>) -> Option<usize> {
    matrix.get_element(coord).map(|value| {
        [north(coord), east(coord), south(coord), west(coord)]
            .iter()
            .filter_map(|c| *c)
            .map(|c| {
                if let Some(neighbor) = matrix.get_element(c) {
                    (neighbor == value) as usize
                } else {
                    0
                }
            })
            .sum()
    })
}

/// Segment an image into regions of identical value,
/// connected in the 4 cardinal directions.
/// # Example usage
/// ```rust
/// use advent_of_code_2024::day12::watershed;
/// use advent_of_code_2024::util::Matrix;
/// let matrix = Matrix::new(vec![
///     vec!['A', 'A', 'A', 'A'],
///     vec!['B', 'B', 'C', 'D'],
///     vec!['B', 'B', 'C', 'C'],
///     vec!['E', 'E', 'E', 'C'],
/// ]);
/// let expected = Matrix::new(vec![
///     vec![0, 0, 0, 0],
///     vec![1, 1, 2, 3],
///     vec![1, 1, 2, 2],
///     vec![4, 4, 4, 2],
/// ]);
/// assert_eq!(watershed(&matrix), expected)
/// ```
pub fn watershed<T: PartialEq>(matrix: &Matrix<T>) -> Matrix<usize> {
    let mut output = Matrix::new_like(matrix, 0usize);
    let mut counter = 0usize;
    let mut visited = Matrix::new(vec![vec![false; matrix.shape()[1]]; matrix.shape()[0]]);
    for row in matrix.row_range() {
        for col in matrix.col_range() {
            if visited[row][col] {
                continue;
            }
            let mut queue = vec![Coordinate::new(row as isize, col as isize)];
            while let Some(coord) = queue.pop() {
                let [row, col] = [coord.r as usize, coord.c as usize];
                if visited[row][col] {
                    continue;
                }
                let neighbors = get_cardinal_neighbors(coord, matrix);
                if !neighbors.is_empty() {
                    visited[row][col] = true;
                    queue.extend(neighbors);
                }
                output[row][col] = counter;
            }
            counter += 1;
        }
    }
    output
}

fn get_cardinal_neighbors<T: PartialEq>(coord: Coordinate, matrix: &Matrix<T>) -> Vec<Coordinate> {
    let [row, col] = [coord.r as usize, coord.c as usize];
    let mut vector = vec![];
    for neighbor in coord.cardinals() {
        if !neighbor.r.is_negative() && !neighbor.c.is_negative() {
            let [neighbor_row, neighbor_col] = [neighbor.r as usize, neighbor.c as usize];
            if let Some(n) = matrix.get_element([neighbor_row, neighbor_col]) {
                if n == &(matrix[row][col]) {
                    vector.push(neighbor);
                }
            }
        }
    }
    vector
}
fn get_diagonal_neighbors<T: PartialEq>(coord: Coordinate, matrix: &Matrix<T>) -> Vec<Coordinate> {
    let [row, col] = [coord.r as usize, coord.c as usize];
    let mut vector = vec![];
    for neighbor in coord.diagonals() {
        if !neighbor.r.is_negative() && !neighbor.c.is_negative() {
            let [neighbor_row, neighbor_col] = [neighbor.r as usize, neighbor.c as usize];
            if let Some(n) = matrix.get_element([neighbor_row, neighbor_col]) {
                if n == &(matrix[row][col]) {
                    vector.push(neighbor);
                }
            }
        }
    }
    vector
}

#[derive(Debug)]
struct RegionCircumference {
    pub area: usize,
    pub circumference: usize,
}

/// Track the area and circumference of each connected region of space.
/// Calculate the sum of all products area x circumference.
pub fn part_1(matrix: &Matrix<char>) -> usize {
    let mut regions = <Vec<RegionCircumference>>::new();
    let watershed = watershed(matrix);
    for row in matrix.row_range() {
        for col in 0..matrix.shape()[1] {
            let circumference = 4 - get_n_equal_neighbors([row, col], &watershed).unwrap();
            let idx = watershed[row][col];
            if idx == regions.len() {
                regions.push(RegionCircumference {
                    area: 1,
                    circumference,
                });
            } else {
                regions[idx].area += 1;
                regions[idx].circumference += circumference;
            }
        }
    }
    regions.iter().fold(0, |coord, region| {
        coord + region.area * region.circumference
    })
}

#[derive(Debug)]
pub struct RegionCorners {
    area: usize,
    n_corners: usize,
}

fn added_corners<T: PartialEq>(coord: Coordinate, matrix: &Matrix<T>) -> usize {
    let cardinals = get_cardinal_neighbors(coord, matrix);
    let diagonals = get_diagonal_neighbors(coord, matrix);
    match cardinals.len() {
        0 => 4,
        1 => 2,
        2 => {
            // Neighbors are returned in cardinal order NESW.
            if cardinals == vec![coord.north(), coord.south()]
                || cardinals == vec![coord.east(), coord.west()]
            {
                0
            } else {
                check_corners_for_2_cardinals(
                    [coord.north(), coord.east()],
                    &coord.north_east(),
                    &cardinals,
                    &diagonals,
                )
                .or_else(|| {
                    check_corners_for_2_cardinals(
                        [coord.east(), coord.south()],
                        &coord.south_east(),
                        &cardinals,
                        &diagonals,
                    )
                })
                .or_else(|| {
                    check_corners_for_2_cardinals(
                        [coord.south(), coord.west()],
                        &coord.south_west(),
                        &cardinals,
                        &diagonals,
                    )
                })
                .or_else(|| {
                    check_corners_for_2_cardinals(
                        [coord.north(), coord.west()],
                        &coord.north_west(),
                        &cardinals,
                        &diagonals,
                    )
                })
                .unwrap()
            }
        }
        3 => check_corners_for_3_cardinals(
            [coord.north(), coord.east(), coord.south()],
            [coord.north_east(), coord.south_east()],
            &cardinals,
            &diagonals,
        )
        .or_else(|| {
            check_corners_for_3_cardinals(
                [coord.east(), coord.south(), coord.west()],
                [coord.south_east(), coord.south_west()],
                &cardinals,
                &diagonals,
            )
        })
        .or_else(|| {
            check_corners_for_3_cardinals(
                [coord.north(), coord.south(), coord.west()],
                [coord.south_west(), coord.north_west()],
                &cardinals,
                &diagonals,
            )
        })
        .or_else(|| {
            check_corners_for_3_cardinals(
                [coord.north(), coord.east(), coord.west()],
                [coord.north_west(), coord.north_east()],
                &cardinals,
                &diagonals,
            )
        })
        .unwrap(),
        4 => check_corners_for_4_cardinals(&diagonals),
        _ => unreachable!(),
    }
}

fn check_corners_for_2_cardinals(
    cardinal: [Coordinate; 2],
    diagonal: &Coordinate,
    cardinals: &[Coordinate],
    diagonals: &[Coordinate],
) -> Option<usize> {
    if cardinals == cardinal {
        if diagonals.contains(diagonal) {
            return Some(1);
        }
        return Some(2);
    }
    None
}

fn check_corners_for_3_cardinals(
    cardinal: [Coordinate; 3],
    diagonal: [Coordinate; 2],
    cardinals: &[Coordinate],
    diagonals: &[Coordinate],
) -> Option<usize> {
    if cardinal != cardinals {
        return None;
    }
    Some(
        2usize.saturating_sub(
            diagonal
                .iter()
                .map(|diag| diagonals.contains(diag) as usize)
                .sum(),
        ),
    )
}

fn check_corners_for_4_cardinals(diagonals: &[Coordinate]) -> usize {
    4usize.saturating_sub(diagonals.len())
}

/// Track the area and number of sides of each connected region of space.
/// Calculate the sum of all products area x n_sides.
pub fn part_2(matrix: &Matrix<char>) -> usize {
    let mut regions = <Vec<RegionCorners>>::new();
    let watershed = watershed(matrix);
    for row in matrix.row_range() {
        for col in 0..matrix.shape()[1] {
            let n_corners = added_corners(
                Coordinate {
                    r: row as isize,
                    c: col as isize,
                },
                matrix,
            );
            let idx = watershed[row][col];
            if idx == regions.len() {
                regions.push(RegionCorners { area: 1, n_corners });
            } else {
                regions[idx].area += 1;
                regions[idx].n_corners += n_corners;
            }
        }
    }
    regions
        .iter()
        .fold(0, |coord, region| coord + region.area * region.n_corners)
}

#[cfg(test)]
mod tests {
    use crate::{
        day12::{get_n_equal_neighbors, parse_input, part_1, part_2, watershed},
        util::{read_file_to_string, Matrix},
    };

    const INPUT: &str = "AAAA\nBBCD\nBBCC\nEEEC";
    const INPUT_LARGE: &str = "RRRRIICCFF\nRRRRIICCCF\nVVRRRCCFFF\nVVRCCCJFFF\nVVVVCJJCFE\nVVIVCCJJEE\nVVIIICJJEE\nMIIIIIJJEE\nMIIISIJEEE\nMMMISSJEEE\n";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT),
            Matrix::new(vec![
                vec!['A', 'A', 'A', 'A'],
                vec!['B', 'B', 'C', 'D'],
                vec!['B', 'B', 'C', 'C'],
                vec!['E', 'E', 'E', 'C']
            ])
        )
    }

    #[test]
    fn test_equal_neighbors() {
        let matrix = Matrix::new(vec![
            vec!['A', 'A', 'A', 'A'],
            vec!['B', 'B', 'C', 'D'],
            vec!['B', 'B', 'C', 'C'],
            vec!['E', 'E', 'E', 'C'],
        ]);
        let mut calculated_neighbors = Vec::new();
        for y in 0..matrix.shape()[0] {
            let mut row = Vec::new();
            for x in 0..matrix.shape()[1] {
                row.push(get_n_equal_neighbors([y, x], &matrix).unwrap());
            }
            calculated_neighbors.push(row);
        }
        let expected_neighbors: Vec<Vec<usize>> = vec![
            vec![1, 2, 2, 1],
            vec![2, 2, 1, 0],
            vec![2, 2, 2, 2],
            vec![1, 2, 1, 1],
        ];
        assert_eq!(calculated_neighbors, expected_neighbors);
    }

    #[test]
    fn test_watershed() {
        let matrix = Matrix::new(vec![
            vec!['A', 'A', 'A', 'A'],
            vec!['B', 'B', 'C', 'D'],
            vec!['B', 'B', 'C', 'C'],
            vec!['E', 'E', 'E', 'C'],
        ]);
        let expected = Matrix::new(vec![
            vec![0, 0, 0, 0],
            vec![1, 1, 2, 3],
            vec![1, 1, 2, 2],
            vec![4, 4, 4, 2],
        ]);
        assert_eq!(watershed(&matrix), expected)
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&parse_input(INPUT)), 140);
        let input_2 = "OOOOO\nOXOXO\nOOOOO\nOXOXO\nOOOOO";
        assert_eq!(part_1(&parse_input(input_2)), 772);
        assert_eq!(part_1(&parse_input(INPUT_LARGE)), 1930);
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(&parse_input(&read_file_to_string("data/day12.txt"))),
            1434856
        );
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&parse_input(INPUT)), 80);
        assert_eq!(
            part_2(&parse_input("EEEEE\nEXXXX\nEEEEE\nEXXXX\nEEEEE")),
            236
        );
        assert_eq!(
            part_2(&parse_input(
                "AAAAAA\nAAABBA\nAAABBA\nABBAAA\nABBAAA\nAAAAAA"
            )),
            368
        );
        assert_eq!(part_2(&parse_input(INPUT_LARGE)), 1206);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(&parse_input(&read_file_to_string("data/day12.txt"))),
            891106
        );
    }
}
