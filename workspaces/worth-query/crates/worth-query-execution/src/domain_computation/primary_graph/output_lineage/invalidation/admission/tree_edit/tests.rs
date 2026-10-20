//! What one tree value's edit sequence admits on the request meter.
use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

use super::super::super::InvalidationEditAdmission;
use super::super::{index_capacity, IndexAdmission};
use super::PreparedTreeEdits;

fn meter(maximum_preparation_bytes: Option<u64>) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: None,
        maximum_preparation_bytes,
    })
}

fn charges(meter: &InvalidationEditAdmission) -> (u64, u64, u64) {
    (
        meter.charged_work(),
        meter.charged_bytes(),
        meter.charged_navigation(),
    )
}

/// Insertions into a tree created empty are the same work as a fresh build:
/// the sequence admits exactly the build's growth, visit for visit.
#[test]
fn insertions_into_an_empty_tree_admit_what_a_fresh_build_admits() {
    for entries in [1, 31, 32, 64, 2048, 5000] {
        let (mut sequence, mut fresh) = (meter(None), meter(None));
        let mut edits = PreparedTreeEdits::<usize, ()>::new();
        for length in 0..entries {
            edits.insert(length, &mut sequence).unwrap();
            fresh.fresh_insert::<usize, ()>(length).unwrap();
        }
        assert_eq!(charges(&sequence), charges(&fresh), "{entries}");
    }
}

/// Edits of a tree shared with a predecessor admit at most what the final
/// tree's insertions can allocate together, not one copied path per edit.
/// A meter one byte short refuses with the meter's cumulative amounts.
#[test]
fn a_shared_tree_admits_its_final_size_not_its_edit_count() {
    let (prior, added, replaced) = (2047usize, 4000usize, 4000usize);
    let run = |maximum| {
        let mut meter = meter(maximum);
        let mut edits = PreparedTreeEdits::<usize, ()>::new();
        let ran = (0..added)
            .map(|step| prior + step)
            .chain(std::iter::repeat_n(prior + added, replaced))
            .try_for_each(|length| edits.insert(length, &mut meter));
        (ran, meter.charged_bytes())
    };
    let (ran, admitted) = run(None);
    assert_eq!(ran, Ok(()));
    // A replacement passes its unchanged length, so it may still add one.
    let reach = prior + added + 1;
    let whole = index_capacity::insertion_sequence_bytes::<usize, ()>(reach).unwrap();
    let paths =
        index_capacity::ordered_edit_bytes::<usize, ()>(prior).unwrap() * (added + replaced) as u64;
    assert!(admitted <= whole && whole < paths);
    assert_eq!(
        run(Some(admitted - 1)).0,
        Err(CompanionPreflightStop::PreparationMemoryExhausted {
            required: admitted,
            maximum: admitted - 1,
        })
    );
}

/// A removal can merge into a fresh node, outside the insertion bound: it
/// admits its own path, and every later edit then admits its own path too.
#[test]
fn after_a_removal_every_edit_admits_its_own_copied_path() {
    let (mut sequence, mut copied) = (meter(None), meter(None));
    let mut edits = PreparedTreeEdits::<usize, ()>::new();
    for length in 100..140 {
        edits.insert(length, &mut sequence).unwrap();
    }
    let before = charges(&sequence);
    edits.remove(140, &mut sequence).unwrap();
    copied.ordered_remove::<usize, ()>(140).unwrap();
    for length in [139, 140, 141] {
        edits.insert(length, &mut sequence).unwrap();
        copied.ordered_edit::<usize, ()>(length).unwrap();
    }
    let after = charges(&sequence);
    assert_eq!(
        (after.0 - before.0, after.1 - before.1, after.2 - before.2),
        charges(&copied)
    );
}
