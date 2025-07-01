use std::fmt::Display;

use nom::{
    character::complete::{line_ending, one_of},
    error::Error,
    multi::{count, fold_many1, separated_list1},
    sequence::separated_pair,
    Finish, IResult, Parser,
};

use crate::util::{Coordinate, Matrix};

#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Cardinal {
    North = b'^',
    East = b'>',
    South = b'v',
    West = b'<',
}

const COORDINATE_NORTH: Coordinate = Coordinate { r: -1, c: 0 };
const COORDINATE_EAST: Coordinate = Coordinate { r: 0, c: 1 };
const COORDINATE_SOUTH: Coordinate = Coordinate { r: 1, c: 0 };
const COORDINATE_WEST: Coordinate = Coordinate { r: 0, c: -1 };

impl From<Cardinal> for Coordinate {
    fn from(value: Cardinal) -> Self {
        match value {
            Cardinal::North => COORDINATE_NORTH,
            Cardinal::East => COORDINATE_EAST,
            Cardinal::South => COORDINATE_SOUTH,
            Cardinal::West => COORDINATE_WEST,
        }
    }
}

#[derive(Debug)]
pub struct CannotParseFromChar;

impl TryFrom<char> for Cardinal {
    type Error = CannotParseFromChar;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '^' => Ok(Self::North),
            '>' => Ok(Self::East),
            'v' => Ok(Self::South),
            '<' => Ok(Self::West),
            _ => Err(CannotParseFromChar),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Narrow {
    Robot,
    Wall,
    Empty,
    Package,
}

impl TryFrom<char> for Narrow {
    type Error = CannotParseFromChar;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '@' => Ok(Self::Robot),
            '#' => Ok(Self::Wall),
            '.' => Ok(Self::Empty),
            'O' => Ok(Self::Package),
            _ => Err(CannotParseFromChar),
        }
    }
}

impl Display for Narrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Empty => '.',
                Self::Package => 'O',
                Self::Robot => '@',
                Self::Wall => '#',
            }
        )
    }
}

fn parse_warehouse(input: &str) -> IResult<&str, Vec<Vec<Narrow>>> {
    separated_list1(
        line_ending,
        fold_many1(one_of("@#.O"), Vec::new, |mut acc, c| {
            acc.push(Narrow::try_from(c).expect("invalid char"));
            acc
        }),
    )
    .parse(input)
}

fn parse_directions(input: &str) -> IResult<&str, Vec<Cardinal>> {
    fold_many1(
        separated_list1(line_ending, one_of("^>v<")),
        Vec::new,
        |mut acc, line| unsafe {
            acc.extend(
                line.iter()
                    .map(|c| Cardinal::try_from(*c).unwrap_unchecked()),
            );
            acc
        },
    )
    .parse(input)
}

#[derive(PartialEq, Debug)]
pub struct Warehouse<W> {
    robot: Coordinate,
    matrix: Matrix<W>,
    directions: Vec<Cardinal>,
    i: usize,
}

impl<W: Display> Display for Warehouse<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.matrix.row_range() {
            for col in self.matrix.col_range() {
                if Coordinate::new(row as isize, col as isize) == self.robot {
                    write!(f, "@")?;
                } else {
                    write!(f, "{}", self.matrix[row][col])?;
                }
            }
            writeln!(f,)?;
        }
        Ok(())
    }
}

impl Warehouse<Narrow> {
    pub fn take_step(&mut self) -> Option<()> {
        if self.i < self.directions.len() {
            let direction = self.directions[self.i];
            let destination = self.robot + direction.into();
            match self.matrix[destination.r as usize][destination.c as usize] {
                Narrow::Empty => self.robot = destination,
                Narrow::Wall => (),
                Narrow::Robot => unreachable!(),
                Narrow::Package => self.move_package(&destination, &direction),
            }
            self.i += 1;
            Some(())
        } else {
            None
        }
    }

    /// Create an iter along the given axis and direction.
    /// If it is unobstructed i.e., does not contain any walls before an empty
    /// spot, move the boxes. This can be done "smartly" by moving the first box
    /// to the end and the robot the first spot.
    fn move_package(&mut self, package: &Coordinate, towards: &Cardinal) {
        let p = [package.r as usize, package.c as usize];
        let iter: Box<dyn Iterator<Item = &Narrow>> = match towards {
            Cardinal::North => Box::new(
                self.matrix
                    .col(p[1])
                    .unwrap()
                    .rev()
                    .skip(self.matrix.shape()[0] - p[0]),
            ),
            Cardinal::South => Box::new(self.matrix.col(p[1]).unwrap().skip(p[0] + 1)),
            Cardinal::East => Box::new(self.matrix.row(p[0]).unwrap().skip(p[1] + 1)),
            Cardinal::West => Box::new(
                self.matrix
                    .row(p[0])
                    .unwrap()
                    .rev()
                    .skip(self.matrix.shape()[1] - p[1]),
            ),
        };
        let mut can_move_to = None;
        for (i, spot) in iter.enumerate() {
            match spot {
                Narrow::Empty => {
                    can_move_to = Some(i);
                    break;
                }
                Narrow::Wall => {
                    break;
                }
                Narrow::Package => {
                    continue;
                }
                Narrow::Robot => unreachable!(),
            }
        }
        if let Some(i) = can_move_to {
            self.robot = self.robot + (*towards).into();
            let destination = *package + Coordinate::from(*towards) * (i as isize + 1);
            self.matrix[p[0]][p[1]] = Narrow::Empty;
            self.matrix[destination.r as usize][destination.c as usize] = Narrow::Package;
        }
    }
}

