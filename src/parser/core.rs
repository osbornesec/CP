use memchr::memmem;

const SECTION_DELIMITER: &[u8] = b"==============================================";

/// Find section byte ranges without copying data
#[must_use]
#[inline]
pub fn find_section_ranges(data: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let finder = memmem::Finder::new(SECTION_DELIMITER);
    let mut start_pos = 0;

    for delimiter_pos in finder.find_iter(data) {
        if delimiter_pos > start_pos {
            ranges.push((start_pos, delimiter_pos));
        }
        start_pos = delimiter_pos + SECTION_DELIMITER.len();
    }

    if start_pos < data.len() {
        ranges.push((start_pos, data.len()));
    }

    return ranges;
}

// Implementation methods moved to mod.rs to consolidate impl blocks
