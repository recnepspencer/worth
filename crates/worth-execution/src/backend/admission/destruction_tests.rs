use super::{AdmittedBatch, BatchDeclaration};
use std::cell::Cell;
use worth_foundational::PartitionIdentity;

thread_local! {
    static EVENTS: Cell<[u8; 2]> = const { Cell::new([0; 2]) };
}
fn record(event: u8) {
    let _ = EVENTS.try_with(|events| {
        let mut observed = events.get();
        if observed[0] == 0 {
            observed[0] = event;
        } else {
            observed[1] = event;
        }
        events.set(observed);
    });
}

// Observe the real capacity owner's destruction, before its Vec fields release.
// This callback neither takes ownership nor changes the fields' release order.
impl Drop for BatchDeclaration {
    fn drop(&mut self) {
        record(2);
    }
}
struct Value;
impl Drop for Value {
    fn drop(&mut self) {
        record(1);
    }
}

#[test]
fn borrowed_batch_drops_identities_then_values_then_capacities() {
    let batch =
        AdmittedBatch::try_admit(vec![(PartitionIdentity::new(1), Value, 0, 0)], 0).unwrap();
    EVENTS.with(|events| events.set([0; 2]));
    drop(batch);
    let observed = EVENTS.with(|events| events.replace([0; 2]));
    assert_eq!(
        observed,
        [1, 2],
        "value Drop must precede capacity owner Drop"
    );
}
