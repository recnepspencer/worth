//! An inverse lexical enclosure asks whether any prefix of a row lies inside it.
use std::cmp::Ordering;

pub fn prefix_watch_bounds_may_match<E>(
    lower: &str,
    upper: &str,
    row: &str,
    mut checkpoint: impl FnMut(u64) -> Result<(), E>,
) -> Result<bool, E> {
    for (index, byte) in lower.bytes().enumerate() {
        checkpoint(1)?;
        let Some(row_byte) = row.as_bytes().get(index) else {
            return Ok(false);
        };
        match row_byte.cmp(&byte) {
            Ordering::Less => return Ok(false),
            Ordering::Greater => {
                // This is the smallest valid UTF-8 prefix at or above lower.
                // Byte divergence may occur inside a scalar, so retain its end.
                let mut end = index + 1;
                while !row.is_char_boundary(end) {
                    checkpoint(1)?;
                    end += 1;
                }
                return Ok(
                    crate::compare_prefix_keys(&row[..end], 0, upper, 0, checkpoint)?
                        != Ordering::Greater,
                );
            }
            Ordering::Equal => {}
        }
    }
    // The actual lower probe itself is a prefix, including the empty probe.
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_interval_matches_independent_utf8_prefix_enumeration() {
        let words = [
            "",
            "\0",
            "a",
            "aa",
            "ab",
            "b",
            "e\u{301}",
            "é",
            "éa",
            "ê",
            "😀",
            "😀a",
            "😁",
            "\u{10ffff}",
        ];
        for row in words {
            let prefixes: Vec<_> = (0..=row.len())
                .filter(|i| row.is_char_boundary(*i))
                .map(|i| &row[..i])
                .collect();
            for lower in words {
                for upper in words.into_iter().filter(|upper| lower <= *upper) {
                    let expected = prefixes.iter().any(|p| lower <= *p && *p <= upper);
                    let mut units = 0;
                    let actual = prefix_watch_bounds_may_match(lower, upper, row, |work| {
                        units += work;
                        Ok::<_, ()>(())
                    })
                    .unwrap();
                    assert_eq!(
                        actual, expected,
                        "row={row:?} interval={lower:?}..={upper:?}"
                    );
                    assert!(units <= (lower.len() + upper.len() + 4) as u64);
                }
            }
        }
    }

    #[test]
    fn inverse_interval_keeps_original_stop_before_a_long_comparison() {
        let lower = "a".repeat(4096);
        let row = format!("{lower}b");
        let mut units = 0;
        assert_eq!(
            prefix_watch_bounds_may_match(&lower, "z", &row, |_| {
                units += 1;
                if units == 7 {
                    Err(23)
                } else {
                    Ok(())
                }
            }),
            Err(23)
        );
        assert_eq!(units, 7);
    }
}
