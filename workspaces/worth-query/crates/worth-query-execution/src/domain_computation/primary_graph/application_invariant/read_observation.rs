use std::cell::Cell;

use super::super::invariant_projection::WorthQueryInvariantProjectionWork;

/// One invocation's observations; views borrow it rather than resetting it.
#[derive(Default)]
pub(super) struct InvariantReadObservation(Cell<WorthQueryInvariantProjectionWork>);

impl InvariantReadObservation {
    pub(super) fn from_preparation(reads: WorthQueryInvariantProjectionWork) -> Self {
        Self(Cell::new(reads))
    }

    pub(super) fn reads(&self) -> WorthQueryInvariantProjectionWork {
        self.0.get()
    }

    pub(super) fn walk(&self, reads: worth_relational::facade::runtime::StructuralWalkReads) {
        self.record(|work| {
            work.record_adjacencies(reads.adjacency_lists, reads.relation_records);
            work.record_endpoints(reads.entity_records);
        });
    }

    pub(super) fn adjacency(&self) {
        self.record(|reads| reads.record_adjacencies(1, 0));
    }

    pub(super) fn edge(&self) {
        self.record(|reads| reads.record_edge());
    }

    pub(super) fn endpoint(&self) {
        self.record(|reads| reads.record_endpoint());
    }

    pub(super) fn field(&self) {
        self.record(|reads| reads.record_fields(1));
    }

    fn record(&self, record: impl FnOnce(&mut WorthQueryInvariantProjectionWork)) {
        let mut reads = self.0.get();
        record(&mut reads);
        self.0.set(reads);
    }
}
