use std::cmp;

use crate::util::parse_decimal;
use nom::bytes::complete::tag;
use nom::multi::separated_list1;

#[derive(Clone, Copy, PartialEq)]
enum Gradient {
    Ascending,
    Descending,
}

pub fn parse_input<T>(input: &str) -> Vec<Vec<T>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let mut parser = separated_list1(tag(" "), parse_decimal);
    let mut buffer = vec![];
    for line in input.lines() {
        let output = parser(line).expect("every line is `<int> <int>`");
        buffer.push(output.1)
    }
    buffer
}

fn is_ok<T>(data: &[T], max_delta: isize) -> bool
where
    T: Copy + Into<isize> + std::ops::Sub<Output = T>,
{
    let mut is_ok = true;
    let mut gradient = None;
    for delta in data.windows(2).map(|window| (window[0] - window[1]).into()) {
        let gradient_next = match delta.cmp(&0isize) {
            // The delta between neighbors must be at least 1.
            cmp::Ordering::Equal => {
                is_ok = false;
                break;
            }
            cmp::Ordering::Greater => Some(Gradient::Descending),
            cmp::Ordering::Less => Some(Gradient::Ascending),
        };
        // Get the gradient based on the first delta.
        if gradient.is_none() {
            gradient = gradient_next;
        }
        // Inconsistent gradients are a failure.
        if gradient != gradient_next {
            is_ok = false;
            break;
        }
        // Excessive gradients are a failure.
        if delta.abs() > max_delta {
            is_ok = false;
            break;
        }
    }
    is_ok
}

/// Compute how many reports are safe.
/// A report is considered safe if:
/// - the absolute difference between all neighboring elements is in 1..=3.
/// - the vector of number is monotonic.
pub fn part_1<T>(data: &[Vec<T>]) -> usize
where
    T: Copy + Into<isize> + std::ops::Sub<Output = T>,
{
    const MAX_DELTA: isize = 3;
    data.iter().map(|vec| is_ok(vec, MAX_DELTA) as usize).sum()
}

fn try_remove<T>(vec: &[T], idx: usize, max_delta: isize) -> bool
where
    T: std::marker::Copy + std::clone::Clone + Into<isize> + std::ops::Sub<Output = T>,
{
    let mut cloned = Vec::with_capacity(vec.len());
    vec.clone_into(&mut cloned);
    cloned.remove(idx);
    is_ok(&cloned, max_delta)
}