pub fn parse_input(input: &str) -> Result<Warehouse<Narrow>, Error<&str>> {
    let (input, (mut objects, directions)) =
        separated_pair(parse_warehouse, count(line_ending, 2), parse_directions)
            .parse(input)
            .finish()?;
    assert!(input.is_empty());

    let mut robot = Coordinate::default();
    'outer: for (r, row) in objects.iter_mut().enumerate() {
        for (c, col) in row.iter_mut().enumerate() {
            if *col == Narrow::Robot {
                robot = Coordinate::new(r as isize, c as isize);
                *col = Narrow::Empty;
                break 'outer;
            }
        }
    }

    Ok(Warehouse {
        robot,
        matrix: Matrix::new(objects),
        directions,
        i: 0,
    })
}

pub fn part_1(warehouse: &mut Warehouse<Narrow>) -> usize {
    while warehouse.take_step().is_some() {}
    let mut sum = 0;
    for row in warehouse.matrix.row_range() {
        for col in warehouse.matrix.col_range() {
            if warehouse.matrix[row][col] == Narrow::Package {
                sum += 100 * row + col;
            }
        }
    }
    sum
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Wide {
    Wall,
    Empty,
    PackageLeft,
    PackageRight,
}

impl Display for Wide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Wide::Empty => '.',
            Wide::PackageLeft => '[',
            Wide::PackageRight => ']',
            Wide::Wall => '#',
        };
        write!(f, "{c}")
    }
}

fn matrix_to_wide_matrix(matrix: &Matrix<Narrow>) -> Matrix<Wide> {
    let mut vec: Vec<Vec<Wide>> = Vec::with_capacity(matrix.shape()[0]);
    for row in matrix.row_iter() {
        let mut new_row: Vec<Wide> = Vec::with_capacity(matrix.shape()[1]);
        for col in row {
            let arr = match col {
                Narrow::Empty => [Wide::Empty; 2],
                Narrow::Wall => [Wide::Wall; 2],
                Narrow::Package => [Wide::PackageLeft, Wide::PackageRight],
                Narrow::Robot => unreachable!(),
            };
            new_row.extend(arr.iter());
        }
        vec.push(new_row)
    }
    Matrix::new(vec)
}

impl From<Warehouse<Narrow>> for Warehouse<Wide> {
    fn from(value: Warehouse<Narrow>) -> Self {
        Self {
            robot: Coordinate::new(value.robot.r, value.robot.c * 2),
            matrix: matrix_to_wide_matrix(&value.matrix),
            directions: value.directions,
            i: 0,
        }
    }
}

impl Warehouse<Wide> {
    /// Create a graph that connects every box part (left and right) to:
    /// - it's neighboring part
    /// - the box part directly adjacent along the movement axis and direction.
    ///
    /// Compute a BFS along this tree, tracking which box parts might need to be
    /// moved. If no walls are encountered, make all moves on a matrix copy (it
    /// might be possible without allocating as well). Otherwise, abort the
    /// search, clear the stack and list of moves.
    fn move_package(&mut self, package: Coordinate, direction: &Cardinal) -> Vec<Coordinate> {
        let mut moves = Vec::<Coordinate>::new();
        let mut stack = Vec::<Coordinate>::new();
        let mut visited = Matrix::new_like(&self.matrix, false);
        stack.push(package);
        while let Some(next_package) = stack.pop() {
            let [row, col] = [next_package.r as usize, next_package.c as usize];
            if visited[row][col] {
                continue;
            }
            let package_l_r = match self.matrix[row][col] {
                Wide::PackageLeft => {
                    stack.push(next_package.east());
                    [next_package, next_package.east()]
                }
                Wide::PackageRight => {
                    stack.push(next_package.west());
                    [next_package.west(), next_package]
                }
                Wide::Empty | Wide::Wall => unreachable!(),
            };
            for package_part in package_l_r {
                if visited[package_part.r as usize][package_part.c as usize] {
                    continue;
                } else {
                    visited[package_part.r as usize][package_part.c as usize] = true;
                }
                let destination = match direction {
                    Cardinal::North => package_part.north(),
                    Cardinal::East => package_part.east(),
                    Cardinal::South => package_part.south(),
                    Cardinal::West => package_part.west(),
                };
                match self.matrix[destination.r as usize][destination.c as usize] {
                    Wide::Empty => moves.push(package_part),
                    Wide::Wall => {
                        moves.clear();
                        stack.clear();
                        break;
                    }
                    Wide::PackageLeft | Wide::PackageRight => {
                        moves.push(package_part);
                        stack.push(destination);
                    }
                }
            }
        }
        moves
    }

