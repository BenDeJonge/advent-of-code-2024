//! This problem is linear algebra question disguised as a programming one.
//! A claw machine can move in two directions `x` and `y` in two fixed amounts.
//! We can model these two possible moves as two vectors:
//!
//! ```text
//! ->   | x1 |    ->   | x2 |
//! v1 = | y1 |    v2 = | y2 |
//! ```
//!
//! Here, `v1` and `v2` represent the movements caused by button A and B,
//! respectively. More verbosely, button A will move the claw `x1` units in the
//! `x` direction and `y1` units in the `y` direction. We can write the net
//! transformation of both vectors in a matrix `A`.
//!
//! ```text
//!     | x1 x2 |
//! A = | y1 y2 |
//! ```
//!
//! Our goal is to identify how many time button A and B need to be pressed to
//! end up at a fixed location we will call `p` (for point). Mathematically, we
//! will represent `p` as a vector, too:
//!
//! ```text
//! ->   | p1 |
//! p  = | p2 |
//! ```
//!
//! In the end we want to solve the following linear system:
//!
//! ```text
//!   ->   ->
//! A s  = p
//! ```
//!
//! i.e., we interested in a vector `s`(for solution) that, when transformed
//! through `A` ends up at the target `p`. This vector will encode the number of
//! presses of buttons A and B, respectively. To solve this equation, we use:
//!
//! ```text
//!       ->       ->
//! A⁻¹ A s  = A⁻¹ p
//! ```
//!
//! Where `A⁻¹` is the inverse of the matrix `A`. Matrix inversion is typically
//! a computationally intensive process which is calculated as:
//!
//! ```text
//! A⁻¹ = 1/det(A) adj(A)
//! ```
//!
//! Where `det(A)` and `adj(A)` are the determinant and adjugate of `A`,
//! respectively. For a simple 2 x 2 matrix, the solution is given by:
//!
//! ```text
//!     | a b |                                 |  d -b |
//! A = | c d |    det(A) = ad - bc    adj(A) = | -c  a |
//!
//!             1    |  d -b |
//! => A⁻¹ = ------- |       |
//!          ad - bc | -c  a |
//! ```
//!
//! The reason we want this matrix inverse is because this is the matrix that
//! has the interesting property to obtain the identity matrix `I` upon
//! multication with its original matrix e.g., for the 2 x 2 case:
//!
//! ```text
//!             | 1 0 |
//! A⁻¹ A = I = | 0 1 |
//! ```
//!
//! The identity matrix is the number `1` from linear algebra i.e., when
//! multiplying it to another matrix or vector, the same object is returned:
//!
//! ```text
//! I A = A
//! ```
//!
//! Hence, we can simplify our equation from above:
//!
//! ```text
//!       ->       ->        ->       ->      ->       ->
//! A⁻¹ A a  = A⁻¹ p  <=>  I a  = A⁻¹ p   <=> a  = A⁻¹ p
//! ```
//!
//! The vector `a` will contain the needed number of presses from both buttons.
//! However, we are only interested in integer solutions as a "half press" does
//! not exist.
//!
//! The question is also formulated in a tricky fashion: "what is the smallest
//! number of tokens you would have to spend to win as many prizes as possible?"
//! This is a nonsensical question to ask as for each machine we are solving a
//! system of 2 linear equations, both of the same 2 variables. There is exactly
//! one solution or no solution at all. Only when both `v1` and `v2` are not
//! linearly independent i.e., one can be made as a combination of the other,
//! there can also be infinitly many solutions. This will be reflected through:
//!
//! ```text
//! det(A) = 0
//! ```
//!
//! Specifically for the question at hand, we are solving this system:
//!
//! ```text
//! ax + by = px
//! cx + dy = py
//! ```
//! with:
//! - `a`: the step size of button A along x
//! - `b`: the step size of button B along x
//! - `px`: the x-coordinate of the prize
//! - `c`: the step size of button A along y
//! - `d`: the step size of button B along y
//! - `py`: the y-coordinate of the prize.
//!
//! and the unknowns:
//! - `x`: the number of times button a is pressed
//! - `y`: the number of times button b is pressed.
//!
//! We can describe this system with the following matrix:
//!
//! ```text
//!     | a b |   | x_a x_b |
//! A = | c d | = | y_a y_b |,
//! ```
//!
//! for which the determinant is:
//!
//! ```text
//! det(A) = (x_a . y_b) - (x_b . y_a)
//! ```
//!
//! and the adjugate is:
//!
//! ```text
//!          |  y_b -x_b |
//! adj(A) = | -y_a  x_a |
//! ```
//!
//! From here, we can calculate the inverted matrix A⁻¹, solve the system and
//! reject any non-integer solutions.
use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, u32},
    error::Error,
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair, tuple},
    Finish, IResult,
};

const COST_BUTTON_A: u32 = 3;
const COST_BUTTON_B: u32 = 1;
const FLOAT_PRECISION: f64 = 1e-4;
const PART_1_MAX_PRESSES: u32 = 100;
const PART_2_PRIZE_OFFSET: f64 = 10_000_000_000_000f64;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Button {
    x: f64,
    y: f64,
    cost: u32,
}

impl Button {
    pub fn new(x: f64, y: f64, cost: u32) -> Self {
        Button { x, y, cost }
    }

    pub fn new_button_a(x: f64, y: f64) -> Self {
        Button::new(x, y, COST_BUTTON_A)
    }