/// | Data                   | Window delta       | Removal        | Ok  |
/// |------------------------|--------------------|----------------|-----|
/// | `[ 7,  6,  4,  2,  1]` | `[ 1,  2,  2,  1]` | /              | Yes |
/// | `[ 1,  2,  7,  8,  9]` | `[-1, -5, -1, -1]` | /              | No  |
/// | `[ 9,  7,  6,  2,  1]` | `[ 2,  1,  4,  1]` | /              | No  |
/// | `[ 1,  3,  2,  4,  5]` | `[-2,  1, -2, -1]` | `[-1, -2, -1]` | Yes |
/// | `[ 8,  6,  4,  4,  1]` | `[ 2,  2,  0,  3]` | `[ 2,  2,  3]` | Yes |
/// | `[ 1,  3,  6,  7,  9]` | `[-2, -3, -1, -2]` | /              | Yes |
/// | `[10,  1,  2,  3,  4]` | `[ 9, -1, -1, -1]` | `[-1, -1, -1]` | Yes |
/// | `[ 1,  2,  3,  4, 10]` | `[-1, -1, -1, -6]` | `[-1, -1, -1]` | Yes |
///
/// [2, 3, 7, 6, 9] -> [-1, -4,  1, -3] -> [-1, -3, -3]
/// [9, 6, 7, 3, 2] -> [ 3, -1,  4,  1] -> [ 3, 3, 1]
///
/// By investigating the above table, we can see that:
/// - `[1, 3, 2, 4, 5]` can be fixed by removing the 3 at index 1 or the 2 at
///   index 2.
/// - `[8, 6, 4, 4, 1]` can be fixed by removing the 4 at index 2 or 3.
/// - `[10,  1,  2,  3,  4]` and `[ 1,  2,  3,  4, 10]` can be fixed by removing
///   the 10 at the first and last index, respectively.
///
/// For both of these, the removal column can be computed by adding the delta at
/// the removed index to the previous delta. This can be proven as follows:
/// - A delta `x` between two neighboring values `a` and `b` can be computed as:
///   `x = a - b`.
/// - The next delta `y` between the two subsequent values is then, analogously:
///   `y = b - c`.
/// - When removing `b` would fix the sequence, both deltas `x` and `y` can
///   simply be replaced by their sum, shortening the sequence by one element:
///   `a - c = (x + b) - (b - y) = x + y`.
/// - If `b` is positioned at either end of the sequence, the respective delta
///   can simply be removed.
///
/// This could be the basis for a slightly more efficient algorithm that solves
/// the question in a single pass.
pub fn part_2<T>(data: &[Vec<T>]) -> usize
where
    T: Copy + Into<isize> + std::ops::Sub<Output = T>,
{
    const MAX_DELTA: isize = 3;
    let mut score = 0;
    for vec in data {
        if is_ok(vec, MAX_DELTA) {
            score += 1;
            continue;
        }
        if try_remove(vec, 0, MAX_DELTA) || try_remove(vec, vec.len() - 1, MAX_DELTA) {
            score += 1;
            continue;
        }
        // We can remove the extreme bounds (1..(vec.len() - 2)) because these
        // were already checked in the clause above.
        for i in 1..(vec.len() - 2) {
            // Any delta is outside of the wanted range.
            let delta1: isize = (vec[i] - vec[i + 1]).into();
            if !(1..=3).contains(&delta1.abs())
                && (try_remove(vec, i, MAX_DELTA) || try_remove(vec, i + 1, MAX_DELTA))
            {
                score += 1;
                break;
            }
            // Two deltas are inconsistent (negative, positive): [1, 3, 2].
            let delta2: isize = (vec[i + 1] - vec[i + 2]).into();
            if delta1.signum() != delta2.signum()
                && (try_remove(vec, i, MAX_DELTA)
                    || try_remove(vec, i + 1, MAX_DELTA)
                    || try_remove(vec, i + 2, MAX_DELTA))
            {
                score += 1;
                break;
            }
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::{parse_input, part_1, part_2};
    use crate::util::read_file_to_string;
    const INPUT: &str = "7 6 4 2 1\n1 2 7 8 9\n9 7 6 2 1\n1 3 2 4 5\n8 6 4 4 1\n1 3 6 7 9";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            &parse_input::<usize>(INPUT),
            &[
                [7, 6, 4, 2, 1], // [ 1,  2,  2,  1] -> Ok
                [1, 2, 7, 8, 9], // [-1, -5, -1, -1] -> Not ok
                [9, 7, 6, 2, 1], // [ 2,  1,  4,  1] -> Not ok
                [1, 3, 2, 4, 5], // [-2,  1, -2, -1] -> Remove idx 1: [1, 2, 4, 5] = [-1, -2, -1] -> Ok
                [8, 6, 4, 4, 1], // [ 2,  2,  0,  3] -> Remove idx 2: [2, 2, 3] -> Ok
                [1, 3, 6, 7, 9], // [-2, -3, -1, -2] -> Ok
            ]
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&(parse_input::<isize>(INPUT))), 2)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&parse_input::<isize>(&read_file_to_string(
                "data/day02.txt"
            ))),
            639
        );
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&parse_input::<isize>(INPUT)), 4)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&parse_input::<isize>(&read_file_to_string(
                "data/day02.txt"
            ))),
            674
        )
    }
}
