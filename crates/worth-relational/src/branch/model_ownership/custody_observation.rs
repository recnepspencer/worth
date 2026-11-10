//! Passive count of candidate custody reads on one thread, with unwind cleanup.
use crate::identity::data::EntityId;
use std::cell::RefCell;
use std::collections::BTreeSet;

/// How often a candidate ownership validation read custody, and of how many
/// distinct records. A cost court compares the two; nothing is decided here.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationalCandidateCustodyReads {
    pub reads: u64,
    pub distinct_records: u64,
}

thread_local! {
    static READS: RefCell<Option<(u64, BTreeSet<EntityId>)>> = const { RefCell::new(None) };
}
struct Recording;
impl Drop for Recording {
    fn drop(&mut self) {
        READS.with(|reads| *reads.borrow_mut() = None);
    }
}

/// Count the custody reads candidate ownership validation makes on this thread
/// while `operation` runs. Observation changes no request's controls or bill.
pub fn observe_candidate_custody_reads<T>(
    operation: impl FnOnce() -> T,
) -> (T, RelationalCandidateCustodyReads) {
    READS.with(|reads| {
        let mut reads = reads.borrow_mut();
        assert!(reads.is_none(), "nested candidate custody recorder");
        *reads = Some((0, BTreeSet::new()));
    });
    let recording = Recording;
    let value = operation();
    let (reads, records) = READS
        .with(|reads| reads.borrow_mut().take())
        .expect("active recorder");
    drop(recording);
    (
        value,
        RelationalCandidateCustodyReads {
            reads,
            distinct_records: records.len() as u64,
        },
    )
}

pub(super) fn record(entity: EntityId) {
    READS.with(|reads| {
        if let Some((count, records)) = reads.borrow_mut().as_mut() {
            *count = count.saturating_add(1);
            records.insert(entity);
        }
    });
}
