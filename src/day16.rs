use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::util::{Cardinal, Coordinate, Matrix};

#[derive(PartialEq, Debug)]
pub struct Maze {
    pub matrix: Matrix<bool>,
    start: Coordinate,
    end: Coordinate,
    direction: Cardinal,
}

#[repr(u8)]
enum MazeChar {
    Vacant = b'.',
    Wall = b'#',
    Start = b'S',
    End = b'E',
}

impl TryFrom<u8> for MazeChar {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == MazeChar::Vacant as u8 => Ok(MazeChar::Vacant),
            x if x == MazeChar::Wall as u8 => Ok(MazeChar::Wall),
            x if x == MazeChar::Start as u8 => Ok(MazeChar::Start),
            x if x == MazeChar::End as u8 => Ok(MazeChar::End),
            _ => Err(()),
        }
    }
}

pub fn parse_input(input: &str) -> Maze {
    let mut start: Option<Coordinate> = None;
    let mut end: Option<Coordinate> = None;

    let mut rows = vec![];
    for (r, line) in input.lines().enumerate() {
        let mut row = vec![false; line.len()];
        for (c, byte) in line.bytes().enumerate() {
            if byte != MazeChar::Wall as u8 {
                row[c] = true;
            }
            match byte.try_into() {
                Ok(MazeChar::Wall) | Ok(MazeChar::Vacant) => {}
                Ok(MazeChar::Start) => {
                    start = Some(Coordinate {
                        r: r as isize,
                        c: c as isize,
                    })
                }
                Ok(MazeChar::End) => {
                    end = Some(Coordinate {
                        r: r as isize,
                        c: c as isize,
                    })
                }
                Err(()) => unimplemented!(),
            }
        }
        rows.push(row)
    }
    Maze {
        matrix: Matrix::new(rows),
        start: start.unwrap(),
        end: end.unwrap(),
        direction: Cardinal::East,
    }
}

#[repr(usize)]
enum Score {
    Straight = 1,
    Turn = 1000,
}

#[derive(Debug, Clone)]
pub struct TraversalState {
    pub score: usize,
    pub coord: Coordinate,
    pub direction: Cardinal,
    pub positions: Vec<Coordinate>,
}

impl PartialEq for TraversalState {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for TraversalState {}

impl PartialOrd for TraversalState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TraversalState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .score
            .cmp(&self.score)
            .then_with(|| self.coord.cmp(&other.coord))
            .then_with(|| self.direction.cmp(&other.direction))
    }
}

fn solve(maze: Maze) -> Vec<TraversalState> {
    let mut min_heap: BinaryHeap<TraversalState> = BinaryHeap::from([TraversalState {
        score: 0,
        coord: maze.start,
        direction: maze.direction,
        positions: vec![maze.start],
    }]);
    let mut visited = HashMap::new();
    let mut states = vec![];
    let mut best_score = None;

    while let Some(state) = min_heap.pop() {
        if best_score.is_some() && state.score > best_score.unwrap() {
            continue;
        }
        if state.coord == maze.end {
            best_score = Some(state.score);
            states.push(state.clone());
        }

        // This can be improved. Deviating paths of equal score that merge back
        // into the main track are again explored fully.
        // We could run part 1 to get a path, trackin scores along the way.
        // This could serve as an input to part 2 where we can reject side paths
        // that get a worse score upon merging.
        let mut worse_path = false;
        visited
            .entry((state.coord, state.direction))
            .and_modify(|best_score: &mut usize| {
                if *best_score < state.score {
                    worse_path = true;
                } else {
                    *best_score = state.score
                }
            })
            .or_insert(state.score);
        if worse_path {
            continue;
        }

        let directions = match &state.direction {
            Cardinal::North => [Cardinal::West, Cardinal::North, Cardinal::East],
            Cardinal::East => [Cardinal::North, Cardinal::East, Cardinal::South],
            Cardinal::South => [Cardinal::East, Cardinal::South, Cardinal::West],
            Cardinal::West => [Cardinal::South, Cardinal::West, Cardinal::North],
        };

        for direction in directions {
            let destination = state.coord.cardinal(direction);
            if [destination.r, destination.c]
                .iter()
                .any(|val| val.is_negative())
                || !*maze
                    .matrix
                    .get_element([destination.r as usize, destination.c as usize])
                    .unwrap_or(&false)
            {
                continue;
            };

            let (coord, score) = if direction == state.direction {
                (destination, state.score + Score::Straight as usize)
            } else {
                (
                    destination,
                    state.score + Score::Straight as usize + Score::Turn as usize,
                )
            };
            let mut positions = state.clone().positions;
            positions.push(destination);
            min_heap.push(TraversalState {
                direction,
                score,
                coord,
                positions,
            });
        }
    }
    states
}

