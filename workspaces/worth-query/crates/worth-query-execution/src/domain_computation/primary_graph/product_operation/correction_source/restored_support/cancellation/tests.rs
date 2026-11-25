//! Public #388 follow-up: an empty selection reads no World original history.
use super::{originals, Occurrences};

#[test]
fn an_empty_selection_reads_no_original_history() {
    let mut reads = 0;
    let mut observe = || {
        reads += 1;
        Ok::<_, ()>(())
    };
    assert_eq!(originals(true, &Occurrences::default(), &mut observe), Ok(None));
    assert_eq!(originals(false, &Occurrences::default(), &mut observe), Ok(None));
    assert_eq!(reads, 0);
}
