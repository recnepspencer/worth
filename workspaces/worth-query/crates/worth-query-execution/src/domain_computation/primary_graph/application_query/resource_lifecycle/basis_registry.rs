use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_relational::facade::{
    branch::{
        AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor,
        RelationalBranchRetentionLease, RelationalBranchRetentionTerminalOutcome,
    },
    history::BranchId,
    snapshots::{SnapshotHandle, SnapshotId},
};
use worth_runtime_bridge::facade::BridgePreviewSessionLivenessObserver;

use super::lifecycle_count::{acquire, record_one_saturating, release};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

#[derive(Default)]
struct WorthQueryApplicationBasisRegistryState {
    active: AtomicUsize,
    peak_active: AtomicUsize,
    acquisitions: AtomicUsize,
}

#[derive(Default)]
pub(crate) struct WorthQueryApplicationBasisRegistry {
    state: Arc<WorthQueryApplicationBasisRegistryState>,
}

#[derive(Clone)]
pub struct WorthQueryApplicationBasisObserver {
    state: Arc<WorthQueryApplicationBasisRegistryState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisIdentity {
    runtime_instance_id: u64,
    branch_id: BranchId,
    snapshot_id: SnapshotId,
    descriptor: RelationalBranchBasisDescriptor,
    selection: WorthQueryApplicationBasisSelectionIdentity,
}

/// Descriptive origin of this exact snapshot; it grants no read authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationBasisSelectionIdentity {
    Relational,
    Product(crate::basis::WorthQueryProductBranchReadIdentity),
}

