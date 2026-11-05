use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::lifecycle_count::{acquire, record_one_saturating};

pub(crate) mod lease;
use crate::domain_computation::primary_graph::vocabulary::application_query::basis_registry::WorthQueryApplicationBasisRegistrationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity;
use crate::domain_computation::vocabulary::primary_graph::basis_registry::WorthQueryApplicationBasisIdentity;
pub(crate) use lease::WorthQueryApplicationBasisLease;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;

#[derive(Default)]
struct WorthQueryApplicationBasisRegistryState {
    active: AtomicUsize,
    peak_active: AtomicUsize,
    acquisitions: AtomicUsize,
}

#[derive(Default)]
pub struct WorthQueryApplicationBasisRegistry {
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

/// Point-in-time counts of query bases: how many are open now, the most ever
/// open at once, and how many were ever acquired.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisObservation {
    active: usize,
    peak_active: usize,
    acquisitions: usize,
}

impl WorthQueryApplicationBasisRegistry {
    pub fn observer(&self) -> WorthQueryApplicationBasisObserver {
        WorthQueryApplicationBasisObserver {
            state: Arc::clone(&self.state),
        }
    }

    pub(crate) fn register(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: AdmittedRelationalBranchBasis,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
    ) -> Result<WorthQueryApplicationBasisLease, WorthQueryApplicationBasisRegistrationDenial> {
        let snapshot = graph
            .with_runtime_mut(execution, |runtime| {
                crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                    runtime,
                    &basis,
                )
            })
            .map_err(WorthQueryApplicationBasisRegistrationDenial::Snapshot)?;
        let retention =
            graph.with_runtime(execution, |runtime| runtime.retain_component_basis(&basis));
        let retention = match retention {
            Ok(retention) => retention,
            Err(denial) => {
                graph.release_query_snapshot(&snapshot);
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

mod court_doors_for_test;
