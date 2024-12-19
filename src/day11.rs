use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use nom::{bytes::complete::tag, error::Error, multi::separated_list1};

use crate::util::{count_digits, hashmap_add_or_default};

#[derive(Debug, PartialEq)]
pub struct Stones<T>(HashMap<T, usize>)
where
    T: std::hash::Hash + std::cmp::Eq;

impl<T> Deref for Stones<T>
where
    T: std::hash::Hash + std::cmp::Eq,
{
    type Target = HashMap<T, usize>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Stones<T>
where
    T: std::hash::Hash + std::cmp::Eq,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Stones<T>
where
    T: std::hash::Hash + std::cmp::Eq + std::marker::Copy,
{
    pub fn new(stones: &[T]) -> Self {
        let mut hashmap = HashMap::<T, usize>::new();
        for &stone in stones.iter() {
            hashmap_add_or_default(&mut hashmap, stone, 1);
        }
        Stones(hashmap)
    }

    pub fn count(&self) -> usize {
        self.values().sum()
    }
}

impl Stones<u64> {
    pub fn take_step(&mut self) {
        let mut new_hashmap = HashMap::<u64, usize>::new();
        for (&stone, &count) in self.iter() {
            if stone == 0 {
                // new_hashmap.insert(1, count);
                hashmap_add_or_default(&mut new_hashmap, 1, count);
                continue;
            }
            let digits = count_digits(stone);
            if digits % 2 == 0 {
                let power = 10u64.pow(digits / 2);
                // new_hashmap.insert(stone / power, count);
                // new_hashmap.insert(stone % power, count);
                hashmap_add_or_default(&mut new_hashmap, stone / power, count);
                hashmap_add_or_default(&mut new_hashmap, stone % power, count);
            } else {
                // new_hashmap.insert(stone * 2024, count);
                hashmap_add_or_default(&mut new_hashmap, stone * 2024, count);
            }
        }
        self.0 = new_hashmap;
    }
}

pub fn parse_input(input: &str) -> Stones<u64> {
    let mut parser = separated_list1(tag(" "), nom::character::complete::u64::<&str, Error<_>>);
    let (_, output) = parser(input).expect("should be able to parse input");
    Stones::new(&output)
}

/// Count the number of stones after 25 moves, using the following rules:
/// - Value 0 becomes 1.
/// - Even number of digits becomes 2 values with equally split digits,
///   ignoring leading zeros.
/// - Else, a value becomes 2024 x original.
pub fn part_1(stones: &mut Stones<u64>) -> usize {
    for _ in 0..25 {
        stones.take_step();
    }
    stones.count()
}

/// For each number in the first vector calculate the value times the number of
/// occurences in the second vector, and sum all these results.
pub fn part_2(stones: &mut Stones<u64>) -> usize {
    for _ in 0..75 {
        stones.take_step();
    }
    stones.count()
}

#[cfg(test)]
mod tests {
    use super::{parse_input, part_1, part_2};
    use crate::{day11::Stones, util::read_file_to_string};
    const INPUT: &str = "125 17";

    #[test]
    fn test_parse_input() {
        assert_eq!(parse_input(INPUT), Stones::new(&[125, 17]))
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&mut parse_input(INPUT)), 55312)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&mut parse_input(&read_file_to_string("data/day11.txt"))),
            193899
        );
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&mut parse_input(INPUT)), 65601038650482)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&mut parse_input(&read_file_to_string("data/day11.txt"))),
            229682160383225
        )
    }
}
