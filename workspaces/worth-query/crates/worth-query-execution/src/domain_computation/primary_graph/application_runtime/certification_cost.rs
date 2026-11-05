//! Read-only cost evidence for the certification audience.

use std::sync::atomic::Ordering;

use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::inspection::{
    RelationalBranchSharingInspectionDenial, RelationalMvccCostObservation,
    RelationalMvccCostScope, RelationalRetainedStorageInspectionDenial,
    RelationalRetainedStorageObservation,
};

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::vocabulary::provider::application_attempt_work::WorthQueryApplicationAttemptWorkSnapshot;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationApplicationWork;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationCostObservation;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationCostScope;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationWorldHistory;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationWorldRetention;






/// Read-only cost measurement on an application runtime, for certification.
///
/// Capture a scope before the work, then observe the cost after it. Neither
/// call changes runtime state.
pub trait WorthQueryCertificationCostRuntimeExt<Schema: ApplicationSchema> {
    /// Explicit cold diagnostic work, separate from interval counters and
    /// selected-head sharing. Never called by ordinary publication or capture.
    fn observe_certification_retained_storage(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<RelationalRetainedStorageObservation, WorthQueryCertificationCostDenial>;
    /// Captures the cost baseline for a product branch. Refused when the branch
    /// cannot be admitted.
    fn capture_certification_cost_scope(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<WorthQueryCertificationCostScope, WorthQueryCertificationCostDenial>;

    /// Observes the cost accrued since `scope` was captured. Refused when the
    /// branch's storage cost cannot be inspected.
    fn observe_certification_cost(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        scope: &WorthQueryCertificationCostScope,
    ) -> Result<WorthQueryCertificationCostObservation, WorthQueryCertificationCostDenial>;
}

impl<Schema: ApplicationSchema> WorthQueryCertificationCostRuntimeExt<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn observe_certification_retained_storage(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<RelationalRetainedStorageObservation, WorthQueryCertificationCostDenial> {
        let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
        self.runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |runtime| runtime.observe_retained_storage())
            .map_err(WorthQueryCertificationCostDenial::RetainedStorage)
    }

    fn capture_certification_cost_scope(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<WorthQueryCertificationCostScope, WorthQueryCertificationCostDenial> {
        let lease = self
            .runtime
            .product_runtime
            .integration_admit_product_branch(branch)?;
        let relational_identity = lease.relational_basis().identity().clone();
        drop(lease);
        let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
        let relational = self
            .runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |runtime| {
                RelationalMvccCostScope::capture(runtime, vec![relational_identity])
            });
        Ok(WorthQueryCertificationCostScope {
            relational,
            application_work: self
                .runtime
                .primary_provider
                .application_attempt_work_for_test(),
            output_producer_attempts: self
                .output_production
                .next_producer_attempt()
                .load(Ordering::Relaxed),
            world_retention: self.certification_world_retention(),
            world_history: self.certification_world_history(),
        })
    }

    fn observe_certification_cost(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        scope: &WorthQueryCertificationCostScope,
    ) -> Result<WorthQueryCertificationCostObservation, WorthQueryCertificationCostDenial> {
        let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
        let relational = self
            .runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |runtime| {
                runtime.observe_mvcc_cost(&scope.relational)
            })?;
        let application_work = WorthQueryCertificationApplicationWork::from(
            self.runtime
                .primary_provider
                .application_attempt_work_for_test()
                .since(scope.application_work),
        );
        let world_retention_after = self.certification_world_retention();
        let world_history_after = self.certification_world_history();
        Ok(WorthQueryCertificationCostObservation {
            relational,
            application_work,
            output_producer_attempts: self
                .output_production
                .next_producer_attempt()
                .load(Ordering::Relaxed)
                .saturating_sub(scope.output_producer_attempts),
            world_retention_before: scope.world_retention,
            world_retention_after,
            world_history_before: scope.world_history,
            world_history_after,
            world_reserved_entry_writes: world_history_after
                .reserved_entry_writes
                .saturating_sub(scope.world_history.reserved_entry_writes),
        })
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn certification_world_retention(&self) -> WorthQueryCertificationWorldRetention {
        let snapshot = self
            .runtime
            .product_runtime
            .owner
            .inspection_port()
            .retention_snapshot()
            .expect("the installed World remains inspectable");
        WorthQueryCertificationWorldRetention {
            unique_pins: snapshot.unique_pins(),
            component_obligations: snapshot.component_obligations(),
            in_flight_acquisitions: snapshot.in_flight_acquisitions(),
            reserved_unique_pins: snapshot.reserved_unique_pins(),
            reserved_acquisitions: snapshot.reserved_acquisitions(),
            observations: snapshot.observations(),
            active_publication_attempts: snapshot.active_publication_attempts(),
        }
    }

    fn certification_world_history(&self) -> WorthQueryCertificationWorldHistory {
        let snapshot = self
            .runtime
            .product_runtime
            .owner
            .inspection_port()
            .history_snapshot()
            .expect("the installed World remains inspectable");
        WorthQueryCertificationWorldHistory {
            installed_commits: snapshot.installed_commits(),
            reserved_commits: snapshot.reserved_commits(),
            metadata_bytes: snapshot.metadata().total_occupancy(),
            reserved_entry_writes: snapshot.costs().reserved_entry_writes(),
            reachability_lookups: snapshot.costs().reachability_lookups(),
        }
    }
}


macro_rules! accessors {
    ($type:ty; $($name:ident : $kind:ty),+ $(,)?) => {
        impl $type { $(pub const fn $name(self) -> $kind { self.$name })+ }
    };
}

accessors!(WorthQueryCertificationApplicationWork;
    retained_resolutions: u64, managed_bridge_plans: u64,
    provider_session_readmissions: u64, provider_session_preparations: u64,
    staged_session_preparations: u64, attempt_registrations: u64,
    overlay_stagings: u64, invariant_state_loads: u64, invariant_executions: u64,
    prepared_commits: u64, attempt_aborts: u64, managed_cleanups: u64,
    external_dispatch_admissions: u64,
);
accessors!(WorthQueryCertificationWorldRetention;
    unique_pins: usize, component_obligations: usize, in_flight_acquisitions: usize,
    reserved_unique_pins: usize, reserved_acquisitions: usize, observations: usize,
    active_publication_attempts: usize,
);
accessors!(WorthQueryCertificationWorldHistory;
    installed_commits: usize, reserved_commits: usize, metadata_bytes: usize,
    reserved_entry_writes: u64, reachability_lookups: u64,
);


#[derive(Debug)]
pub enum WorthQueryCertificationCostDenial {
    Branch(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Sharing(RelationalBranchSharingInspectionDenial),
    RetainedStorage(RelationalRetainedStorageInspectionDenial),
    ForeignPhase(crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase),
}
impl From<crate::basis::WorthQueryProductBranchAdmissionDenial>
    for WorthQueryCertificationCostDenial
{
    fn from(cause: crate::basis::WorthQueryProductBranchAdmissionDenial) -> Self {
        Self::Branch(cause)
    }
}
impl From<RelationalBranchSharingInspectionDenial> for WorthQueryCertificationCostDenial {
    fn from(cause: RelationalBranchSharingInspectionDenial) -> Self {
        Self::Sharing(cause)
    }
}
impl From<crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase>
    for WorthQueryCertificationCostDenial
{
    fn from(
        cause: crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase,
    ) -> Self {
        Self::ForeignPhase(cause)
    }
}
