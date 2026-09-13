//! Read-only cost evidence for the certification audience.

use std::sync::atomic::Ordering;

use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::inspection::{
    RelationalBranchSharingInspectionDenial, RelationalMvccCostObservation, RelationalMvccCostScope,
};

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::WorthQueryApplicationAttemptWorkSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryCertificationCostScope {
    relational: RelationalMvccCostScope,
    application_work: WorthQueryApplicationAttemptWorkSnapshot,
    output_producer_attempts: u64,
    world_retention: WorthQueryCertificationWorldRetention,
    world_history: WorthQueryCertificationWorldHistory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryCertificationCostObservation {
    relational: RelationalMvccCostObservation,
    application_work: WorthQueryCertificationApplicationWork,
    output_producer_attempts: u64,
    world_retention_before: WorthQueryCertificationWorldRetention,
    world_retention_after: WorthQueryCertificationWorldRetention,
    world_history_before: WorthQueryCertificationWorldHistory,
    world_history_after: WorthQueryCertificationWorldHistory,
    world_reserved_entry_writes: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryCertificationApplicationWork {
    retained_resolutions: u64,
    managed_bridge_plans: u64,
    provider_session_readmissions: u64,
    provider_session_preparations: u64,
    staged_session_preparations: u64,
    attempt_registrations: u64,
    overlay_stagings: u64,
    invariant_state_loads: u64,
    invariant_executions: u64,
    prepared_commits: u64,
    attempt_aborts: u64,
    managed_cleanups: u64,
    external_dispatch_admissions: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryCertificationWorldRetention {
    unique_pins: usize,
    component_obligations: usize,
    in_flight_acquisitions: usize,
    reserved_unique_pins: usize,
    reserved_acquisitions: usize,
    observations: usize,
    active_publication_attempts: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryCertificationWorldHistory {
    installed_commits: usize,
    reserved_commits: usize,
    metadata_bytes: usize,
    reserved_entry_writes: u64,
}

pub trait WorthQueryCertificationCostRuntimeExt<Schema: ApplicationSchema> {
    fn capture_certification_cost_scope(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        WorthQueryCertificationCostScope,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    >;

    fn observe_certification_cost(
        &self,
        scope: &WorthQueryCertificationCostScope,
    ) -> Result<WorthQueryCertificationCostObservation, RelationalBranchSharingInspectionDenial>;
}

impl<Schema: ApplicationSchema> WorthQueryCertificationCostRuntimeExt<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn capture_certification_cost_scope(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        WorthQueryCertificationCostScope,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        let lease = self
            .product_runtime
            .integration_admit_product_branch(branch)?;
        let relational_identity = lease.relational_basis().identity().clone();
        drop(lease);
        let relational = self.primary_provider.graph.with_runtime(|runtime| {
            RelationalMvccCostScope::capture(runtime, vec![relational_identity])
        });
        Ok(WorthQueryCertificationCostScope {
            relational,
            application_work: self.primary_provider.application_attempt_work(),
            output_producer_attempts: self.next_output_producer_attempt.load(Ordering::Relaxed),
            world_retention: self.certification_world_retention(),
            world_history: self.certification_world_history(),
        })
    }

    fn observe_certification_cost(
        &self,
        scope: &WorthQueryCertificationCostScope,
    ) -> Result<WorthQueryCertificationCostObservation, RelationalBranchSharingInspectionDenial>
    {
        let relational = self
            .primary_provider
            .graph
            .with_runtime(|runtime| runtime.observe_mvcc_cost(&scope.relational))?;
        let application_work = WorthQueryCertificationApplicationWork::from(
            self.primary_provider
                .application_attempt_work()
                .since(scope.application_work),
        );
        let world_retention_after = self.certification_world_retention();
        let world_history_after = self.certification_world_history();
        Ok(WorthQueryCertificationCostObservation {
            relational,
            application_work,
            output_producer_attempts: self
                .next_output_producer_attempt
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
        }
    }
}

impl WorthQueryCertificationCostObservation {
    pub fn relational(&self) -> &RelationalMvccCostObservation {
        &self.relational
    }
    pub const fn application_work(&self) -> WorthQueryCertificationApplicationWork {
        self.application_work
    }
    pub const fn output_producer_attempts(&self) -> u64 {
        self.output_producer_attempts
    }
    pub const fn world_retention_before(&self) -> WorthQueryCertificationWorldRetention {
        self.world_retention_before
    }
    pub const fn world_retention_after(&self) -> WorthQueryCertificationWorldRetention {
        self.world_retention_after
    }
    pub const fn world_history_before(&self) -> WorthQueryCertificationWorldHistory {
        self.world_history_before
    }
    pub const fn world_history_after(&self) -> WorthQueryCertificationWorldHistory {
        self.world_history_after
    }
    pub const fn world_reserved_entry_writes(&self) -> u64 {
        self.world_reserved_entry_writes
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
    reserved_entry_writes: u64,
);

impl From<WorthQueryApplicationAttemptWorkSnapshot> for WorthQueryCertificationApplicationWork {
    fn from(value: WorthQueryApplicationAttemptWorkSnapshot) -> Self {
        Self {
            retained_resolutions: value.retained_resolutions,
            managed_bridge_plans: value.managed_bridge_plans,
            provider_session_readmissions: value.provider_session_readmissions,
            provider_session_preparations: value.provider_session_preparations,
            staged_session_preparations: value.staged_session_preparations,
            attempt_registrations: value.attempt_registrations,
            overlay_stagings: value.overlay_stagings,
            invariant_state_loads: value.invariant_state_loads,
            invariant_executions: value.invariant_executions,
            prepared_commits: value.prepared_commits,
            attempt_aborts: value.attempt_aborts,
            managed_cleanups: value.managed_cleanups,
            external_dispatch_admissions: value.external_dispatch_admissions,
        }
    }
}
