use crate::util::Matrix;
use itertools::Itertools;
use nom::character::complete::line_ending;
use nom::error::Error;
use nom::multi::separated_list1;
use nom::{branch::alt, multi::many1};

pub fn parse_input(input: &str) -> Matrix<char> {
    let mut parser = separated_list1(
        line_ending::<&str, Error<_>>,
        many1(alt((
            nom::character::complete::char('X'),
            nom::character::complete::char('M'),
            nom::character::complete::char('A'),
            nom::character::complete::char('S'),
        ))),
    );
    let (_, output) = parser(input).expect("should be able to parse input");
    Matrix::new(output)
}

pub fn part_1(data: &Matrix<char>) -> usize {
    count_xmas_samx_in_iter(data.row_iter())
        + count_xmas_samx_in_iter(data.col_iter())
        + count_xmas_samx_in_iter(data.diagonal_iter())
        + count_xmas_samx_in_iter(data.antidiagonal_iter())
}

fn count_xmas_samx_in_iter<'a>(
    iter: impl Iterator<Item = impl Iterator<Item = &'a char>>,
) -> usize {
    let accepted = [(&'X', &'M', &'A', &'S'), (&'S', &'A', &'M', &'X')];
    iter.map(|iter| {
        iter.tuple_windows::<(_, _, _, _)>()
            .filter(|tuple| accepted.contains(tuple))
            .count()
    })
    .sum()
}

pub fn part_2(data: &Matrix<char>) -> usize {
    let mut score = 0;

    for row in 0..(data.shape()[0] - 2) {
        let top = get_row_as_char_vec(data, row).expect("i is in range");
        let middle = get_row_as_char_vec(data, row + 1).expect("i + 1 is in range");
        let bottom = get_row_as_char_vec(data, row + 2).expect("i + 2 is in range");
        for ((m, t), b) in middle.windows(3).zip(top.windows(3)).zip(bottom.windows(3)) {
            if m[1] != &'A' {
                continue;
            }
            // M . M
            // . A .
            // S . S
            if top_and_bottom_first_last_equals(t, b, ['M', 'M'], ['S', 'S']) {
                score += 1;
                continue;
            }
            // S . M
            // . A .
            // S . M
            if top_and_bottom_first_last_equals(t, b, ['S', 'M'], ['S', 'M']) {
                score += 1;
                continue;
            }
            // S . S
            // . A .
            // M . M
            if top_and_bottom_first_last_equals(t, b, ['S', 'S'], ['M', 'M']) {
                score += 1;
                continue;
            }
            // M . S
            // . A .
            // M . S
            if top_and_bottom_first_last_equals(t, b, ['M', 'S'], ['M', 'S']) {
                score += 1;
                continue;
            }
        }
    }
    score
}

fn get_row_as_char_vec<T>(data: &Matrix<T>, index: usize) -> Option<Vec<&T>> {
    data.row(index).map(|r| r.collect::<Vec<&T>>())
}

fn top_and_bottom_first_last_equals<T>(
    top: &[&T],
    bottom: &[&T],
    top_equals: [T; 2],
    bottom_equals: [T; 2],
) -> bool
where
    T: PartialEq,
{
    top[0] == &top_equals[0]
        && top[top.len() - 1] == &top_equals[1]
        && bottom[0] == &bottom_equals[0]
        && bottom[bottom.len() - 1] == &bottom_equals[1]
}

#[cfg(test)]
mod tests {
    use super::{parse_input, part_1, part_2};
    use crate::util::{read_file_to_string, Matrix};
    const INPUT: &str = "MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT),
            Matrix::new(vec![
                vec!['M', 'M', 'M', 'S', 'X', 'X', 'M', 'A', 'S', 'M'],
                vec!['M', 'S', 'A', 'M', 'X', 'M', 'S', 'M', 'S', 'A'],
                vec!['A', 'M', 'X', 'S', 'X', 'M', 'A', 'A', 'M', 'M'],
                vec!['M', 'S', 'A', 'M', 'A', 'S', 'M', 'S', 'M', 'X'],
                vec!['X', 'M', 'A', 'S', 'A', 'M', 'X', 'A', 'M', 'M'],
                vec!['X', 'X', 'A', 'M', 'M', 'X', 'X', 'A', 'M', 'A'],
                vec!['S', 'M', 'S', 'M', 'S', 'A', 'S', 'X', 'S', 'S'],
                vec!['S', 'A', 'X', 'A', 'M', 'A', 'S', 'A', 'A', 'A'],
                vec!['M', 'A', 'M', 'M', 'M', 'X', 'M', 'M', 'M', 'M'],
                vec!['M', 'X', 'M', 'X', 'A', 'X', 'M', 'A', 'S', 'X'],
            ])
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&parse_input(INPUT)), 18)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&parse_input(&read_file_to_string("data/day04.txt"))),
            2427
        );
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&parse_input(INPUT)), 9)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&parse_input(&read_file_to_string("data/day04.txt"))),
            1900
        )
    }
}
