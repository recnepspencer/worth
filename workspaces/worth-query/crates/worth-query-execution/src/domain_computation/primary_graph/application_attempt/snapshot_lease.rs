use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::WorthQueryPrimaryGraphIntegrationHandle;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisReleaseOutcome;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryApplicationSnapshotLeaseDenial {
    ForeignRuntime,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryApplicationSnapshotRelease {
    NativeSnapshotReleased,
    SharedBasisRelease(WorthQueryApplicationBasisReleaseOutcome),
}

impl WorthQueryApplicationSnapshotRelease {
    pub(in crate::domain_computation) const fn snapshot_released(self) -> bool {
        match self {
            Self::NativeSnapshotReleased => true,
            Self::SharedBasisRelease(outcome) => outcome.snapshot_released(),
        }
    }

    pub(in crate::domain_computation) const fn custody_released(self) -> bool {
        match self {
            Self::NativeSnapshotReleased => true,
            Self::SharedBasisRelease(outcome) => outcome.custody_released(),
        }
    }
}

enum SnapshotCustody {
    Owned {
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        snapshot: SnapshotHandle,
    },
    Shared(WorthQueryApplicationBasisLease),
    Original(std::sync::Arc<SharedOriginalSnapshot>),
}

struct SharedOriginalSnapshot {
    custody: Option<SnapshotCustody>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Drop for SharedOriginalSnapshot {
    fn drop(&mut self) {
        if let Some(custody) = self.custody.take() {
            release_custody(custody);
        }
    }
}
impl SnapshotCustody {
    fn snapshot(&self) -> &SnapshotHandle {
        match self {
            Self::Owned { snapshot, .. } => snapshot,
            Self::Shared(basis) => basis.snapshot_handle(),
            Self::Original(shared) => shared
                .custody
                .as_ref()
                .expect("live original pin")
                .snapshot(),
        }
    }
    fn handle(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        match self {
            Self::Owned { handle, .. } => handle,
            Self::Shared(basis) => basis.graph_handle(),
            Self::Original(shared) => shared.custody.as_ref().expect("live original pin").handle(),
        }
    }
}

pub(in crate::domain_computation) struct WorthQueryApplicationSnapshotLease {
    custody: Option<SnapshotCustody>,
    product: crate::basis::WorthQueryProductBranchLease,
    pub(super) layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
}

impl WorthQueryApplicationSnapshotLease {
    pub(super) fn share_original(
        mut self,
        control: super::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<(Self, Self), super::retained_decision_facts::StoreDenial> {
        let memory = control
            .policy()
            .reserve_memory((size_of::<SharedOriginalSnapshot>() + 2 * size_of::<usize>()) as u64);
        control.check_live()?;
        let memory = memory?;
        let custody = self.custody.take().expect("live snapshot custody");
        let shared = match custody {
            SnapshotCustody::Original(shared) => shared,
            custody => std::sync::Arc::new(SharedOriginalSnapshot {
                custody: Some(custody),
                _memory: memory,
            }),
        };
        self.custody = Some(SnapshotCustody::Original(shared.clone()));
        let retained = Self {
            custody: Some(SnapshotCustody::Original(shared)),
            product: self.product.retained_clone(),
            layout: self.layout.clone(),
        };
        Ok((self, retained))
    }
    pub(in crate::domain_computation) fn acquire(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthQueryApplicationSnapshotLeaseDenial> {
        let basis = product.relational_basis().clone();
        let snapshot = handle.with_runtime_mut(execution, |runtime| {
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
            custody: Some(SnapshotCustody::Owned { handle, snapshot }),
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
            custody: Some(SnapshotCustody::Owned { handle, snapshot }),
            product,
            layout,
        }
    }

    /// Shares the wave's final Native snapshot owner without registering or
    /// releasing another Native snapshot. The session owns this local share.
    pub(in crate::domain_computation) fn from_shared_selected<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<Self, CompanionPreflightStop> {
        let selected = shared.selected();
        let graph = selected
            .application()
            .runtime
            .primary_graph()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let basis = selected.application_basis().retain_shared()?;
        // Publication binding, two World observation Arc pairs, optional
        // guard, Bridge Arc and layout Arc are all shallow fixed copies.

        Ok(Self {
            custody: Some(SnapshotCustody::Shared(basis)),
            product: selected.product().retained_clone(),
            layout: graph.retain_layout(),
        })
    }

    pub(in crate::domain_computation) fn snapshot(&self) -> &SnapshotHandle {
        self.custody
            .as_ref()
            .expect("snapshot lease retains custody")
            .snapshot()
    }

    pub(in crate::domain_computation) fn product(
        &self,
    ) -> &crate::basis::WorthQueryProductBranchLease {
        &self.product
    }

    pub(in crate::domain_computation) fn handle(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        self.custody
            .as_ref()
            .expect("snapshot lease retains custody")
            .handle()
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn release(self) -> bool {
        self.release_custody().snapshot_released()
    }

    pub(in crate::domain_computation) fn release_custody(
        mut self,
    ) -> WorthQueryApplicationSnapshotRelease {
        release_custody(self.custody.take().expect("snapshot lease releases once"))
    }
}

impl Drop for WorthQueryApplicationSnapshotLease {
    fn drop(&mut self) {
        if let Some(custody) = self.custody.take() {
            release_custody(custody);
        }
    }
}

fn release_custody(custody: SnapshotCustody) -> WorthQueryApplicationSnapshotRelease {
    match custody {
        SnapshotCustody::Owned { handle, snapshot } => {
            handle.release_query_snapshot(&snapshot);
            WorthQueryApplicationSnapshotRelease::NativeSnapshotReleased
        }
        SnapshotCustody::Shared(basis) => {
            WorthQueryApplicationSnapshotRelease::SharedBasisRelease(basis.release().outcome())
        }
        SnapshotCustody::Original(shared) => match std::sync::Arc::try_unwrap(shared) {
            Ok(mut owner) => release_custody(owner.custody.take().expect("live original pin")),
            Err(shared) => {
                drop(shared);
                WorthQueryApplicationSnapshotRelease::SharedBasisRelease(
                    WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained,
                )
            }
        },
    }
}
