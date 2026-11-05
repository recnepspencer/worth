//! Sealed test doors over this module's private members.
use crate::domain_computation::primary_graph::vocabulary::provider::application_attempt_work::WorthQueryApplicationAttemptWorkSnapshot;

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn attempt_registrations_for_test(&self) -> &u64 {
        &self.attempt_registrations
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn external_dispatch_admissions_for_test(&self) -> &u64 {
        &self.external_dispatch_admissions
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn invariant_executions_for_test(&self) -> &u64 {
        &self.invariant_executions
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn invariant_state_loads_for_test(&self) -> &u64 {
        &self.invariant_state_loads
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn managed_bridge_plans_for_test(&self) -> &u64 {
        &self.managed_bridge_plans
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn managed_cleanups_for_test(&self) -> &u64 {
        &self.managed_cleanups
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn overlay_stagings_for_test(&self) -> &u64 {
        &self.overlay_stagings
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn prepared_commits_for_test(&self) -> &u64 {
        &self.prepared_commits
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn provider_session_preparations_for_test(&self) -> &u64 {
        &self.provider_session_preparations
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn provider_session_readmissions_for_test(&self) -> &u64 {
        &self.provider_session_readmissions
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn staged_session_preparations_for_test(&self) -> &u64 {
        &self.staged_session_preparations
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn since_for_test(self, earlier: Self) -> Self {
        Self::since(self, earlier)
    }
}

impl WorthQueryApplicationAttemptWorkSnapshot {
    #[doc(hidden)]
    pub fn retained_resolutions_for_test(&self) -> &u64 {
        &self.retained_resolutions
    }
}
