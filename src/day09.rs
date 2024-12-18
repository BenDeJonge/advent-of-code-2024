const SUMMATION: [usize; 10] = [
    0,  // 0
    1,  // 1
    3,  // 2
    6,  // 3
    10, // 4
    15, // 5
    21, // 6
    28, // 7
    36, // 8
    45, // 9
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockValue {
    Empty,
    File(usize),
}

#[derive(Debug, PartialEq)]
pub struct Memory {
    files: Vec<Block>,
    gaps: Vec<Block>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    pub start: usize,
    pub stop: usize,
    pub size: usize,
    pub value: BlockValue,
}

impl Block {
    pub fn new(start: usize, stop: usize, value: BlockValue) -> Block {
        Block {
            start,
            stop,
            size: stop - start,
            value,
        }
    }

    pub fn update_stop(&mut self, stop: usize) -> Result<(), &str> {
        if stop <= self.start {
            return Err("stop must be strictly larger than start");
        }
        self.stop = stop;
        self.size = stop - self.start;
        Ok(())
    }

    pub fn update_start(&mut self, start: usize) -> Result<(), &str> {
        if start >= self.stop {
            return Err("start must be strictly smaller than stop");
        }
        self.start = start;
        self.size = self.stop - start;
        Ok(())
    }

    pub fn move_to_start(&mut self, start: usize) {
        self.stop -= self.start - start;
        self.start = start;
    }
}

impl Memory {
    pub fn new(files: Vec<Block>, gaps: Vec<Block>) -> Self {
        Memory { files, gaps }
    }

    /// Imagine a memory block with file index `f` extending from index `i` to `j`,
    /// for a total size s=j-i.
    /// ```text
    /// ..|i....j
    /// ..|f...f|..
    /// ```
    ///
    /// The checksum c of this block would be:
    /// ```text
    /// c = i * f + (i+i) * f ... (j-1) * f = f * (i + i+1 + ... + j)
    ///   = f * (i + i+1 + ... + i+s-1) = f * (s*i + SUM(0, s-1))
    /// ```
    /// where `SUM(0, s-1)` can be computed as by indexing `SUMMATION[s-1]`.
    pub fn checksum(&self) -> usize {
        self.files
            .iter()
            .map(|block| match block.value {
                BlockValue::File(file_idx) => {
                    file_idx * (block.start * block.size + SUMMATION[block.size - 1])
                }
                BlockValue::Empty => unreachable!(),
            })
            .sum()
    }
}

/// Get sizes of the files and gaps.
pub fn parse_input(input: &str) -> Memory {
    let bytes = input.bytes();
    let mut files = Vec::with_capacity(bytes.len() / 2);
    let mut gaps = Vec::with_capacity(bytes.len() / 2);
    let mut start = 0;
    for (i, byte) in bytes.enumerate() {
        // Digit 0 is represented by 0x30.
        let size = (byte - 0x30) as usize;
        // The block is empty.
        if size == 0 {
            continue;
        }
        // Digits alternate between a file and a gap.
        if i % 2 == 0 {
            files.push(Block::new(start, start + size, BlockValue::File(i / 2)));
        } else {
            gaps.push(Block::new(start, start + size, BlockValue::Empty));
        }
        start += size;
    }
    Memory::new(files, gaps)
}

/// Compute the checksum of the filesystem after moving file fragments from the
/// back into open gaps at the front.
pub fn part_1(memory: &mut Memory) -> usize {
    let total_length = memory
        .files
        .last()
        .expect("at least 1 file")
        .stop
        .max(memory.gaps.last().expect("at least 1 gap").stop);
    let n_gaps = memory.gaps.len() - 1;
    let mut files = Vec::new();
    let mut i_file = memory.files.len() - 1;
    let mut i_gap = 0;
    let mut file = &mut memory.files[i_file];
    let mut gap = &mut memory.gaps[i_gap];
    while file.start >= gap.stop {
        let block = Block::new(gap.start, gap.start + file.size.min(gap.size), file.value);
        // The file is (more than) exactly emptied into the gap.
        if file.update_stop(file.stop - block.size).is_err() {
            // All files have been processed.
            if i_file == 0 {
                break;
            }
            i_file -= 1;
            file = &mut memory.files[i_file];
        }
        // The gap is (more than) exactly filled by the file.
        if gap.update_start(gap.start + block.size).is_err() {
            // All gaps have been processed.
            if i_gap == n_gaps {
                break;
            }
            i_gap += 1;
            gap = &mut memory.gaps[i_gap];
        }
        files.push(block);
    }
    files.extend(memory.files.drain(0..=i_file));
    files.sort_by(|a, b| a.start.cmp(&b.start));
    // The last file could have some of its final elements moved to a gap
    // connected to its first element. In that case, group them.
    let n_files = files.len();
    let mut last_file_stop = 0;
    if n_files > 1 {
        let last = files.pop().expect("at least 2 files");
        last_file_stop = last.stop;
        let mut prev = files.pop().expect("at least 2 files");
        if last.value == prev.value {
            prev.update_stop(last.stop)
                .expect("stop is larger than start");
            files.push(prev);
        } else {
            files.push(prev);
            files.push(last);
        }
    }
    memory.files = files;
    memory.gaps = vec![Block::new(last_file_stop, total_length, BlockValue::Empty)];

    memory.checksum()
}

/// Compute the checksum of the filesystem after moving file fragments from the
/// back into the first open gap at the front that can completely house them.
pub fn part_2(memory: &mut Memory) -> usize {
    let mut n_gaps = memory.gaps.len();
    for file in memory.files.iter_mut().rev() {
        // All gaps have already been filled.
        if n_gaps == 0 {
            break;
        }
        for i_gap in 0..n_gaps {
            let gap = &mut memory.gaps[i_gap];
            // Files can only move to the left.
            if gap.start > file.start {
                break;
            }
            if gap.size >= file.size {
                let mut new_gap = *file;
                new_gap.value = BlockValue::Empty;
                file.move_to_start(gap.start);
                if gap.update_start(file.stop).is_err() {
                    memory.gaps.remove(i_gap);
                }
                // The mut_range_bound clippy warning does not apply because we
                // are modifying the range bound for the next iteration, due to
                // the immediate break.
                // https://rust-lang.github.io/rust-clippy/master/index.html#mut_range_bound
                n_gaps -= 1;
                break;

                // In principle, we should account for the fact that moving a
                // file to another location leaves behind a gap. However, as
                // files are moved right to left only once, no other file can
                // ever fill this newly created gap.
                // If needed, the files before and after could be found using
                // binary search by matching to the new_gap endpoints.
            }
        }
    }
    memory.files.sort_by(|a, b| a.start.cmp(&b.start));
    // dbg!(&memory);
    memory.checksum()
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::{parse_input, part_1, part_2, Block, Memory};
    use crate::{day09::BlockValue, util::read_file_to_string};

    // 0    5    10   15   20   25   30   35   40
    // 00...111...2...333.44.5555.6666.777.888899
    // 0099811188827773336446555566..............
    const INPUT: &str = "2333133121414131402";

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(INPUT),
            Memory::new(
                vec![
                    Block::new(0, 2, BlockValue::File(0)),
                    Block::new(5, 8, BlockValue::File(1)),
                    Block::new(11, 12, BlockValue::File(2)),
                    Block::new(15, 18, BlockValue::File(3)),
                    Block::new(19, 21, BlockValue::File(4)),
                    Block::new(22, 26, BlockValue::File(5)),
                    Block::new(27, 31, BlockValue::File(6)),
                    Block::new(32, 35, BlockValue::File(7)),
                    Block::new(36, 40, BlockValue::File(8)),
                    Block::new(40, 42, BlockValue::File(9)),
                ],
                vec![
                    Block::new(2, 5, BlockValue::Empty),
                    Block::new(8, 11, BlockValue::Empty),
                    Block::new(12, 15, BlockValue::Empty),
                    Block::new(18, 19, BlockValue::Empty),
                    Block::new(21, 22, BlockValue::Empty),
                    Block::new(26, 27, BlockValue::Empty),
                    Block::new(31, 32, BlockValue::Empty),
                    Block::new(35, 36, BlockValue::Empty),
                ],
            )
        )
    }

    #[test]
    fn test_part_1_small() {
        // 0    5    10   15   20   25   30   35   40
        // 00...111...2...333.44.5555.6666.777.888899
        // 0099811188827773336446555566..............
        assert_eq!(part_1(&mut parse_input(INPUT)), 1928)
    }

    #[test]
    fn test_part_1_full() {
        assert_eq!(
            part_1(&mut parse_input(&read_file_to_string("data/day09.txt"))),
            6242766523059
        )
    }

    #[test]
    fn test_part_2_small() {
        // 0    5    10   15   20   25   30   35   40
        // 00...111...2...333.44.5555.6666.777.888899
        // 00992111777.44.333....5555.6666.....8888..
        assert_eq!(part_2(&mut parse_input(INPUT)), 2858)
    }

    #[test]
    fn test_part_2_full() {
        assert_eq!(
            part_2(&mut parse_input(&read_file_to_string("data/day09.txt"))),
            6272188244509
        )
    }
}
