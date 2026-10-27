//! Full source custody is established before a read capability can be constructed.
use super::VisibilitySnapshotBasis;
use crate::branch::RelationalBranchRoot;
use crate::identity::data::{PartitionId, VersionId};
use crate::storage::residency::RelationalRegionHeadView;
use std::sync::Arc;

/// The immutable root owns counted Full leases for all of its regions. Holding
/// this selected basis keeps those exact leases alive, even if a later fresh
/// transport discovers damage. This is accepted truth, not a new demand.
#[derive(Clone, Debug)]
pub(crate) struct ResidentVisibilitySnapshotBasis(VisibilitySnapshotBasis);
impl ResidentVisibilitySnapshotBasis {
    pub(crate) fn admit(
        basis: VisibilitySnapshotBasis,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(request)?;
        basis.root().demand_all_full(request, symbols)?;
        Ok(Self(basis))
    }
    pub(crate) fn basis(&self) -> &VisibilitySnapshotBasis {
        &self.0
    }
    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        self.0.root()
    }
    pub(crate) fn version_id(&self) -> VersionId {
        self.0.version_id()
    }
    pub(crate) fn branch_id(&self) -> &crate::history::data::BranchId {
        self.0.branch_id()
    }
    pub(crate) fn head_region(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.root()
            .partition_state(id)
            .map(RelationalRegionHeadView::from_owned_partition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_basis_cannot_infallibly_become_resident_custody() {
        trait WithoutPromotion<A> {
            fn witness() {}
        }
        impl<T> WithoutPromotion<()> for T {}
        impl<T: Into<ResidentVisibilitySnapshotBasis>> WithoutPromotion<u8> for T {}
        let _ = <VisibilitySnapshotBasis as WithoutPromotion<_>>::witness;
        trait WithoutDefault<A> {
            fn witness() {}
        }
        impl<T> WithoutDefault<()> for T {}
        impl<T: Default> WithoutDefault<u8> for T {}
        let _ = <ResidentVisibilitySnapshotBasis as WithoutDefault<_>>::witness;
    }
}
