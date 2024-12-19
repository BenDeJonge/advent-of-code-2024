use std::ops::ControlFlow;

use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    error::Error,
    multi::{fold_many1, separated_list1},
    sequence::{separated_pair, terminated},
};

use crate::util::count_digits;

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Add,
    Multiply,
    Combine,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Calculation<T> {
    result: T,
    components: Vec<T>,
}

impl<T> Calculation<T> {
    pub fn new(result: T, components: Vec<T>) -> Self {
        Calculation { result, components }
    }
}

pub fn parse_input(input: &str) -> Vec<Calculation<u64>> {
    let (_input, parsed) = fold_many1(
        terminated(
            separated_pair(
                complete::u64::<&str, Error<_>>,
                tag(": "),
                separated_list1(tag(" "), complete::u64),
            ),
            line_ending,
        ),
        Vec::new,
        |mut acc: Vec<_>, (result, components)| {
            acc.push(Calculation::new(result, components));
            acc
        },
    )(input)
    .expect("should be able to parse input");
    parsed
}

fn backtrack(
    calc: &Calculation<u64>,
    operations: &mut Vec<Operation>,
    supported: &[Operation],
) -> bool {
    if operations.len() < calc.components.len() - 1 {
        for operation in supported {
            operations.push(*operation);
            if backtrack(calc, operations, supported) {
                return true;
            }
            operations.pop();
        }
        // No solution has been found.
        return false;
    }
    // Base case: the correct number of operations has been added.
    // TODO: check overflow through ControlFlow.
    is_ok(calc, operations)
}

fn is_ok(calc: &Calculation<u64>, operations: &[Operation]) -> bool {
    (1..(calc.components.len())).try_fold(calc.components[0], |mut acc, i| {
        let other = calc.components[i];
        match operations[i - 1] {
            Operation::Add => acc += other,
            Operation::Multiply => acc *= other,
            Operation::Combine => {
                acc = acc * 10u64.pow(count_digits(other)) + other;
            }
        }
        // Early return whenever the values get too large.
        if acc <= calc.result {
            ControlFlow::Continue(acc)
        } else {
            dbg!(calc, operations, acc);
            ControlFlow::Break(acc)
        }
    }) == ControlFlow::Continue(calc.result)
}

/// The sum of the results of all calculations that can be made using Add and Multiply.
pub fn part_1(calcs: &[Calculation<u64>]) -> u64 {
    calcs
        .iter()
        .filter(|calc| backtrack(calc, &mut vec![], &[Operation::Add, Operation::Multiply]))
        .map(|calc| calc.result)
        .sum()
}

/// The sum of the results of all calculations that can be made using Add, Multiply and Combine.
pub fn part_2(calcs: &[Calculation<u64>]) -> u64 {
    // TODO: include some early return that lets us know at which operation
    // index we started overflowing and pop all untill there.
    calcs
        .iter()
        .filter(|calc| {
            backtrack(
                calc,
                &mut vec![],
                &[Operation::Add, Operation::Multiply, Operation::Combine],
            )
        })
        .map(|calc| calc.result)
        .sum()
}
#[cfg(test)]
mod tests {

    use super::{parse_input, part_1, part_2};
    use crate::{day07::Calculation, util::read_file_to_string};
    const INPUT: &str = "190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20
";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT),
            &[
                Calculation::new(190, vec![10, 19]),
                Calculation::new(3267, vec![81, 40, 27]),
                Calculation::new(83, vec![17, 5]),
                Calculation::new(156, vec![15, 6]),
                Calculation::new(7290, vec![6, 8, 6, 15]),
                Calculation::new(161011, vec![16, 10, 13]),
                Calculation::new(192, vec![17, 8, 14]),
                Calculation::new(21037, vec![9, 7, 18, 13]),
                Calculation::new(292, vec![11, 6, 16, 20]),
            ]
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&parse_input(INPUT)), 3749)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&parse_input(&read_file_to_string("data/day07.txt"))),
            7710205485870
        )
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&parse_input(INPUT)), 11387)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&parse_input(&read_file_to_string("data/day07.txt"))),
            20928985450275
        )
    }
}
