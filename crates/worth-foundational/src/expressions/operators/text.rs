//! Resumable String and Bytes scans.
//!
//! String order and equality are ordinal over Unicode scalars, which is
//! bytewise order over UTF-8, so containment and affixes compare bytes. Each
//! scanner advances one 8-byte word per call, so the evaluator charges one
//! work unit per word and can suspend between any two words. Early exits
//! stop at a word boundary of the scan stream, so the cost never depends on
//! where a slice ended.

/// Bytes a scanner consumes per step.
pub(crate) const WORD: usize = 8;

/// Whether `byte` starts a UTF-8 scalar rather than continuing one.
fn starts_scalar(byte: u8) -> bool {
    (byte as i8) >= -0x40
}

/// Counts the Unicode scalars of a String.
#[derive(Debug, Clone, Default)]
pub(crate) struct ScalarCount {
    offset: usize,
    scalars: i128,
}

impl ScalarCount {
    /// Scans the next word; the count once `text` is exhausted.
    pub(crate) fn step(&mut self, text: &[u8]) -> Option<i128> {
        let end = (self.offset + WORD).min(text.len());
        let chunk = &text[self.offset..end];
        self.scalars += chunk.iter().filter(|byte| starts_scalar(**byte)).count() as i128;
        self.offset = end;
        (end == text.len()).then_some(self.scalars)
    }
}

/// Finds the byte range of scalars `[low, high)`.
#[derive(Debug, Clone)]
pub(crate) struct ScalarRange {
    low: i128,
    high: i128,
    offset: usize,
    scalars: i128,
    start: Option<usize>,
}

impl ScalarRange {
    /// `None` for bounds that can never be valid.
    pub(crate) fn new(low: i128, high: i128) -> Option<Self> {
        (0 <= low && low <= high).then_some(Self {
            low,
            high,
            offset: 0,
            scalars: 0,
            start: None,
        })
    }

    /// Scans the next word. `Some(None)` reports bounds past the end.
    pub(crate) fn step(&mut self, text: &[u8]) -> Option<Option<(usize, usize)>> {
        let end = (self.offset + WORD).min(text.len());
        for index in self.offset..=end {
            // The word's end is its successor's first byte, except at the
            // end of the text, which always bounds a scalar.
            let boundary = index == text.len() || (index < end && starts_scalar(text[index]));
            if !boundary {
                continue;
            }
            if self.scalars == self.low {
                self.start = Some(index);
            }
            if self.scalars == self.high {
                return Some(self.start.map(|start| (start, index)));
            }
            self.scalars += 1;
        }
        self.offset = end;
        (end == text.len()).then_some(None)
    }
}

/// Knuth-Morris-Pratt containment: the needle's failure table, then the
/// haystack, one word of the combined stream per step.
#[derive(Debug, Clone)]
pub(crate) struct Matcher {
    table: Vec<usize>,
    /// Needle bytes processed into the table.
    built: usize,
    /// Haystack bytes scanned.
    offset: usize,
    matched: usize,
}

impl Matcher {
    /// Scratch the failure table needs, in bytes.
    pub(crate) fn table_bytes(needle: &[u8]) -> u64 {
        needle.len() as u64 * 8
    }

    pub(crate) fn new(needle: &[u8]) -> Self {
        Self {
            table: vec![0; needle.len()],
            built: 1.min(needle.len()),
            offset: 0,
            matched: 0,
        }
    }

    /// Advances one word; the answer once it is known.
    pub(crate) fn step(&mut self, haystack: &[u8], needle: &[u8]) -> Option<bool> {
        if needle.is_empty() {
            return Some(true);
        }
        let mut budget = WORD;
        while budget > 0 && self.built < needle.len() {
            let byte = needle[self.built];
            let mut matched = self.table[self.built - 1];
            while matched > 0 && byte != needle[matched] {
                matched = self.table[matched - 1];
            }
            if byte == needle[matched] {
                matched += 1;
            }
            self.table[self.built] = matched;
            self.built += 1;
            budget -= 1;
        }
        while budget > 0 && self.offset < haystack.len() {
            let byte = haystack[self.offset];
            while self.matched > 0 && byte != needle[self.matched] {
                self.matched = self.table[self.matched - 1];
            }
            if byte == needle[self.matched] {
                self.matched += 1;
            }
            self.offset += 1;
            budget -= 1;
            if self.matched == needle.len() {
                return Some(true);
            }
        }
        let exhausted = self.built == needle.len() && self.offset == haystack.len();
        exhausted.then_some(false)
    }
}

#[cfg(test)]
mod tests {
    use super::{Matcher, ScalarCount, ScalarRange};

    fn run<T>(mut step: impl FnMut() -> Option<T>) -> (T, u64) {
        let mut words = 1;
        loop {
            if let Some(result) = step() {
                return (result, words);
            }
            words += 1;
        }
    }

    fn count(text: &str) -> (i128, u64) {
        let mut counter = ScalarCount::default();
        run(|| counter.step(text.as_bytes()))
    }

    fn range(text: &str, low: i128, high: i128) -> Option<&str> {
        let mut range = ScalarRange::new(low, high)?;
        run(|| range.step(text.as_bytes()))
            .0
            .map(|(start, end)| &text[start..end])
    }

    fn contains(haystack: &str, needle: &str) -> (bool, u64) {
        let mut matcher = Matcher::new(needle.as_bytes());
        run(|| matcher.step(haystack.as_bytes(), needle.as_bytes()))
    }

    #[test]
    fn strings_count_unicode_scalars_one_word_at_a_time() {
        assert_eq!(count("héllo"), (5, 1));
        assert_eq!(count(""), (0, 1));
        assert_eq!(count("日本語日本語"), (6, 3));
        assert_eq!(count(&"a".repeat(64)), (64, 8));
    }

    #[test]
    fn strings_slice_by_scalar_offsets() {
        assert_eq!(range("héllo", 1, 3), Some("él"));
        assert_eq!(range("héllo", 0, 5), Some("héllo"));
        assert_eq!(range("héllo", 5, 5), Some(""));
        assert_eq!(range("héllo", 2, 2), Some(""));
        assert_eq!(range("héllo", 4, 6), None);
        assert_eq!(range("héllo", 6, 6), None);
        assert_eq!(range("héllo", 3, 2), None);
        assert_eq!(range("héllo", -1, 2), None);
        assert_eq!(range("日本語", 1, 2), Some("本"));
        assert_eq!(range("", 0, 0), Some(""));
        let long = "abcdefgh".repeat(4);
        assert_eq!(range(&long, 7, 17), Some(&long[7..17]));
        assert_eq!(range(&long, 8, 16), Some(&long[8..16]));
    }

    #[test]
    fn containment_is_exact_and_charged_per_scanned_word() {
        for (haystack, needle) in [
            ("abcabd", "abd"),
            ("aaaaab", "aab"),
            ("abababc", "ababc"),
            ("abc", "abcd"),
            ("abc", ""),
            ("Steel", "steel"),
            ("x", "x"),
            ("", "x"),
        ] {
            assert_eq!(contains(haystack, needle).0, haystack.contains(needle));
        }
        let haystack = "a".repeat(64);
        assert_eq!(contains(&haystack, "b"), (false, 8));
        assert_eq!(contains(&haystack, "a"), (true, 1));
        let late = format!("{}b", "a".repeat(63));
        assert_eq!(contains(&late, "ab"), (true, 9));
    }
}