impl WorthQueryApplicationBasisIdentity {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub const fn snapshot_id(&self) -> SnapshotId {
        self.snapshot_id
    }

    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.descriptor
    }

    pub fn selection(&self) -> &WorthQueryApplicationBasisSelectionIdentity {
        &self.selection
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisReleaseReceipt {
    identity: WorthQueryApplicationBasisIdentity,
    outcome: WorthQueryApplicationBasisReleaseOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisReleaseOutcome {
    snapshot_released: bool,
    relational_retention: RelationalBranchRetentionTerminalOutcome,
}

impl WorthQueryApplicationBasisReleaseReceipt {
    pub fn identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.identity
    }

    pub const fn released(&self) -> bool {
        self.outcome.released()
    }

    pub const fn outcome(&self) -> WorthQueryApplicationBasisReleaseOutcome {
        self.outcome
    }
}

impl WorthQueryApplicationBasisReleaseOutcome {
    pub const fn released(self) -> bool {
        self.snapshot_released
            && matches!(
                self.relational_retention,
                RelationalBranchRetentionTerminalOutcome::Released
            )
    }

    pub const fn snapshot_released(self) -> bool {
        self.snapshot_released
    }

    pub const fn relational_retention(self) -> RelationalBranchRetentionTerminalOutcome {
        self.relational_retention
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisObservation {
    active: usize,
    peak_active: usize,
    acquisitions: usize,
}

pub(crate) struct WorthQueryApplicationBasisLease {
    identity: WorthQueryApplicationBasisIdentity,
    basis: Option<AdmittedRelationalBranchBasis>,
    retention: Option<RelationalBranchRetentionLease>,
    snapshot: Option<SnapshotHandle>,
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    preview_session_liveness: Option<BridgePreviewSessionLivenessObserver>,
    state: Arc<WorthQueryApplicationBasisRegistryState>,
}

pub(crate) enum WorthQueryApplicationBasisRegistrationDenial {
    Basis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    Snapshot(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
}

impl WorthQueryApplicationBasisRegistry {
    pub(in crate::domain_computation::primary_graph) fn observer(
        &self,
    ) -> WorthQueryApplicationBasisObserver {
        WorthQueryApplicationBasisObserver {
            state: Arc::clone(&self.state),
        }
    }

    pub(crate) fn register(
        &self,
        basis: AdmittedRelationalBranchBasis,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
    ) -> Result<WorthQueryApplicationBasisLease, WorthQueryApplicationBasisRegistrationDenial> {
        let snapshot = graph
            .with_runtime_mut(|runtime| {
                crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                    runtime,
                    &basis,
                )
            })
            .map_err(WorthQueryApplicationBasisRegistrationDenial::Snapshot)?;
        let retention = graph.with_runtime(|runtime| runtime.retain_component_basis(&basis));
        let retention = match retention {
            Ok(retention) => retention,
            Err(denial) => {
                graph.with_runtime_mut(|runtime| {
                    crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                });
                return Err(WorthQueryApplicationBasisRegistrationDenial::Basis(denial));
            }
        };
        let active = acquire(&self.state.active, 1)
            .expect("live application-query basis count cannot overflow");
        record_one_saturating(&self.state.acquisitions);
        self.state.peak_active.fetch_max(active, Ordering::AcqRel);
        Ok(WorthQueryApplicationBasisLease {
            identity: WorthQueryApplicationBasisIdentity {
                runtime_instance_id: basis.identity().runtime_instance_id(),
                branch_id: basis.identity().branch_id().clone(),
                snapshot_id: snapshot.snapshot_id(),
                descriptor: basis.descriptor().clone(),
                selection: WorthQueryApplicationBasisSelectionIdentity::Relational,
            },
            basis: Some(basis),
            retention: Some(retention),
            snapshot: Some(snapshot),
            graph,
            preview_session_liveness: None,
            state: Arc::clone(&self.state),
        })
    }
}

impl WorthQueryApplicationBasisObserver {
    pub fn observe(&self) -> WorthQueryApplicationBasisObservation {
        WorthQueryApplicationBasisObservation {
            active: self.state.active.load(Ordering::Acquire),
            peak_active: self.state.peak_active.load(Ordering::Acquire),
            acquisitions: self.state.acquisitions.load(Ordering::Acquire),
        }
    }
}

impl WorthQueryApplicationBasisObservation {
    pub const fn active(self) -> usize {
        self.active
    }

    pub const fn peak_active(self) -> usize {
        self.peak_active
    }

    pub const fn acquisitions(self) -> usize {
        self.acquisitions
    }
}

impl WorthQueryApplicationBasisLease {
    pub(crate) fn bind_product_observation(
        &mut self,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        assert_eq!(
            self.identity.descriptor(),
            product.basis().relational_basis().descriptor(),
            "product custody retains the exact query snapshot basis"
        );
        self.identity.selection = WorthQueryApplicationBasisSelectionIdentity::Product(
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(product),
        );
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn bind_preview_session(
        mut self,
        liveness: BridgePreviewSessionLivenessObserver,
    ) -> Self {
        self.preview_session_liveness = Some(liveness);
        self
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn preview_session_liveness(
        &self,
    ) -> Option<&BridgePreviewSessionLivenessObserver> {
        self.preview_session_liveness.as_ref()
    }

    pub fn identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.identity
    }

    pub fn version_id(&self) -> worth_relational::facade::identity::VersionId {
        self.basis().observation().version_id()
    }

    pub fn snapshot_handle(&self) -> &SnapshotHandle {
        self.snapshot
            .as_ref()
            .expect("an active application-query basis retains its snapshot")
    }

    pub fn is_live(&self) -> bool {
        self.basis.is_some()
            && self.snapshot.as_ref().is_some_and(|snapshot| {
                self.graph.with_runtime(|runtime| {
                    runtime.read_truth().project_snapshot(snapshot).is_some()
                })
            })
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retain_for_continuation(
        &self,
    ) -> Result<
        RelationalBranchRetentionLease,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    > {
        self.graph
            .with_runtime(|runtime| runtime.retain_component_basis(self.basis()))
    }

    pub fn release(mut self) -> WorthQueryApplicationBasisReleaseReceipt {
        let released = self.release_snapshot();
        let retention = self
            .retention
            .take()
            .expect("an active application-query basis retains its Relational lease");
        let retention_receipt = self
            .graph
            .with_runtime(|runtime| runtime.release_component_basis(retention))
            .unwrap_or_else(|denial| {
                panic!(
                    "Query retained a lease that its exact Relational owner rejected: {:?}",
                    denial.denial()
                )
            });
        assert_eq!(
            retention_receipt.descriptor(),
            self.identity.descriptor(),
            "Query releases the exact Relational basis it retained"
        );
        self.basis.take();
        self.release_observation();
        WorthQueryApplicationBasisReleaseReceipt {
            identity: self.identity.clone(),
            outcome: WorthQueryApplicationBasisReleaseOutcome {
                snapshot_released: released,
                relational_retention: retention_receipt.outcome(),
            },
        }
    }

    fn basis(&self) -> &AdmittedRelationalBranchBasis {
        self.basis
            .as_ref()
            .expect("an active application-query basis retains its Relational observation")
    }

    fn release_snapshot(&mut self) -> bool {
        let Some(snapshot) = self.snapshot.take() else {
            return false;
        };
        self.graph.with_runtime_mut(|runtime| {
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        });
        true
    }

    fn release_observation(&self) {
        release(&self.state.active, 1)
            .expect("live application-query basis count cannot underflow");
    }
}

impl Drop for WorthQueryApplicationBasisLease {
    fn drop(&mut self) {
        if self.basis.take().is_some() {
            self.release_snapshot();
            self.retention.take();
            self.release_observation();
        }
    }
}
