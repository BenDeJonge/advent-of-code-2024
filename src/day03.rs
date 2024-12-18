use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{self, anychar};
use nom::combinator::value;
use nom::multi::{many0, many_till};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Mul(u32, u32),
    Do,
    Dont,
}

pub fn parse_input(input: &str) -> Vec<Instruction> {
    let mut buffer = <Vec<Instruction>>::new();
    let mut parser = many0(many_till(anychar, parse_instruction));
    for line in input.lines() {
        let (_, result) = parser(line).expect("should be able to parse line");
        buffer.extend(result.iter().map(|(_chars, instr)| *instr));
    }
    buffer
}

fn parse_instruction_mul(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag("mul")(input)?;
    let (input, pair) = delimited(
        tag("("),
        separated_pair(complete::u32, tag(","), complete::u32),
        tag(")"),
    )(input)?;
    Ok((input, Instruction::Mul(pair.0, pair.1)))
}

fn parse_instruction(input: &str) -> IResult<&str, Instruction> {
    alt((
        value(Instruction::Do, tag("do()")),
        value(Instruction::Dont, tag("don't()")),
        parse_instruction_mul,
    ))(input)
}

/// Compute the sum of all valid multiplications in the instruction set.
/// An instruction is valid if it is of the form:
/// ```regex
/// mul\(\d+,\d+\)
/// ```
pub fn part_1(data: &[Instruction]) -> u32 {
    data.iter().fold(0, |acc, instr| match instr {
        Instruction::Mul(l, r) => acc + l * r,
        _ => acc,
    })
}

/// Compute the sum of all valid multiplications in the instruction set.
/// An instruction is valid if:
/// - it is of the form:
/// ```regex
/// mul\(\d+,\d+\)
/// ```
/// - the current state is `do`, not `don't`. The state is toggled whenever the
/// corresponding instruction is encountered.
pub fn part_2(data: &[Instruction]) -> u32 {
    data.iter()
        .fold((Instruction::Do, 0), |(state, acc), instr| match instr {
            Instruction::Mul(l, r) => match state {
                Instruction::Do => (state, acc + l * r),
                Instruction::Dont => (state, acc),
                _ => unreachable!(),
            },
            switch_state => (*switch_state, acc),
        })
        .1
}

#[cfg(test)]
mod tests {
    use super::{parse_input, part_1, part_2, Instruction};
    use crate::util::read_file_to_string;
    const INPUT: &str = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            &parse_input(INPUT),
            &[
                Instruction::Mul(2, 4),
                Instruction::Dont,
                Instruction::Mul(5, 5),
                Instruction::Mul(11, 8),
                Instruction::Do,
                Instruction::Mul(8, 5),
            ]
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&parse_input(INPUT)), 161)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&parse_input(&read_file_to_string("data/day03.txt"))),
            188741603
        );
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&parse_input(INPUT)), 48)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&parse_input(&read_file_to_string("data/day03.txt"))),
            67269798
        )
    }
}
