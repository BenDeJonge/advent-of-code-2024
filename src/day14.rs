use std::ops::Range;

use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    error::Error,
    multi::many1,
    sequence::{preceded, separated_pair, terminated},
    Finish, IResult,
};

use nom::character::complete::i32;

use crate::util::Coordinate;

const DIMENSIONS: Coordinate = Coordinate { r: 101, c: 103 };
const N_STEPS_PART_1: usize = 100;
const N_STEPS_PART_2: usize = 10_000;

#[derive(Debug, PartialEq)]
pub struct Robot {
    coordinate: Coordinate,
    velocity: Coordinate,
}

impl Robot {
    pub fn new(coordinate: [isize; 2], velocity: [isize; 2]) -> Self {
        Self {
            coordinate: coordinate.into(),
            velocity: velocity.into(),
        }
    }
}

fn parse<'a>(input: &'a str, preceded_str: &str) -> IResult<&'a str, Coordinate> {
    let (input, (x, y)) = preceded(tag(preceded_str), separated_pair(i32, tag(","), i32))(input)?;
    Ok((input, Coordinate::new(x as isize, y as isize)))
}
fn parse_coordinate(input: &str) -> IResult<&str, Coordinate> {
    parse(input, "p=")
}
fn parse_velocity(input: &str) -> IResult<&str, Coordinate> {
    parse(input, "v=")
}

fn parse_robot(input: &str) -> IResult<&str, Robot> {
    let (input, (coordinate, velocity)) = terminated(
        separated_pair(parse_coordinate, tag(" "), parse_velocity),
        line_ending,
    )(input)?;
    Ok((
        input,
        Robot {
            coordinate,
            velocity,
        },
    ))
}

pub fn parse_input(input: &str) -> Result<Vec<Robot>, Error<&str>> {
    many1(parse_robot)(input).finish().map(|(input, robots)| {
        assert!(input.is_empty());
        robots
    })
}

struct Quadrant<T> {
    x: Range<T>,
    y: Range<T>,
    pub count: usize,
}

impl Quadrant<isize> {
    pub fn contains(&self, coordinate: &Coordinate) -> bool {
        self.x.contains(&coordinate.r) && self.y.contains(&coordinate.c)
    }

    pub fn top_left(dimensions: &Coordinate) -> Self {
        Quadrant {
            x: 0..(dimensions.r / 2),
            y: 0..(dimensions.c / 2),
            count: 0,
        }
    }

    pub fn bottom_left(dimensions: &Coordinate) -> Self {
        Quadrant {
            x: 0..(dimensions.r / 2),
            y: (dimensions.c - dimensions.c / 2)..dimensions.c,
            count: 0,
        }
    }

    pub fn top_right(dimensions: &Coordinate) -> Self {
        Quadrant {
            x: (dimensions.r - dimensions.r / 2)..dimensions.r,
            y: 0..(dimensions.c / 2),
            count: 0,
        }
    }

    pub fn bottom_right(dimensions: &Coordinate) -> Self {
        Quadrant {
            x: (dimensions.r - dimensions.r / 2)..dimensions.r,
            y: (dimensions.c - dimensions.c / 2)..dimensions.c,
            count: 0,
        }
    }
}

pub fn get_total_step(robot: &Robot, steps: usize) -> Coordinate {
    Coordinate::from([
        robot.velocity.r * steps as isize,
        robot.velocity.c * steps as isize,
    ])
}

pub fn get_destination(robot: &Robot, steps: usize, dimensions: &Coordinate) -> Coordinate {
    let destination = robot.coordinate + get_total_step(robot, steps);
    Coordinate::new(
        destination.r.rem_euclid(dimensions.r),
        destination.c.rem_euclid(dimensions.c),
    )
}

pub fn solve(robots: &[Robot], dimensions: Coordinate, steps: usize) -> usize {
    let mut quadrants = [
        Quadrant::top_left(&dimensions),
        Quadrant::top_right(&dimensions),
        Quadrant::bottom_left(&dimensions),
        Quadrant::bottom_right(&dimensions),
    ];
    for robot in robots {
        let destination = get_destination(robot, steps, &dimensions);
        for quadrant in quadrants.iter_mut() {
            if quadrant.contains(&destination) {
                quadrant.count += 1;
                break;
            }
        }
    }
    quadrants.iter().map(|quadrant| quadrant.count).product()
}

pub fn part_1(robots: &[Robot]) -> usize {
    solve(robots, DIMENSIONS, N_STEPS_PART_1)
}

