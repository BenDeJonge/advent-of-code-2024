use nom::{character::complete::one_of, combinator::recognize, multi::many1, IResult, Parser};
use std::fs::{read_to_string, File};
use std::io;
use std::io::BufRead;
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

#[cfg(test)]
mod test {
    use super::parse_decimal;
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
}
