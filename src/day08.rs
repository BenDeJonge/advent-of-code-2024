use std::collections::{HashMap, HashSet};

use crate::util::Coordinate;

#[derive(Debug, PartialEq)]
pub struct SparseMatrix<T>
where
    T: std::cmp::Eq + std::hash::Hash,
{
    shape: [usize; 2],
    elements: HashMap<T, Vec<Coordinate>>,
}

impl<T> SparseMatrix<T>
where
    T: std::cmp::Eq + std::hash::Hash,
{
    pub fn find_nodes(&self, n: Option<usize>) -> HashSet<Coordinate> {
        let mut hashset = HashSet::<Coordinate>::new();
        for (_, locations) in self.elements.iter() {
            for i in 0..(locations.len() - 1) {
                let antenna1 = locations[i];
                for &antenna2 in locations.iter().skip(i + 1) {
                    self.calc_antenna_pair(antenna1, antenna2, n, &mut hashset);
                }
            }
        }
        hashset
    }

    /// Compute where nodes will be positioned relative to any antenna pair.
    /// a = [a1, a2]
    /// b = [b1, b2]
    /// d = a - b = [a1 - b1, a2 - b2]
    /// n1 = a + d = [a1 + a1 - b1, a2 + a2 - b2] = [2a1 - b1, 2a2 - b2]
    /// n2 = b - d = [b1 - (a1 - b1), b2 - (a2 - b2)] = [2b1 - a1, 2b2 - a2]
    ///
    /// * `a1`, `a2`: the antenna pair in question
    /// * `n`: the number of nodes to compute, `None` for all.
    /// * `hashset`: mutable reference to the `HashSet` storing all nodes.
    fn calc_antenna_pair(
        &self,
        a1: Coordinate,
        a2: Coordinate,
        n: Option<usize>,
        hashset: &mut HashSet<Coordinate>,
    ) {
        let delta = a1 - a2;
        let origin = Coordinate::new(0, 0);
        let topright = Coordinate::from([
            self.shape[0].try_into().expect("shape fits in i32"),
            self.shape[1].try_into().expect("shape fits in i32"),
        ]);
        let nodes1 = (1isize..)
            .map(|i| a1 + delta * i)
            .take_while(|sum| sum.is_in(&origin, &topright));
        let nodes2 = (1isize..)
            .map(|i| a2 - delta * i)
            .take_while(|sum| sum.is_in(&origin, &topright));
        if let Some(n) = n {
            hashset.extend(nodes1.take(n));
            hashset.extend(nodes2.take(n));
        } else {
            hashset.extend(nodes1);
            hashset.extend(nodes2);
            // When calculating all nodes, every antenna is also a node.
            hashset.insert(a1);
            hashset.insert(a2);
        }
    }
}

pub fn parse_input(input: &str) -> SparseMatrix<char> {
    const IGNORE: char = '.';
    let mut elements = HashMap::<char, Vec<Coordinate>>::new();
    let mut shape = [0, 0];
    let mut row_map = HashMap::<char, Vec<isize>>::new();
    for (i, row) in input.lines().enumerate() {
        shape[0] = row.len();
        parse_row(&mut row_map, row, IGNORE);
        for (char, row) in row_map.iter_mut() {
            elements
                .entry(*char)
                .and_modify(|vec| {
                    vec.extend(
                        row.iter()
                            .map(|el| Coordinate::from([i.try_into().unwrap(), *el])),
                    );
                })
                .or_insert(
                    row.iter()
                        .map(|el| Coordinate::from([i.try_into().unwrap(), *el]))
                        .collect(),
                );
        }
        row_map.clear();
        shape[1] = i + 1;
    }
    SparseMatrix { shape, elements }
}

fn parse_row(hashmap: &mut HashMap<char, Vec<isize>>, row: &str, ignore: char) {
    for (i, ch) in row.chars().enumerate().filter(|(_i, ch)| *ch != ignore) {
        hashmap
            .entry(ch)
            .and_modify(|vec| vec.push(i.try_into().unwrap()))
            .or_insert(vec![i.try_into().unwrap()]);
    }
}

/// Count all the nodes created from antenna with the same symbol.
pub fn part_1<T>(matrix: &SparseMatrix<T>) -> usize
where
    T: std::cmp::Eq,
    T: std::hash::Hash,
{
    matrix.find_nodes(Some(1)).len()
}

/// Count all nodes created from antenna with the same symbol. Nodes are placed
/// at any integer multiple of the offset, while remaining inside the shape of
/// the matrix. As a result, every antenna is also a node.
pub fn part_2<T>(matrix: &SparseMatrix<T>) -> usize
where
    T: std::cmp::Eq + std::hash::Hash,
{
    matrix.find_nodes(None).len()
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::{parse_input, part_1, part_2};
    use crate::{
        day08::SparseMatrix,
        util::{read_file_to_string, Coordinate},
    };
    const INPUT: &str = "............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT),
            SparseMatrix {
                shape: [12, 12],
                elements: HashMap::from([
                    (
                        '0',
                        vec![
                            Coordinate::from([1, 8]),
                            Coordinate::from([2, 5]),
                            Coordinate::from([3, 7]),
                            Coordinate::from([4, 4])
                        ]
                    ),
                    (
                        'A',
                        vec![
                            Coordinate::from([5, 6]),
                            Coordinate::from([8, 8]),
                            Coordinate::from([9, 9])
                        ]
                    ),
                ])
            }
        )
    }

    #[test]
    fn test_part_1_small() {
        assert_eq!(part_1(&parse_input(INPUT)), 14)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&parse_input(&read_file_to_string("data/day08.txt"))),
            265
        )
    }

    #[test]
    fn test_part_2_small() {
        assert_eq!(part_2(&parse_input(INPUT)), 34)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&parse_input(&read_file_to_string("data/day08.txt"))),
            962
        )
    }
}
