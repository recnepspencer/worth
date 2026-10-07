use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::lifecycle_count::{acquire, record_one_saturating};

mod lease;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;
pub(crate) use lease::WorthQueryApplicationBasisLease;
use worth_relational::facade::{
    branch::{
        AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor,
        RelationalBranchRetentionTerminalOutcome,
    },
    history::BranchId,
    snapshots::SnapshotId,
};

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

/// Read-only view of the query bases this runtime holds open.
///
/// Get one from `application_query_basis_observer` and call `observe` for a
/// point-in-time count. Observing grants nothing and changes nothing.
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

/// Distinguishes physical Native resource release from ending one local share
/// while another admitted Query owner retains the same resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationBasisReleaseOutcome {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
    NativeResourcesReleased {
        snapshot_released: bool,
        relational_retention: RelationalBranchRetentionTerminalOutcome,
    },
    /// This local Query share ended; another admitted share still owns the
    /// authentic snapshot, Native retention and program interpretation.
    SharedCustodyRetained,
}

impl WorthQueryApplicationBasisReleaseReceipt {
    pub fn identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.identity
    }

    pub const fn released(&self) -> bool {
        self.outcome.released()
    }

    pub const fn custody_released(&self) -> bool {
        self.outcome.custody_released()
    }

    pub const fn outcome(&self) -> WorthQueryApplicationBasisReleaseOutcome {
        self.outcome
    }
}

impl WorthQueryApplicationBasisReleaseOutcome {
    /// Whether this release physically closed both native resources.
    pub const fn released(self) -> bool {
        matches!(
            self,
            Self::NativeResourcesReleased {
                snapshot_released: true,
                relational_retention: RelationalBranchRetentionTerminalOutcome::Released,
            }
        )
    }

    /// Whether this lease surrendered its complete local custody. A surviving
    /// shared owner is explicitly distinct from physical native release.
    pub const fn custody_released(self) -> bool {
        self.released() || matches!(self, Self::SharedCustodyRetained)
    }

    pub const fn snapshot_released(self) -> bool {
        matches!(
            self,
            Self::NativeResourcesReleased {
                snapshot_released: true,
                ..
            }
        )
    }

    pub const fn relational_retention(self) -> Option<RelationalBranchRetentionTerminalOutcome> {
        match self {
            Self::NativeResourcesReleased {
                relational_retention,
                ..
            } => Some(relational_retention),
            Self::SharedCustodyRetained | Self::Handle(_) => None,
        }
    }
}

/// Point-in-time counts of query bases: how many are open now, the most ever
/// open at once, and how many were ever acquired.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisObservation {
    active: usize,
    peak_active: usize,
    acquisitions: usize,
}

pub(crate) enum WorthQueryApplicationBasisRegistrationDenial {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
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
            })?
            .map_err(WorthQueryApplicationBasisRegistrationDenial::Snapshot)?;
        let retention = graph.with_runtime(|runtime| runtime.retain_component_basis(&basis))?;
        let retention = match retention {
            Ok(retention) => retention,
            Err(denial) => {
                graph.with_runtime_mut(|runtime| {
                    crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                })?;
                return Err(WorthQueryApplicationBasisRegistrationDenial::Basis(denial));
            }
        };
        let active = acquire(&self.state.active, 1)
            .expect("live application-query basis count cannot overflow");
        record_one_saturating(&self.state.acquisitions);
        self.state.peak_active.fetch_max(active, Ordering::AcqRel);
        Ok(WorthQueryApplicationBasisLease::registered(
            WorthQueryApplicationBasisIdentity {
                runtime_instance_id: basis.identity().runtime_instance_id(),
                branch_id: basis.identity().branch_id().clone(),
                snapshot_id: snapshot.snapshot_id(),
                descriptor: basis.descriptor().clone(),
                selection: WorthQueryApplicationBasisSelectionIdentity::Relational,
            },
            basis,
            retention,
            snapshot,
            graph,
            Arc::clone(&self.state),
        ))
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

impl From<crate::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryApplicationBasisRegistrationDenial
{
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::Handle(denial)
    }
}
