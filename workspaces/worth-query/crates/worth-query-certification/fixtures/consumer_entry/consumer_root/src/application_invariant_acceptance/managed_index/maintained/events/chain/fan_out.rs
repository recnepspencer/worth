//! One early row inherited by every scope of an ordered chain: changing it
//! invokes each scope's rule, and each loan borrows the membership discovery
//! recorded, so the commit's work grows with the chain, never with its square.
use super::*;

/// The charged SDK work of renaming A's root row once `count` scopes inherit
/// it, with each invocation's loan taken or skipped.
pub(in super::super::super) fn rename_inherited(count: usize, lend: bool) -> u64 {
    let (world, _, records, mut key) = setup_growth();
    records.skip_loan.store(!lend, Ordering::SeqCst);
    let mut previous = "A".to_owned();
    for i in 0..count {
        let event = format!("fan-{i:04}");
        edit_history(&world, &mut key, append(&event, &previous, None, None));
        previous = event;
    }
    records.observed.lock().unwrap().clear();
    let rename = crate::application_invariant_acceptance::managed_index::maintained::nested::edit::Edit::Name {
        item: "x".into(),
        value: "match-renamed".into(),
    };
    let ((), work) =
        super::super::super::court::prefix_growth::invoice(|| publish(&world, &mut key, rename));
    if !lend {
        return work;
    }
    let observed = records.observed.lock().unwrap();
    let lent = observed
        .iter()
        .filter(|(index, _, [committed, proposed])| {
            *index == "ordered" && committed.len() == 1 && proposed.len() == 1
        })
        .count();
    assert!(lent > count, "A and every inheriting scope lend x: {lent}");
    work
}