/// The safety factor is a metric for image entropy as it encodes how clustered
/// the robots (high pixels) are together. Since an image with clear structure
/// i.e., a christmas tree, will have lower entropy than a random image, the
/// minimum of the safety factor is where the tree will be.
pub fn part_2(robots: &mut [Robot]) -> usize {
    (0..N_STEPS_PART_2)
        .map(|steps| solve(robots, DIMENSIONS, steps))
        .enumerate()
        .min_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(index, _)| index)
        .unwrap()
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::{
        day14::{
            get_destination, part_1, part_2, solve, Quadrant, Robot, DIMENSIONS, N_STEPS_PART_1,
        },
        util::{read_file_to_string, Coordinate},
    };

    use super::parse_input;

    const DIMENSIONS_SMALL: Coordinate = Coordinate { r: 11, c: 7 };
    const INPUT: &str = "p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3
";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT).expect("cannot parse"),
            vec![
                Robot::new([0, 4], [3, -3]),
                Robot::new([6, 3], [-1, -3]),
                Robot::new([10, 3], [-1, 2]),
                Robot::new([2, 0], [2, -1]),
                Robot::new([0, 0], [1, 3]),
                Robot::new([3, 0], [-2, -2]),
                Robot::new([7, 6], [-1, -3]),
                Robot::new([3, 0], [-1, -2]),
                Robot::new([9, 3], [2, 3]),
                Robot::new([7, 3], [-1, 2]),
                Robot::new([2, 4], [2, -3]),
                Robot::new([9, 5], [-3, -3]),
            ]
        )
    }

    #[test]
    fn test_quadrants() {
        let top_left = Quadrant::top_left(&DIMENSIONS_SMALL);
        assert_eq!(top_left.x, 0..5);
        assert_eq!(top_left.y, 0..3);
        let top_right = Quadrant::top_right(&DIMENSIONS_SMALL);
        assert_eq!(top_right.x, 6..11);
        assert_eq!(top_right.y, 0..3);
        let bottom_left = Quadrant::bottom_left(&DIMENSIONS_SMALL);
        assert_eq!(bottom_left.x, 0..5);
        assert_eq!(bottom_left.y, 4..7);
        let bottom_right = Quadrant::bottom_right(&DIMENSIONS_SMALL);
        assert_eq!(bottom_right.x, 6..11);
        assert_eq!(bottom_right.y, 4..7);

        let top_left = Quadrant::top_left(&DIMENSIONS);
        assert_eq!(top_left.x, 0..50);
        assert_eq!(top_left.y, 0..51);
        let top_right = Quadrant::top_right(&DIMENSIONS);
        assert_eq!(top_right.x, 51..101);
        assert_eq!(top_right.y, 0..51);
        let bottom_left = Quadrant::bottom_left(&DIMENSIONS);
        assert_eq!(bottom_left.x, 0..50);
        assert_eq!(bottom_left.y, 52..103);
        let bottom_right = Quadrant::bottom_right(&DIMENSIONS);
        assert_eq!(bottom_right.x, 51..101);
        assert_eq!(bottom_right.y, 52..103);
    }

    #[test]
    fn test_positions() {
        let robots = parse_input(INPUT).expect("cannot parse");
        let destinations: Vec<Coordinate> = robots
            .iter()
            .map(|robot| get_destination(robot, N_STEPS_PART_1, &DIMENSIONS_SMALL))
            .sorted()
            .collect();
        let expected: Vec<Coordinate> = vec![
            Coordinate::new(6, 0),
            Coordinate::new(6, 0),
            Coordinate::new(9, 0),
            Coordinate::new(0, 2),
            Coordinate::new(1, 3),
            Coordinate::new(2, 3),
            Coordinate::new(5, 4),
            Coordinate::new(4, 5),
            Coordinate::new(4, 5),
            Coordinate::new(3, 5),
            Coordinate::new(1, 6),
            Coordinate::new(6, 6),
        ]
        .into_iter()
        .sorted()
        .collect();
        assert_eq!(destinations, expected);
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(
            12,
            solve(
                &parse_input(INPUT).unwrap(),
                DIMENSIONS_SMALL,
                N_STEPS_PART_1
            )
        )
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            230436441,
            part_1(&parse_input(&read_file_to_string("data/day14.txt")).unwrap())
        )
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            8270,
            part_2(&mut parse_input(&read_file_to_string("data/day14.txt")).unwrap())
        )
    }
}
