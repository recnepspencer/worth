use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    admit_primary_runtime_granular_batch, WorthQueryOperationResultState,
    WorthQuerySemanticDependencyRole, WorthQuerySharedLiveProjectionLease,
};
use crate::runtime::{
    WorthQuerySharedExecutionOwnerIdentity, WorthQuerySharedProjectionLeaseIdentity,
};
use std::collections::BTreeMap;

use super::consumer_delivery::current_shared_consumer_delivery_authority;
use super::primary_runtime::{
    coalesced_plan, full_scope_plan, prepare_covered_projection_maintenance,
};
use super::{WorthQueryMaintenanceScope, WorthQueryMaintenanceStrategy};
use worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationCoverage;

#[path = "shared_primary/consumer_set.rs"]
mod consumer_set;
use consumer_set::admitted_consumer_set;

pub struct WorthQueryPublishedSharedPrimaryInvalidation {
    owner: WorthQuerySharedExecutionOwnerIdentity,
    lease: WorthQuerySharedProjectionLeaseIdentity,
    strategies: Vec<WorthQueryMaintenanceStrategy>,
    scope: WorthQueryMaintenanceScope,
    roles: Vec<WorthQuerySemanticDependencyRole>,
    result_state: WorthQueryOperationResultState,
    consumer_binding_identity: String,
    consumer_operation_identity: String,
    consumer_delivery_authority: super::WorthQuerySharedConsumerDeliveryAuthority,
    delivery_identity: String,
    effect: std::sync::Arc<super::WorthQueryPerformedMaintenanceEffect>,
}

impl WorthQueryPublishedSharedPrimaryInvalidation {
    pub const fn owner_identity(&self) -> WorthQuerySharedExecutionOwnerIdentity {
        self.owner
    }

    pub const fn lease_identity(&self) -> WorthQuerySharedProjectionLeaseIdentity {
        self.lease
    }

    pub fn strategies(&self) -> &[WorthQueryMaintenanceStrategy] {
        &self.strategies
    }

    pub const fn scope(&self) -> &WorthQueryMaintenanceScope {
        &self.scope
    }

    pub fn roles(&self) -> &[WorthQuerySemanticDependencyRole] {
        &self.roles
    }

    pub const fn result_state(&self) -> WorthQueryOperationResultState {
        self.result_state
    }

    pub fn delivery_identity(&self) -> &str {
        &self.delivery_identity
    }

    pub fn consumer_binding_identity(&self) -> &str {
        &self.consumer_binding_identity
    }

    pub fn consumer_operation_identity(&self) -> &str {
        &self.consumer_operation_identity
    }

    pub const fn consumer_delivery_authority(
        &self,
    ) -> &super::WorthQuerySharedConsumerDeliveryAuthority {
        &self.consumer_delivery_authority
    }

    pub fn effect(&self) -> &super::WorthQueryPerformedMaintenanceEffect {
        &self.effect
    }
}

pub struct WorthQuerySharedPrimaryGranularMaintenancePerformed {
    _refresh: crate::domain_installation::WorthQueryLiveProjectionRefresh,
    publications: Vec<WorthQueryPublishedSharedPrimaryInvalidation>,
    denied_consumer_count: usize,
    admission_counters: crate::domain_installation::WorthQueryGranularAdmissionCounters,
    maintenance_counters: super::WorthQueryGranularMaintenanceCounters,
    impact_observations: Vec<crate::domain_installation::WorthQueryAdmittedInvalidationObservation>,
}

impl WorthQuerySharedPrimaryGranularMaintenancePerformed {
    pub fn publications(&self) -> &[WorthQueryPublishedSharedPrimaryInvalidation] {
        &self.publications
    }

    pub const fn shared_execution_count(&self) -> usize {
        1
    }

    pub const fn denied_consumer_count(&self) -> usize {
        self.denied_consumer_count
    }

    pub const fn admission_counters(
        &self,
    ) -> crate::domain_installation::WorthQueryGranularAdmissionCounters {
        self.admission_counters
    }

    pub const fn maintenance_counters(&self) -> super::WorthQueryGranularMaintenanceCounters {
        self.maintenance_counters
    }

    #[doc(hidden)]
    pub fn impact_observations(
        &self,
    ) -> &[crate::domain_installation::WorthQueryAdmittedInvalidationObservation] {
        &self.impact_observations
    }
}

pub enum WorthQuerySharedPrimaryGranularMaintenanceOutcome {
    NoRelevantChange,
    Performed(WorthQuerySharedPrimaryGranularMaintenancePerformed),
}