    pub fn new_button_b(x: f64, y: f64) -> Self {
        Button::new(x, y, COST_BUTTON_B)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Prize {
    x: f64,
    y: f64,
}

impl Prize {
    pub fn new(x: f64, y: f64) -> Self {
        Prize { x, y }
    }
}

#[derive(Debug, PartialEq)]
pub struct ClawMachine {
    button_a: Button,
    button_b: Button,
    prize: Prize,
}

impl ClawMachine {
    pub fn new(button_a: Button, button_b: Button, prize: Prize) -> Self {
        ClawMachine {
            button_a,
            button_b,
            prize,
        }
    }

    pub fn solve(&self) -> Option<[u128; 2]> {
        let determinant = (self.button_a.x * self.button_b.y) - (self.button_b.x * self.button_a.y);
        let inverted = [
            [
                self.button_b.y / determinant,
                -self.button_b.x / determinant,
            ],
            [
                -self.button_a.y / determinant,
                self.button_a.x / determinant,
            ],
        ];
        let solved = [
            inverted[0][0] * self.prize.x + inverted[0][1] * self.prize.y,
            inverted[1][0] * self.prize.x + inverted[1][1] * self.prize.y,
        ];
        if solved
            .iter()
            .all(|el| el.fract() <= FLOAT_PRECISION || el.fract() >= (1f64 - FLOAT_PRECISION))
        {
            Some([solved[0].round() as u128, solved[1].round() as u128])
        } else {
            None
        }
    }
}

fn parse<'a>(
    input: &'a str,
    name: &str,
    preceded_1: &str,
    preceded_2: &str,
) -> IResult<&'a str, (u32, u32)> {
    delimited(
        tag(name),
        separated_pair(
            preceded(tag(preceded_1), u32),
            tag(", "),
            preceded(tag(preceded_2), u32),
        ),
        line_ending,
    )(input)
}

fn parse_button_a(input: &str) -> IResult<&str, (u32, u32)> {
    parse(input, "Button A: ", "X+", "Y+")
}
fn parse_button_b(input: &str) -> IResult<&str, (u32, u32)> {
    parse(input, "Button B: ", "X+", "Y+")
}
fn parse_prize(input: &str) -> IResult<&str, (u32, u32)> {
    parse(input, "Prize: ", "X=", "Y=")
}

fn parse_machine(input: &str) -> IResult<&str, ClawMachine> {
    let (input, (button_a, button_b, prize)) = tuple((
        |input| parse_button_a(input),
        |input| parse_button_b(input),
        parse_prize,
    ))(input)?;
    Ok((
        input,
        ClawMachine {
            button_a: Button::new_button_a(button_a.0 as f64, button_a.1 as f64),
            button_b: Button::new_button_b(button_b.0 as f64, button_b.1 as f64),
            prize: Prize {
                x: prize.0 as f64,
                y: prize.1 as f64,
            },
        },
    ))
}

pub fn parse_input(input: &str) -> Result<Vec<ClawMachine>, Error<&str>> {
    let (_, machines) = separated_list1(line_ending, parse_machine)(input).finish()?;
    Ok(machines)
}

/// Calculate the cost of the required button presses for winning machines,
/// capped at 100 presses for each button.
pub fn part_1(machines: &[ClawMachine]) -> u128 {
    machines
        .iter()
        .filter_map(|machine| machine.solve())
        .filter(|presses| {
            presses
                .iter()
                .all(|press| press <= &(PART_1_MAX_PRESSES as u128))
        })
        .map(|[press_a, press_b]| press_a * COST_BUTTON_A as u128 + press_b * COST_BUTTON_B as u128)
        .sum()
}

/// Calculate the cost of the required button presses for winning machines,
/// updating each machine to have a large offset in the prize coordinates.
pub fn part_2(machines: &[ClawMachine]) -> u128 {
    machines
        .iter()
        .filter_map(|machine| {
            let updated_machine = ClawMachine::new(
                machine.button_a,
                machine.button_b,
                Prize::new(
                    machine.prize.x + PART_2_PRIZE_OFFSET,
                    machine.prize.y + PART_2_PRIZE_OFFSET,
                ),
            );
            updated_machine.solve()
        })
        .map(|[press_a, press_b]| press_a * COST_BUTTON_A as u128 + press_b * COST_BUTTON_B as u128)
        .sum()
}

#[cfg(test)]
mod test {
    use crate::{
        day13::{parse_input, part_1, part_2, Button, ClawMachine, Prize},
        util::read_file_to_string,
    };

    const INPUT: &str = "Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176

Button A: X+17, Y+86
Button B: X+84, Y+37
Prize: X=7870, Y=6450

Button A: X+69, Y+23
Button B: X+27, Y+71
Prize: X=18641, Y=10279
";

    #[test]
    fn test_parse_input() {
        let machines = parse_input(INPUT).expect("cannot parse");
        assert_eq!(
            machines,
            vec![
                ClawMachine::new(
                    Button::new_button_a(94.0, 34.0),
                    Button::new_button_b(22.0, 67.0),
                    Prize::new(8400.0, 5400.0)
                ),
                ClawMachine::new(
                    Button::new_button_a(26.0, 66.0),
                    Button::new_button_b(67.0, 21.0),
                    Prize::new(12748.0, 12176.0)
                ),
                ClawMachine::new(
                    Button::new_button_a(17.0, 86.0),
                    Button::new_button_b(84.0, 37.0),
                    Prize::new(7870.0, 6450.0)
                ),
                ClawMachine::new(
                    Button::new_button_a(69.0, 23.0),
                    Button::new_button_b(27.0, 71.0),
                    Prize::new(18641.0, 10279.0)
                ),
            ]
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(480, part_1(&parse_input(INPUT).unwrap()))
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            34393,
            part_1(&parse_input(&read_file_to_string("data/day13.txt")).unwrap())
        )
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(875318608908, part_2(&parse_input(INPUT).unwrap()))
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            83551068361379,
            part_2(&parse_input(&read_file_to_string("data/day13.txt")).unwrap())
        )
    }
}
