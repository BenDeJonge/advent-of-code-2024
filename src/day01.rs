use std::cmp;

use crate::util::parse_decimal;
use nom::character::complete::space1;
use nom::sequence::separated_pair;

fn parse_input<T>(input: &str) -> [Vec<T>; 2]
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let lines = input.lines();
    let mut left = Vec::<T>::new();
    let mut right = Vec::<T>::new();
    let mut parser = separated_pair(parse_decimal::<T>, space1, parse_decimal::<T>);
    for line in lines {
        let output = parser(line).expect("every line is \"<int>    <int>\"");
        left.push(output.1 .0);
        right.push(output.1 .1);
    }
    [left, right]
}

fn part_1<T>(data: &mut [Vec<T>; 2]) -> T
where
    T: std::cmp::Ord
        + std::ops::Sub
        + std::iter::Sum<<T as std::ops::Sub>::Output>
        + Copy
        + num_traits::Signed
        + std::fmt::Debug,
{
    data[0].sort();
    data[1].sort();
    data[0]
        .iter()
        .zip(data[1].iter())
        .map(|(&l, &r)| num_traits::sign::abs(l - r))
        .sum()
}

fn part_2(data: &mut [Vec<isize>; 2]) -> isize {
    data[0].sort();
    data[1].sort();
    // Otherwise, the last number gets ignored. Remove this afterwards.
    data[0].push(0);
    // data[1].push(0);
    let mut current = *data[0].first().expect("data[0] should not be empty");
    // Counting the number of occurences in both vectors.
    let mut n_left: isize = 0;
    let mut n_right: isize = 0;
    // Use a two pointer approach to keep track of positioning in both vectors.
    let mut i_left: usize = 0;
    let mut i_right: usize = 0;
    let mut score: isize = 0;
    while i_left < data[0].len() {
        let number = data[0][i_left];
        // Looping over number instead of indices would miss number that only
        // occur once in the left vector. With indices, we avoid incrementing
        // i_left on the first occurence.
        if number == current {
            n_left += 1;
            i_left += 1;
        } else {
            // Skipping past all the too small numbers.
            while i_right < data[1].len() {
                let other = data[1][i_right];
                match other.cmp(&current) {
                    cmp::Ordering::Equal => {
                        i_right += 1;
                        n_right += 1;
                    }
                    cmp::Ordering::Greater => break,
                    cmp::Ordering::Less => {
                        i_right += 1;
                    }
                }
            }
            score += current * n_left * n_right;
            n_left = 0;
            n_right = 0;
            current = number;
        }
    }
    // Removing the temporary addition to the left vector.
    data[0].pop();
    score
}

#[cfg(test)]
mod tests {
    use super::{parse_input, part_1, part_2};
    use crate::util::read_file_to_string;
    const INPUT: &str = "3   4
4   3
2   5`
1   3
3   9
3   3";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            &parse_input::<usize>(INPUT),
            &[[3, 4, 2, 1, 3, 3], [4, 3, 5, 3, 9, 3]]
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&mut parse_input::<isize>(INPUT)), 11)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&mut parse_input::<isize>(&read_file_to_string(
                "data/day01.txt"
            ))),
            1320851
        );
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&mut parse_input::<isize>(INPUT)), 31)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&mut parse_input::<isize>(&read_file_to_string(
                "data/day01.txt"
            ))),
            26859182
        )
    }
}
