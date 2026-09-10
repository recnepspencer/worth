use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::WorthQueryPrimaryGraphIntegrationHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryApplicationSnapshotLeaseDenial {
    ForeignRuntime,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
}

pub(in crate::domain_computation) struct WorthQueryApplicationSnapshotLease {
    handle: WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: Option<SnapshotHandle>,
    product: crate::basis::WorthQueryProductBranchLease,
    pub(super) layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
}

impl WorthQueryApplicationSnapshotLease {
    pub(in crate::domain_computation) fn acquire(
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthQueryApplicationSnapshotLeaseDenial> {
        let basis = product.relational_basis().clone();
        let snapshot = handle.with_runtime_mut(|runtime| {
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .map_err(|denial| match denial {
                    worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ForeignRuntime { .. } => {
                        WorthQueryApplicationSnapshotLeaseDenial::ForeignRuntime
                    }
                    worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    } => WorthQueryApplicationSnapshotLeaseDenial::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    },
                    worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
                        WorthQueryApplicationSnapshotLeaseDenial::SnapshotIdentityExhausted
                    }
                })?;
            Ok(snapshot)
        })?;
        Ok(Self {
            handle,
            snapshot: Some(snapshot),
            product,
            layout,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn from_existing(
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
        basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: SnapshotHandle,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Self {
        assert_eq!(
            basis.observation().version_id(),
            snapshot.version_id(),
            "existing application snapshot must select its carried owner basis"
        );
        assert_eq!(
            basis.identity().branch_id(),
            snapshot.branch_id(),
            "existing application snapshot and carried basis must share a branch"
        );
        assert_eq!(
            basis.descriptor(),
            product.relational_basis().descriptor(),
            "existing application snapshot must carry the admitted composite occurrence"
        );
        Self {
            handle,
            snapshot: Some(snapshot),
            product,
            layout,
        }
    }

    pub(in crate::domain_computation) fn snapshot(&self) -> &SnapshotHandle {
        self.snapshot
            .as_ref()
            .expect("application snapshot lease remains live until consumed")
    }

    pub(in crate::domain_computation) fn product(
        &self,
    ) -> &crate::basis::WorthQueryProductBranchLease {
        &self.product
    }

    pub(in crate::domain_computation) fn handle(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.handle
    }

    pub(in crate::domain_computation) fn release(mut self) -> bool {
        let Some(snapshot) = self.snapshot.take() else {
            return false;
        };
        self.handle.with_runtime_mut(|runtime| {
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        });
        true
    }
}

impl Drop for WorthQueryApplicationSnapshotLease {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.handle.with_runtime_mut(|runtime| {
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            });
        }
    }
}
