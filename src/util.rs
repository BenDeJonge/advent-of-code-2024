use nom::{character::complete::one_of, combinator::recognize, multi::many1, IResult, Parser};
use std::fs::{read_to_string, File};
use std::io;
use std::io::BufRead;
use std::ops::{Deref, DerefMut};
use std::path::Path;

pub fn read_file_to_string<P>(filename: P) -> String
where
    P: AsRef<Path>,
{
    read_to_string(filename).expect("Should have been able to read the file")
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
pub fn read_file_to_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
/// A nom parser to identify decimal numbers.
pub fn parse_decimal<T>(input: &str) -> IResult<&str, T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    match recognize(many1(one_of("0123456789"))).parse(input) {
        Ok(output) => Ok((
            output.0,
            output.1.parse::<T>().expect("Should contain only digits"),
        )),
        Err(e) => Err(e),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Matrix<T>(Vec<Vec<T>>);

// Allows direct access to the structs only internal attribute.
impl<T> Deref for Matrix<T> {
    type Target = Vec<Vec<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Matrix<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Matrix<T> {
    /// This checks if all rows have the same column count
    /// and if so, moves the data in to the Matrix.
    pub fn new(data: Vec<Vec<T>>) -> Self {
        if let Some(row0) = data.first() {
            for (i, row) in data.iter().enumerate() {
                assert!(
                    row.len() == row0.len(),
                    "row {} has len {} while row 0 has len {}",
                    i,
                    row.len(),
                    row0.len()
                )
            }
        }
        Self(data)
    }

    pub fn shape(&self) -> [usize; 2] {
        [
            self.len(),
            self.first().expect("first vector is not empty").len(),
        ]
    }

    pub fn get_element(&self, idx: [usize; 2]) -> Option<&T> {
        self.get(idx[0]).and_then(|row| row.get(idx[1]))
    }

    pub fn set_element(&mut self, idx: [usize; 2], value: T) -> Option<()> {
        if idx[0] < self.shape()[0] && idx[1] < self.shape()[1] {
            self[idx[0]][idx[1]] = value;
            Some(())
        } else {
            None
        }
    }

    pub fn row(&self, index: usize) -> Option<impl Iterator<Item = &T>> {
        if index >= self.shape()[0] {
            return None;
        }
        Some(self[index][0..self.shape()[1]].iter())
    }

    pub fn row_iter(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        (0..self.shape()[0]).map(|index| self.row(index).unwrap())
    }

    pub fn col(&self, index: usize) -> Option<impl Iterator<Item = &T>> {
        if index >= self.shape()[1] {
            return None;
        }
        Some((0..self.shape()[0]).map(move |row_index| &self[row_index][index]))
    }

    pub fn col_iter(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        (0..self.shape()[1]).map(|index| self.col(index).unwrap())
    }

    /// Get the diagonal (going top-left to bottom-right) at the index.
    /// Indices are counted clockwise along the outside of the matrix from the
    /// bottom-left corner to the top-right corner e.g., diagonal 2 and 3 are
    /// marked for a 3 x 4 matrix:
    ///
    /// ```text
    ///              ↘ ↘
    /// [2 3 4 5]    [2 3 4 5]
    /// [1 . . .] -> [1 2 3 4]
    /// [0 . . .]    [0 1 2 3]
    ///                     ↘ ↘
    /// ```
    ///
    /// For a general matrix of `r` rows and `c` columns, the indices will span
    /// the range `0..=(r + c - 2)`.
    ///
    /// # Example usage
    ///
    /// ```rust
    /// use advent_of_code_2024::util::Matrix;
    ///
    /// let matrix = Matrix::new(vec![
    ///     vec![ 0,  1,  2,  3],
    ///     vec![ 4,  5,  6,  7],
    ///     vec![ 8,  9, 10, 11],
    /// ]);
    ///
    /// for (el1, el2) in matrix
    ///     .diagonal(3)
    ///     .expect("i is within bounds")
    ///     .zip([1, 6, 11].iter())
    /// {
    ///     assert_eq!(el1, el2)
    /// }
    /// ```
    pub fn diagonal(&self, index: usize) -> Option<impl Iterator<Item = &T>> {
        // Compute a starting position from the diagonal index that moves
        // clockwise along left and top edges of the matrix.
        let [n_rows, n_cols] = self.shape();
        if index > n_rows + n_cols - 2 {
            return None;
        }
        let start = match index < n_rows {
            true => [n_rows - 1 - index, 0],
            false => [0, index - n_rows + 1],
        };
        // Move to the bottom-right with a [1, 1] offset until the matrix edge.
        Some(
            (0..(n_cols).min(n_rows))
                .take_while(move |offset| start[0] + offset < n_rows && start[1] + offset < n_cols)
                .map(move |offset| &self[start[0] + offset][start[1] + offset]),
        )
    }

    pub fn diagonal_iter(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        (0..(self.shape().iter().sum::<usize>() - 2)).map(|index| self.diagonal(index).unwrap())
    }

    /// Get the antidiagonal (going top-right to bottom-left) at the index.
    /// Indices are counted clockwise along the outside of the matrix from the
    /// top-left corner to the bottom-right corner e.g., diagonal 2 and 3 are
    /// marked for a 3 x 4 matrix:
    ///
    /// ```text
    ///                    ↗ ↗
    /// [0 1 2 3]    [0 1 2 3]
    /// [. . . 4] -> [1 2 3 4]
    /// [. . . 5]    [2 3 4 5]
    ///             ↗ ↗      
    /// ```
    ///
    /// For a general matrix of `r` rows and `c` columns, the indices will span
    /// the range `0..=(r + c - 2)`.
    ///
    /// # Example usage
    ///
    /// ```rust
    /// use advent_of_code_2024::util::Matrix;
    ///
    /// let matrix = Matrix::new(vec![
    ///     vec![0, 1, 2, 3],   //
    ///     vec![4, 5, 6, 7],   //
    ///     vec![8, 9, 10, 11], //
    /// ]);
    ///
    /// for (el1, el2) in matrix
    ///     .antidiagonal(3)
    ///     .expect("i is within bounds")
    ///     .zip([9, 6 ,3].iter())
    /// {
    ///     assert_eq!(el1, el2)
    /// }
    /// ```
    pub fn antidiagonal(&self, index: usize) -> Option<impl Iterator<Item = &T>> {
        // Compute a starting position from the diagonal index that moves
        // clockwise along top and right edges of the matrix.
        let [n_rows, n_cols] = self.shape();
        if index > n_rows + n_cols - 2 {
            return None;
        }
        let start = match index < n_rows {
            true => [index, 0],
            false => [n_rows - 1, index - n_rows + 1],
        };
        // Move to the top-right with a [-1, 1] offset until the matrix edge.
        Some(
            (0..(n_cols).min(n_rows))
                .take_while(move |offset| start[0] >= *offset && start[1] + offset < n_cols)
                .map(move |offset| &self[start[0] - offset][start[1] + offset]),
        )
    }

    pub fn antidiagonal_iter(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        (0..(self.shape().iter().sum::<usize>() - 2)).map(|index| self.antidiagonal(index).unwrap())
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::{parse_decimal, Matrix};
    use nom::{bytes::complete::tag, sequence::separated_pair};

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse_decimal("123"), Ok(("", 123)));
        assert_eq!(parse_decimal("0456"), Ok(("", 456)));
        assert_eq!(parse_decimal("789 abc"), Ok((" abc", 789)));
        // Thousands separators are not supported.
        assert_eq!(parse_decimal("1_000_000"), Ok(("_000_000", 1)));
        // assert_eq!(parse_decimal("not a number"), Err(IResult::Err("not a number", OneOf)))
        //     Error(
        //         Error {
        //             input: "not a number",
        //             code: OneOf,
        //         },
        //     ),
        // )
    }

    #[test]
    /// Test if the `parse_decimal` function can be used in conjuction with
    /// standard nom functionalities.
    fn test_parse_decimal_with_nom() {
        let mut parser = separated_pair(parse_decimal, tag(","), parse_decimal);
        let input = "1,2\n3,4\n5,6";
        let mut left = Vec::<usize>::with_capacity(3);
        let mut right = Vec::<usize>::with_capacity(3);
        for line in input.lines() {
            let output = parser(line).expect("should not error");
            assert!(output.0.is_empty());
            left.push(output.1 .0);
            right.push(output.1 .1);
        }
        assert_eq!(&left, &[1, 3, 5]);
        assert_eq!(&right, &[2, 4, 6]);
    }

    #[test]
    fn test_matrix_rows() {
        let matrix = Matrix::new(vec![
            vec![0, 1, 2, 3],   //
            vec![4, 5, 6, 7],   //
            vec![8, 9, 10, 11], //
        ]);
        for (row_iter, row_vec) in matrix.row_iter().zip([
            [0, 1, 2, 3],   //
            [4, 5, 6, 7],   //
            [8, 9, 10, 11], //
        ]) {
            for (el1, el2) in row_iter.zip(row_vec.iter()) {
                assert_eq!(el1, el2)
            }
        }
    }
    #[test]
    fn test_matrix_cols() {
        let matrix = Matrix::new(vec![
            vec![0, 1, 2, 3],   //
            vec![4, 5, 6, 7],   //
            vec![8, 9, 10, 11], //
        ]);
        for (col_iter, col_vec) in
            matrix
                .col_iter()
                .zip([[0, 4, 8], [1, 5, 9], [2, 6, 10], [3, 7, 11]])
        {
            for (el1, el2) in col_iter.zip(col_vec.iter()) {
                assert_eq!(el1, el2)
            }
        }
    }

    #[test]
    fn test_matrix_diagonal() {
        let matrix = Matrix::new(vec![
            vec![0, 1, 2, 3],   //
            vec![4, 5, 6, 7],   //
            vec![8, 9, 10, 11], //
        ]);

        for (diag_iter, diag_vec) in matrix.diagonal_iter().zip([
            vec![8],
            vec![4, 9],
            vec![0, 5, 10],
            vec![1, 6, 11],
            vec![2, 7],
            vec![3],
        ]) {
            for (el1, el2) in diag_iter.zip(diag_vec.iter()) {
                assert_eq!(el1, el2)
            }
        }
    }

    #[test]
    fn test_matrix_antidiagonal() {
        let matrix = Matrix::new(vec![
            vec![0, 1, 2, 3],   //
            vec![4, 5, 6, 7],   //
            vec![8, 9, 10, 11], //
        ]);

        for (antidiag_iter, antidiag_vec) in matrix.antidiagonal_iter().zip([
            vec![0],
            vec![4, 1],
            vec![8, 5, 2],
            vec![9, 6, 3],
            vec![10, 7],
            vec![11],
        ]) {
            for (el1, el2) in antidiag_iter.zip(antidiag_vec.iter()) {
                assert_eq!(el1, el2)
            }
        }
    }
    #[test]
    fn test_matrix_get() {
        let matrix = Matrix::new(vec![
            vec![0, 1, 2, 3],   //
            vec![4, 5, 6, 7],   //
            vec![8, 9, 10, 11], //
        ]);
        assert_eq!(matrix.get_element([0, 0]), Some(&0));
        assert_eq!(matrix.get_element([2, 1]), Some(&9));
        assert_eq!(matrix.get_element([0, 4]), None);
        assert_eq!(matrix.get_element([3, 0]), None);
        assert_eq!(matrix.get_element([3, 4]), None);
    }
}