    fn take_step(&mut self) -> Option<()> {
        if self.i >= self.directions.len() {
            return None;
        }
        let direction = self.directions[self.i];
        let destination = self.robot + direction.into();
        let mut packages = Vec::new();
        match self.matrix[destination.r as usize][destination.c as usize] {
            Wide::Empty => self.robot = destination,
            Wide::Wall => (),
            Wide::PackageLeft | Wide::PackageRight => {
                packages = self.move_package(destination, &direction)
            }
        }
        if !packages.is_empty() {
            self.robot = destination;
            let mut copy = Matrix::new(self.matrix.clone());
            for package in packages.iter() {
                copy[package.r as usize][package.c as usize] = Wide::Empty;
            }
            for package in packages.iter() {
                let dest = match direction {
                    Cardinal::North => package.north(),
                    Cardinal::East => package.east(),
                    Cardinal::South => package.south(),
                    Cardinal::West => package.west(),
                };
                copy[dest.r as usize][dest.c as usize] =
                    self.matrix[package.r as usize][package.c as usize];
            }
            self.matrix = copy;
        }
        self.i += 1;
        Some(())
    }
}

pub fn part_2(warehouse: &mut Warehouse<Wide>) -> usize {
    while warehouse.take_step().is_some() {}
    let mut sum = 0;
    for row in warehouse.matrix.row_range() {
        for col in warehouse.matrix.col_range() {
            if warehouse.matrix[row][col] == Wide::PackageLeft {
                sum += 100 * row + col;
            }
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use crate::{
        day15::{
            matrix_to_wide_matrix, parse_input, part_1, part_2, Cardinal, Narrow, Warehouse, Wide,
        },
        util::{read_file_to_string, Coordinate, Matrix},
    };

    const INPUT: &str = "########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########

<^^>>>vv<v>>v<<";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT).unwrap(),
            Warehouse {
                robot: Coordinate { r: 2, c: 2 },
                matrix: Matrix::new(vec![
                    vec![Narrow::Wall; 8],
                    vec![
                        Narrow::Wall,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Package,
                        Narrow::Empty,
                        Narrow::Package,
                        Narrow::Empty,
                        Narrow::Wall
                    ],
                    vec![
                        Narrow::Wall,
                        Narrow::Wall,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Package,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Wall
                    ],
                    vec![
                        Narrow::Wall,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Package,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Wall
                    ],
                    vec![
                        Narrow::Wall,
                        Narrow::Empty,
                        Narrow::Wall,
                        Narrow::Empty,
                        Narrow::Package,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Wall
                    ],
                    vec![
                        Narrow::Wall,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Package,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Wall
                    ],
                    vec![
                        Narrow::Wall,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Empty,
                        Narrow::Wall
                    ],
                    vec![Narrow::Wall; 8]
                ]),
                directions: vec![
                    Cardinal::West,
                    Cardinal::North,
                    Cardinal::North,
                    Cardinal::East,
                    Cardinal::East,
                    Cardinal::East,
                    Cardinal::South,
                    Cardinal::South,
                    Cardinal::West,
                    Cardinal::South,
                    Cardinal::East,
                    Cardinal::East,
                    Cardinal::South,
                    Cardinal::West,
                    Cardinal::West
                ],
                i: 0
            }
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&mut parse_input(INPUT).expect("cannot read")), 2028);
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&mut parse_input(&read_file_to_string("data/day15.txt")).expect("cannot read")),
            1441031
        );
    }

    #[test]
    fn test_matrix_to_wide_matrix() {
        let matrix = parse_input(INPUT).unwrap().matrix;
        assert_eq!(
            matrix_to_wide_matrix(&matrix),
            Matrix::new(vec![
                vec![Wide::Wall; 16],
                vec![
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::PackageLeft,
                    Wide::PackageRight,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::PackageLeft,
                    Wide::PackageRight,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Wall,
                    Wide::Wall,
                ],
                vec![
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::PackageLeft,
                    Wide::PackageRight,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Wall,
                    Wide::Wall,
                ],
                vec![
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::PackageLeft,
                    Wide::PackageRight,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Wall,
                    Wide::Wall,
                ],
                vec![
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::PackageLeft,
                    Wide::PackageRight,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Wall,
                    Wide::Wall,
                ],
                vec![
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::PackageLeft,
                    Wide::PackageRight,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Wall,
                    Wide::Wall,
                ],
                vec![
                    Wide::Wall,
                    Wide::Wall,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Empty,
                    Wide::Wall,
                    Wide::Wall,
                ],
                vec![Wide::Wall; 16]
            ])
        )
    }

    const INPUT_MEDIUM: &str = "##########
#..O..O.O#
#......O.#
#.OO..O.O#
#..O@..O.#
#O#..O...#
#O..O..O.#
#.OO.O.OO#
#....O...#
##########

<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
<<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
>^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
<><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^";

    #[test]
    fn test_part_2_small() {
        assert_eq!(
            part_2(&mut parse_input(INPUT_MEDIUM).unwrap().into(),),
            9021
        )
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(
                &mut parse_input(&read_file_to_string("data/day15.txt"))
                    .unwrap()
                    .into(),
            ),
            1425169
        )
    }
}
