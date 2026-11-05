//! Moved whole from `domain_computation::primary_graph::provider::application_attempt_work`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryApplicationAttemptWorkSnapshot {
    pub(crate) retained_resolutions: u64,
    pub(crate) managed_bridge_plans: u64,
    pub(crate) provider_session_readmissions: u64,
    pub(crate) provider_session_preparations: u64,
    pub(crate) staged_session_preparations: u64,
    pub(crate) attempt_registrations: u64,
    pub(crate) overlay_stagings: u64,
    pub(crate) invariant_state_loads: u64,
    pub(crate) invariant_executions: u64,
    pub(crate) prepared_commits: u64,
    pub(crate) attempt_aborts: u64,
    pub(crate) managed_cleanups: u64,
    pub(crate) external_dispatch_admissions: u64,
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    pub(crate) fn since(self, earlier: Self) -> Self {
        Self {
            retained_resolutions: self
                .retained_resolutions
                .saturating_sub(earlier.retained_resolutions),
            managed_bridge_plans: self
                .managed_bridge_plans
                .saturating_sub(earlier.managed_bridge_plans),
            provider_session_readmissions: self
                .provider_session_readmissions
                .saturating_sub(earlier.provider_session_readmissions),
            provider_session_preparations: self
                .provider_session_preparations
                .saturating_sub(earlier.provider_session_preparations),
            staged_session_preparations: self
                .staged_session_preparations
                .saturating_sub(earlier.staged_session_preparations),
            attempt_registrations: self
                .attempt_registrations
                .saturating_sub(earlier.attempt_registrations),
            overlay_stagings: self
                .overlay_stagings
                .saturating_sub(earlier.overlay_stagings),
            invariant_state_loads: self
                .invariant_state_loads
                .saturating_sub(earlier.invariant_state_loads),
            invariant_executions: self
                .invariant_executions
                .saturating_sub(earlier.invariant_executions),
            prepared_commits: self
                .prepared_commits
                .saturating_sub(earlier.prepared_commits),
            attempt_aborts: self.attempt_aborts.saturating_sub(earlier.attempt_aborts),
            managed_cleanups: self
                .managed_cleanups
                .saturating_sub(earlier.managed_cleanups),
            external_dispatch_admissions: self
                .external_dispatch_admissions
                .saturating_sub(earlier.external_dispatch_admissions),
        }
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn attempt_aborts_for_test(&self) -> &u64 {
        &self.attempt_aborts
    }
}