pub fn part_1(maze: Maze) -> usize {
    solve(maze).first().unwrap().score
}

pub fn part_2(maze: Maze) -> usize {
    let mut positions = HashSet::<Coordinate>::new();
    for solution in solve(maze) {
        positions.extend(solution.positions);
    }
    positions.len()
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use itertools::assert_equal;

    use crate::{
        day16::{Maze, TraversalState},
        util::{read_file_to_string, Cardinal, Coordinate, Matrix},
    };

    use super::{parse_input, part_1, part_2};

    const INPUT_1: &str = "###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############
";

    const INPUT_2: &str = "#################
#...#...#...#..E#
#.#.#.#.#.#.#.#.#
#.#.#.#...#...#.#
#.#.#.#.###.#.#.#
#...#.#.#.....#.#
#.#.#.#.#.#####.#
#.#...#.#.#.....#
#.#.#####.#.###.#
#.#.#.......#...#
#.#.###.#####.###
#.#.#...#.....#.#
#.#.#.#####.###.#
#.#.#.........#.#
#.#.#.#########.#
#S#.............#
#################
";

    const INPUT_3: &str = "#######E#######
#...#...#######
#.#...#.......#
#.###########.#
#S............#
###############";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT_1),
            Maze {
                matrix: Matrix::new(vec![
                    vec![
                        false, false, false, false, false, false, false, false, false, false,
                        false, false, false, false, false
                    ],
                    vec![
                        false, true, true, true, true, true, true, true, false, true, true, true,
                        true, true, false
                    ],
                    vec![
                        false, true, false, true, false, false, false, true, false, true, false,
                        false, false, true, false
                    ],
                    vec![
                        false, true, true, true, true, true, false, true, false, true, true, true,
                        false, true, false,
                    ],
                    vec![
                        false, true, false, false, false, true, false, false, false, false, false,
                        true, false, true, false
                    ],
                    vec![
                        false, true, false, true, false, true, true, true, true, true, true, true,
                        false, true, false
                    ],
                    vec![
                        false, true, false, true, false, false, false, false, false, true, false,
                        false, false, true, false
                    ],
                    vec![
                        false, true, true, true, true, true, true, true, true, true, true, true,
                        false, true, false,
                    ],
                    vec![
                        false, false, false, true, false, true, false, false, false, false, false,
                        true, false, true, false
                    ],
                    vec![
                        false, true, true, true, false, true, true, true, true, true, false, true,
                        false, true, false
                    ],
                    vec![
                        false, true, false, true, false, true, false, false, false, true, false,
                        true, false, true, false
                    ],
                    vec![
                        false, true, true, true, true, true, false, true, true, true, false, true,
                        false, true, false
                    ],
                    vec![
                        false, true, false, false, false, true, false, true, false, true, false,
                        true, false, true, false
                    ],
                    vec![
                        false, true, true, true, false, true, true, true, true, true, false, true,
                        true, true, false
                    ],
                    vec![
                        false, false, false, false, false, false, false, false, false, false,
                        false, false, false, false, false
                    ],
                ]),
                start: Coordinate { r: 13, c: 1 },
                end: Coordinate { r: 1, c: 13 },
                direction: Cardinal::East
            }
        )
    }

    #[test]
    fn test_min_heap() {
        let state_1 = TraversalState {
            score: 1,
            coord: Coordinate::default(),
            direction: Cardinal::North,
            positions: vec![Coordinate::default()],
        };
        let state_2 = TraversalState {
            score: 2,
            coord: Coordinate::default(),
            direction: Cardinal::North,
            positions: vec![Coordinate::default()],
        };
        let state_3 = TraversalState {
            score: 3,
            coord: Coordinate::default(),
            direction: Cardinal::North,
            positions: vec![Coordinate::default()],
        };
        let states = [state_3.clone(), state_1.clone(), state_2.clone()];

        let mut min_heap = BinaryHeap::from(states);

        assert_equal(min_heap.pop(), Some(state_1));
        assert_equal(min_heap.pop(), Some(state_2));
        assert_equal(min_heap.pop(), Some(state_3));
        assert_equal(min_heap.pop(), None);
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(parse_input(INPUT_1)), 7036);
        assert_eq!(part_1(parse_input(INPUT_2)), 11048);
        assert_eq!(part_1(parse_input(INPUT_3)), 3022);
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(parse_input(&read_file_to_string("data/day16.txt"))),
            106512
        )
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(parse_input(INPUT_1)), 45);
        assert_eq!(part_2(parse_input(INPUT_2)), 64);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(parse_input(&read_file_to_string("data/day16.txt"))),
            563
        )
    }
}
