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
    pub(in crate::domain_computation::primary_graph) relational: RelationalMvccCostScope,
    pub(in crate::domain_computation::primary_graph) application_work:
        WorthQueryApplicationAttemptWorkSnapshot,
    pub(in crate::domain_computation::primary_graph) output_producer_attempts: u64,
    pub(in crate::domain_computation::primary_graph) world_retention:
        WorthQueryCertificationWorldRetention,
    pub(in crate::domain_computation::primary_graph) world_history:
        WorthQueryCertificationWorldHistory,
}

/// The cost of the work done since a [`WorthQueryCertificationCostScope`] was
/// captured: storage cost, application attempt work, output producer attempts,
/// and the product world's retention and history before and after.
///
/// Read-only evidence for certification; it grants nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryCertificationCostObservation {
    pub(in crate::domain_computation::primary_graph) relational: RelationalMvccCostObservation,
    pub(in crate::domain_computation::primary_graph) application_work:
        WorthQueryCertificationApplicationWork,
    pub(in crate::domain_computation::primary_graph) output_producer_attempts: u64,
    pub(in crate::domain_computation::primary_graph) world_retention_before:
        WorthQueryCertificationWorldRetention,
    pub(in crate::domain_computation::primary_graph) world_retention_after:
        WorthQueryCertificationWorldRetention,
    pub(in crate::domain_computation::primary_graph) world_history_before:
        WorthQueryCertificationWorldHistory,
    pub(in crate::domain_computation::primary_graph) world_history_after:
        WorthQueryCertificationWorldHistory,
    pub(in crate::domain_computation::primary_graph) world_reserved_entry_writes: u64,
}

/// Counts of application attempt work steps done during an observed interval,
/// from resolutions and session preparations through prepared commits, aborts,
/// cleanups, and external dispatch admissions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryCertificationApplicationWork {
    pub(in crate::domain_computation::primary_graph) retained_resolutions: u64,
    pub(in crate::domain_computation::primary_graph) managed_bridge_plans: u64,
    pub(in crate::domain_computation::primary_graph) provider_session_readmissions: u64,
    pub(in crate::domain_computation::primary_graph) provider_session_preparations: u64,
    pub(in crate::domain_computation::primary_graph) staged_session_preparations: u64,
    pub(in crate::domain_computation::primary_graph) attempt_registrations: u64,
    pub(in crate::domain_computation::primary_graph) overlay_stagings: u64,
    pub(in crate::domain_computation::primary_graph) invariant_state_loads: u64,
    pub(in crate::domain_computation::primary_graph) invariant_executions: u64,
    pub(in crate::domain_computation::primary_graph) prepared_commits: u64,
    pub(in crate::domain_computation::primary_graph) attempt_aborts: u64,
    pub(in crate::domain_computation::primary_graph) managed_cleanups: u64,
    pub(in crate::domain_computation::primary_graph) external_dispatch_admissions: u64,
}

/// A snapshot of what the product world is holding: pins, component
/// obligations, acquisitions in flight or reserved, observations, and active
/// publication attempts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryCertificationWorldRetention {
    pub(in crate::domain_computation::primary_graph) unique_pins: usize,
    pub(in crate::domain_computation::primary_graph) component_obligations: usize,
    pub(in crate::domain_computation::primary_graph) in_flight_acquisitions: usize,
    pub(in crate::domain_computation::primary_graph) reserved_unique_pins: usize,
    pub(in crate::domain_computation::primary_graph) reserved_acquisitions: usize,
    pub(in crate::domain_computation::primary_graph) observations: usize,
    pub(in crate::domain_computation::primary_graph) active_publication_attempts: usize,
}

/// A snapshot of the product world's commit history: installed and reserved
/// commits, metadata bytes, and reserved entry writes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryCertificationWorldHistory {
    pub(in crate::domain_computation::primary_graph) installed_commits: usize,
    pub(in crate::domain_computation::primary_graph) reserved_commits: usize,
    pub(in crate::domain_computation::primary_graph) metadata_bytes: usize,
    pub(in crate::domain_computation::primary_graph) reserved_entry_writes: u64,
    pub(in crate::domain_computation::primary_graph) reachability_lookups: u64,
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
