use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::WorthQueryPrimaryGraphIntegrationHandle;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisReleaseOutcome;
use crate::domain_computation::vocabulary::primary_graph::snapshot_lease::WorthQueryApplicationSnapshotLeaseDenial;
use crate::domain_computation::vocabulary::primary_graph::snapshot_lease::WorthQueryApplicationSnapshotRelease;
use worth_relational::facade::mvcc::CompanionPreflightStop;

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

/// How the live selected snapshot entered this operation. This is selection
/// provenance, not a promise that its head remains latest during execution.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::domain_computation::primary_graph) struct SelectedQueryOrigin {
    current_at_selection: bool,
}

impl SelectedQueryOrigin {
    pub(in crate::domain_computation::primary_graph) fn current_at_selection() -> Self {
        Self {
            current_at_selection: true,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn is_current(self) -> bool {
        self.current_at_selection
    }
}

pub struct WorthQueryApplicationSnapshotLease {
    custody: Option<SnapshotCustody>,
    product: crate::basis::WorthQueryProductBranchLease,
    selection_origin: SelectedQueryOrigin,
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
            selection_origin: self.selection_origin,
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
            selection_origin: Default::default(),
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
            selection_origin: Default::default(),
            layout,
        }
    }

    /// Shares the wave's final Native snapshot owner without registering or
    /// releasing another Native snapshot. The session owns this local share.
    /// The parts are one selected wave's: its runtime, product occurrence,
    /// retained application basis and selection provenance.
    pub(in crate::domain_computation) fn from_shared_selected<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        application: &dyn super::host::ApplicationAttemptHost<Schema>,
        product: &crate::basis::WorthQueryProductBranchLease,
        application_basis: &WorthQueryApplicationBasisLease,
        selection_origin: SelectedQueryOrigin,
    ) -> Result<Self, CompanionPreflightStop> {
        let graph = application
            .primary_graph()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let basis = application_basis.retain_shared()?;
        // Publication binding, two World observation Arc pairs, optional
        // guard, Bridge Arc and layout Arc are all shallow fixed copies.

        Ok(Self {
            custody: Some(SnapshotCustody::Shared(basis)),
            product: product.retained_clone(),
            selection_origin,
            layout: graph.retain_layout(),
        })
    }

    /// Takes a selection's provenance when that selection shares this lease's
    /// retained mutation source.
    pub(in crate::domain_computation::primary_graph) fn retain_selection_origin_from(
        &mut self,
        selected_product: &crate::basis::WorthQueryProductBranchLease,
        selection_origin: SelectedQueryOrigin,
    ) -> bool {
        if !self
            .product
            .shares_retained_mutation_source(selected_product)
        {
            return false;
        }
        self.selection_origin = selection_origin;
        true
    }

    pub(in crate::domain_computation::primary_graph) fn selection_origin(
        &self,
    ) -> SelectedQueryOrigin {
        self.selection_origin
    }

    pub(in crate::domain_computation) fn snapshot(&self) -> &SnapshotHandle {
        self.custody
            .as_ref()
            .expect("snapshot lease retains custody")
            .snapshot()
    }

    pub fn product(&self) -> &crate::basis::WorthQueryProductBranchLease {
        &self.product
    }

    pub(in crate::domain_computation) fn handle(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        self.custody
            .as_ref()
            .expect("snapshot lease retains custody")
            .handle()
    }

    pub(in crate::domain_computation) fn release(self) -> bool {
        self.release_custody().snapshot_released()
    }

    pub fn release_custody(mut self) -> WorthQueryApplicationSnapshotRelease {
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

mod court_doors_for_test;
