use crate::identity::data::RelationId;
use crate::runtime::RelationalRuntime;
use crate::storage::partition::{AdjacencyKindBasis, AdjacencySet};

/// What a bounded adjacency traversal actually touched.
///
/// A bounded read's honesty claim is that it never touches more of the source
/// than its budget allows. That claim needs a witness the traversal cannot
/// forge, so the ledger counts leases and copies separately: a lease is a
/// borrow of the substrate's own storage and costs nothing, while a copy is
/// Theta(degree) and would defeat the bound.
///
/// Charges accumulate locally and settle once, because the instrumentation sink
/// is behind a mutex and a per-element charge would be its own hot-path defect.
#[derive(Debug, Default)]
pub(crate) struct AdjacencyLeaseLedger {
    kind_slices_leased: usize,
}

impl AdjacencyLeaseLedger {
    /// Borrow one kind's adjacency ids out of the pinned edition.
    ///
    /// The returned slice is the substrate's own storage. Nothing is copied,
    /// so the caller may be handed a fanout far larger than its budget and
    /// still stop after the units it is allowed to spend.
    pub(crate) fn lease<'edition>(
        &mut self,
        adjacency: Option<&'edition AdjacencySet>,
        basis: AdjacencyKindBasis,
        kind_id: crate::identity::data::KindId,
    ) -> &'edition [RelationId] {
        let Some(adjacency) = adjacency else {
            return &[];
        };
        self.kind_slices_leased += 1;
        adjacency.kind_slice(basis, kind_id)
    }

    /// Settle the traversal's leases in one instrumentation visit.
    pub(crate) fn settle(self, runtime: &RelationalRuntime) {
        if self.kind_slices_leased == 0 {
            return;
        }
        runtime
            .services
            .instrumentation
            .count(|counters| counters.adjacency_kind_slices_leased += self.kind_slices_leased);
    }
}
