//! Moved whole from `domain_computation::primary_graph::application_runtime::certification_cost`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::provider::application_attempt_work::WorthQueryApplicationAttemptWorkSnapshot;
use worth_relational::facade::inspection::RelationalMvccCostObservation;
use worth_relational::facade::inspection::RelationalMvccCostScope;

/// A baseline of runtime cost counters for one product branch, captured before
/// the work being measured.
///
/// Get it from `capture_certification_cost_scope` and pass it to
/// `observe_certification_cost` after the work. Capturing it changes nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryCertificationCostScope {
    relational: RelationalMvccCostScope,
    application_work: WorthQueryApplicationAttemptWorkSnapshot,
    output_producer_attempts: u64,
    world_retention: WorthQueryCertificationWorldRetention,
    world_history: WorthQueryCertificationWorldHistory,
}

/// The cost of the work done since a [`WorthQueryCertificationCostScope`] was
/// captured: storage cost, application attempt work, output producer attempts,
/// and the product world's retention and history before and after.
///
/// Read-only evidence for certification; it grants nothing.
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

/// Counts of application attempt work steps done during an observed interval,
/// from resolutions and session preparations through prepared commits, aborts,
/// cleanups, and external dispatch admissions.
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

/// A snapshot of what the product world is holding: pins, component
/// obligations, acquisitions in flight or reserved, observations, and active
/// publication attempts.
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

/// A snapshot of the product world's commit history: installed and reserved
/// commits, metadata bytes, and reserved entry writes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryCertificationWorldHistory {
    installed_commits: usize,
    reserved_commits: usize,
    metadata_bytes: usize,
    reserved_entry_writes: u64,
    reachability_lookups: u64,
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
