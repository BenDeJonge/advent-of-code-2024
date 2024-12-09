use std::cmp;
use std::collections::HashMap;

use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::{fold_many1, separated_list1};
use nom::sequence::terminated;
use nom::IResult;
use nom::{character::complete, sequence::separated_pair};

pub fn parse_input(input: &str) -> (HashMap<u32, Vec<u32>>, Vec<Vec<u32>>) {
    let (_, output) = separated_pair(parse_rules, line_ending, parse_pages)(input)
        .expect("should be able to parse input");
    output
}

fn parse_rules(input: &str) -> IResult<&str, HashMap<u32, Vec<u32>>> {
    let (input, mut map) = fold_many1(
        terminated(
            separated_pair(complete::u32, tag("|"), complete::u32),
            line_ending,
        ),
        HashMap::default,
        |mut map, (page, after)| {
            map.entry(page)
                .and_modify(|vec: &mut Vec<_>| vec.push(after))
                .or_insert(vec![after]);
            map
        },
    )(input)?;
    for (_k, v) in map.iter_mut() {
        v.sort();
    }
    Ok((input, map))
}

fn parse_pages(input: &str) -> IResult<&str, Vec<Vec<u32>>> {
    separated_list1(line_ending, separated_list1(tag(","), complete::u32))(input)
}

/// Take the sum of the middle numbers of the pages that are sorted according to the rules.
pub fn part_1<T>(rules: &HashMap<T, Vec<T>>, pages: &[Vec<T>]) -> T
where
    T: std::cmp::Eq + std::hash::Hash + std::cmp::Ord + std::iter::Sum<T> + std::marker::Copy,
{
    pages
        .iter()
        .filter(|page| {
            page.is_sorted_by(|a, b| {
                rules
                    .get(b)
                    .is_none_or(|after| after.binary_search(a).is_err())
            })
        })
        .map(|page| *page.get(page.len() / 2).expect("page should not be empty"))
        .sum()
}

/// For all pages that are not sorted according to the rules, fix their sorting
/// and take the sum of their middle numbers.
pub fn part_2<T>(rules: &HashMap<T, Vec<T>>, pages: &mut [Vec<T>]) -> T
where
    T: std::cmp::Eq + std::hash::Hash + std::cmp::Ord + std::iter::Sum<T> + std::marker::Copy,
{
    pages
        .iter_mut()
        .filter(|page| {
            !(page.is_sorted_by(|a, b| {
                rules
                    .get(b)
                    .is_none_or(|after| after.binary_search(a).is_err())
            }))
        })
        .map(|page: &mut Vec<T>| {
            page.sort_by(|a, b| {
                if rules
                    .get(b)
                    .is_none_or(|after| after.binary_search(a).is_err())
                {
                    cmp::Ordering::Less
                } else {
                    cmp::Ordering::Greater
                }
            });
            *page.get(page.len() / 2).expect("page should not be empty")
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{parse_input, part_1, part_2};
    use crate::util::read_file_to_string;
    const INPUT: &str = "47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT),
            (
                HashMap::from([
                    (47, vec![13, 29, 53, 61]),
                    (97, vec![13, 29, 47, 53, 61, 75]),
                    (75, vec![13, 29, 47, 53, 61]),
                    (61, vec![13, 29, 53]),
                    (29, vec![13]),
                    (53, vec![13, 29]),
                ]),
                vec![
                    vec![75, 47, 61, 53, 29], // Ok
                    vec![97, 61, 53, 29, 13], // Ok
                    vec![75, 29, 13],         // Ok
                    vec![75, 97, 47, 61, 53], // [97, 75, 47, 61, 53]
                    vec![61, 13, 29],         // [61, 29, 13]
                    vec![97, 13, 75, 29, 47], // [97, 75, 47, 29, 13]
                ]
            )
        )
    }

    #[test]
    fn test_part_1_small() {
        let (map, pages) = parse_input(INPUT);
        assert_eq!(part_1(&map, &pages), 143)
    }

    #[test]
    fn test_part_1_full() {
        let (map, pages) = parse_input(&read_file_to_string("data/day05.txt"));
        assert_eq!(part_1(&map, &pages), 7198)
    }

    #[test]
    fn test_part_2_small() {
        let (map, mut pages) = parse_input(INPUT);
        assert_eq!(part_2(&map, &mut pages), 123)
    }

    #[test]
    fn test_part_2_full() {
        let (map, mut pages) = parse_input(&read_file_to_string("data/day05.txt"));
        assert_eq!(part_2(&map, &mut pages), 4230)
    }
}
