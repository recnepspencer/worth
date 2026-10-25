//! Isolate the real ForkShared interval edits from changed values and their quote.
use super::charge_updates::interval_batch_growth_bound;
use super::fork_overlay::{record_base_readmission, record_base_retirement};
use super::{PersistentOrdMap, PersistentOrdMapStorage};
use crate::tests::performance_support::peak_allocation::measure;

#[test]
fn diagnostics_overlay_bound_covers_shared_retirement_and_readmission_at_scale() {
    const INTERVALS: u64 = 8_192;
    let mut base = PersistentOrdMap::<u64, u64>::new();
    for key in 0..INTERVALS * 8 {
        base.insert(key, key);
    }
    let mut overlay = base.fork_persistent();
    // Two singleton intervals around each live key. Retirement must merge
    // them (remove/remove/insert); readmission must split them back again.
    for index in 0..INTERVALS {
        overlay.remove(&(index * 8 + 1));
        overlay.remove(&(index * 8 + 3));
    }
    let snapshot = overlay.fork_persistent();
    let PersistentOrdMapStorage::ForkShared {
        base,
        retired_base_intervals: intervals,
        ..
    } = overlay.storage
    else {
        panic!("fixture must exercise a fork overlay")
    };
    for edits in [1_usize, 32, 300, 8_192] {
        let mut current = intervals.clone();
        for retire in [true, false] {
            let original = current.clone();
            let forecast = interval_batch_growth_bound::<u64>(current.len(), edits)
                .unwrap()
                .bytes();
            let (updated, peak) = measure(|| {
                for index in 0..edits as u64 {
                    // Spread small batches across the shared three-level tree.
                    let key = (index * INTERVALS / edits as u64) * 8 + 2;
                    let previous = current.clone();
                    if retire {
                        record_base_retirement(&base, &mut current, &key);
                    } else {
                        record_base_readmission(&base, &mut current, &key);
                    }
                    std::hint::black_box(previous);
                }
                current
            });
            let peak = peak.unwrap() as u64;
            // No changed-value or Arc-cell allowance: even the endpoint cells
            // created by the real operations must fit this interval-only quote.
            assert!(
                peak > 0 && peak <= forecast,
                "edits={edits}, retire={retire}: {peak} > {forecast}"
            );
            if retire {
                assert_eq!(updated.len(), original.len() - edits);
            } else {
                let difference = updated
                    .iter()
                    .zip(intervals.iter())
                    .find(|(actual, expected)| actual != expected);
                assert!(updated == intervals,
                    "edits={edits}: interval restoration differs at {difference:?}; lengths {} / {}",
                    updated.len(), intervals.len());
            }
            current = updated;
        }
    }
    assert_eq!(snapshot.len(), (INTERVALS * 6) as usize);
}
