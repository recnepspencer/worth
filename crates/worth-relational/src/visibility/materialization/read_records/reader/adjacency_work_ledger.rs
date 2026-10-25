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
    kind_views_leased: usize,
}

impl AdjacencyLeaseLedger {
    pub(crate) fn settle_read_round(
        self,
        source: &(impl crate::storage::overlay::PartitionAccess + ?Sized),
        instrumentation: &crate::runtime::RuntimeInstrumentation,
    ) {
        if !source.has_read_misses() {
            self.settle_instrumentation(instrumentation);
        }
    }
    /// Borrow one kind's adjacency ids out of the pinned edition.
    ///
    /// The returned view is the substrate's own storage. Nothing is copied,
    /// so the caller may be handed a fanout far larger than its budget and
    /// still stop after the units it is allowed to spend.
    pub(crate) fn lease<'edition>(
        &mut self,
        adjacency: Option<&'edition AdjacencySet>,
        basis: AdjacencyKindBasis,
        kind_id: crate::identity::data::KindId,
    ) -> crate::storage::partition::AdjacencyIds<'edition> {
        let Some(adjacency) = adjacency else {
            return Default::default();
        };
        self.kind_views_leased += 1;
        adjacency.kind_ids(basis, kind_id)
    }

    pub(crate) fn settle_instrumentation(
        self,
        instrumentation: &crate::runtime::RuntimeInstrumentation,
    ) {
        if self.kind_views_leased == 0 {
            return;
        }
        instrumentation
            .count(|counters| counters.adjacency_kind_slices_leased += self.kind_views_leased);
    }
}