pub struct WorthQueryPreparedSharedPrimaryGranularMaintenance {
    owner: WorthQuerySharedExecutionOwnerIdentity,
    selected_consumers: BTreeMap<
        WorthQuerySharedProjectionLeaseIdentity,
        super::WorthQuerySharedConsumerDeliveryAuthority,
    >,
    plan: super::primary_runtime::WorthQueryCoalescedMaintenancePlan,
    impacts: Vec<crate::domain_installation::WorthQueryAdmittedInvalidationImpact>,
    admission_counters: crate::domain_installation::WorthQueryGranularAdmissionCounters,
    coverage: WorthQueryGranularInvalidationCoverage,
}

pub enum WorthQuerySharedPrimaryGranularSelectionOutcome {
    NoRelevantChange,
    Prepared(WorthQueryPreparedSharedPrimaryGranularMaintenance),
}

#[derive(Debug)]
pub enum WorthQuerySharedPrimaryGranularMaintenanceDenial {
    EmptyConsumerSet,
    ForeignPrimaryRuntime,
    ConsumerSetMismatch,
    Admission(crate::domain_installation::WorthQueryImpactAdmissionDenial),
    MixedMaintenancePosture,
    Runtime(crate::runtime::WorthQueryRuntimeError),
    Execution(crate::domain_installation::WorthQueryLiveProjectionRefreshError),
    ConsumerDeliveryPolicyRequired,
    Maintenance(super::WorthQueryMaintenanceDenial),
}

pub fn maintain_shared_primary_runtime_granular_batch<
    D: 'static,
    O: 'static,
    F: 'static,
    L: BasisOperationLane,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    consumers: &[&WorthQuerySharedLiveProjectionLease<D, O, F, L>],
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    binding: &super::WorthQueryPrimaryRuntimeInvalidationBinding,
    batch: worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
) -> Result<
    WorthQuerySharedPrimaryGranularMaintenanceOutcome,
    WorthQuerySharedPrimaryGranularMaintenanceDenial,
> {
    match prepare_shared_primary_runtime_granular_batch(consumers, workspace, binding, batch)? {
        WorthQuerySharedPrimaryGranularSelectionOutcome::NoRelevantChange => {
            Ok(WorthQuerySharedPrimaryGranularMaintenanceOutcome::NoRelevantChange)
        }
        WorthQuerySharedPrimaryGranularSelectionOutcome::Prepared(prepared) => {
            perform_prepared_shared_primary_runtime_granular_maintenance(
                execution, prepared, consumers, workspace, binding,
            )
        }
    }
}

pub fn prepare_shared_primary_runtime_granular_batch<
    D: 'static,
    O: 'static,
    F: 'static,
    L: BasisOperationLane,
>(
    consumers: &[&WorthQuerySharedLiveProjectionLease<D, O, F, L>],
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    binding: &super::WorthQueryPrimaryRuntimeInvalidationBinding,
    batch: worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
) -> Result<
    WorthQuerySharedPrimaryGranularSelectionOutcome,
    WorthQuerySharedPrimaryGranularMaintenanceDenial,
> {
    let Some(first) = consumers.first().copied() else {
        return Err(WorthQuerySharedPrimaryGranularMaintenanceDenial::EmptyConsumerSet);
    };
    if !binding.readmits_workspace(workspace) {
        return Err(WorthQuerySharedPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime);
    }
    let admitted = admit_primary_runtime_granular_batch(first.snapshot(), binding, batch)
        .map_err(WorthQuerySharedPrimaryGranularMaintenanceDenial::Admission)?;
    let coverage = admitted.coverage();
    if admitted.is_empty() && coverage == WorthQueryGranularInvalidationCoverage::Exact {
        return Ok(WorthQuerySharedPrimaryGranularSelectionOutcome::NoRelevantChange);
    }
    let admission_counters = admitted.admission_counters();
    let (impacts, _, source_read_basis) = admitted.into_parts();
    let plan = match coverage {
        // Incomplete impacts: refresh the owner's full scope instead.
        WorthQueryGranularInvalidationCoverage::RefreshAll => full_scope_plan(
            source_read_basis,
            WorthQueryMaintenanceStrategy::LocalProjectionPatch,
        ),
        WorthQueryGranularInvalidationCoverage::Exact => {
            coalesced_plan(&impacts, source_read_basis)
                .ok_or(WorthQuerySharedPrimaryGranularMaintenanceDenial::MixedMaintenancePosture)?
        }
    };
    let owner = first.owner_identity();
    let selected_consumers = admitted_consumer_set(consumers, owner, workspace)?;
    Ok(WorthQuerySharedPrimaryGranularSelectionOutcome::Prepared(
        WorthQueryPreparedSharedPrimaryGranularMaintenance {
            owner,
            selected_consumers,
            plan,
            impacts,
            admission_counters,
            coverage,
        },
    ))
}

#[path = "shared_primary/perform.rs"]
mod perform;
pub use perform::perform_prepared_shared_primary_runtime_granular_maintenance;
