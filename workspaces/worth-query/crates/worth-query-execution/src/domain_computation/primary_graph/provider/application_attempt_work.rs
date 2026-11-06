//! Runtime-private structural work for Primary Graph application attempts.

use crate::domain_computation::primary_graph::vocabulary::provider::application_attempt_work::WorthQueryApplicationAttemptWorkSnapshot;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub(super) struct WorthQueryApplicationAttemptWorkLedger {
    retained_resolutions: AtomicU64,
    managed_bridge_plans: AtomicU64,
    provider_session_readmissions: AtomicU64,
    provider_session_preparations: AtomicU64,
    staged_session_preparations: AtomicU64,
    attempt_registrations: AtomicU64,
    overlay_stagings: AtomicU64,
    invariant_state_loads: AtomicU64,
    invariant_executions: AtomicU64,
    prepared_commits: AtomicU64,
    attempt_aborts: AtomicU64,
    managed_cleanups: AtomicU64,
    external_dispatch_admissions: AtomicU64,
}

macro_rules! observation {
    ($observe:ident, $field:ident) => {
        pub(super) fn $observe(&self) {
            self.$field.fetch_add(1, Ordering::Relaxed);
        }
    };
}

impl WorthQueryApplicationAttemptWorkLedger {
    observation!(observe_retained_resolution, retained_resolutions);
    observation!(observe_managed_bridge_plan, managed_bridge_plans);
    observation!(
        observe_provider_session_readmission,
        provider_session_readmissions
    );
    observation!(
        observe_provider_session_preparation,
        provider_session_preparations
    );
    observation!(
        observe_staged_session_preparation,
        staged_session_preparations
    );
    observation!(observe_attempt_registration, attempt_registrations);
    observation!(observe_overlay_staging, overlay_stagings);
    observation!(observe_invariant_state_load, invariant_state_loads);
    observation!(observe_invariant_execution, invariant_executions);
    observation!(observe_prepared_commit, prepared_commits);
    observation!(observe_attempt_abort, attempt_aborts);
    observation!(observe_managed_cleanup, managed_cleanups);
    observation!(
        observe_external_dispatch_admission,
        external_dispatch_admissions
    );

    pub(super) fn snapshot(&self) -> WorthQueryApplicationAttemptWorkSnapshot {
        WorthQueryApplicationAttemptWorkSnapshot {
            retained_resolutions: load(&self.retained_resolutions),
            managed_bridge_plans: load(&self.managed_bridge_plans),
            provider_session_readmissions: load(&self.provider_session_readmissions),
            provider_session_preparations: load(&self.provider_session_preparations),
            staged_session_preparations: load(&self.staged_session_preparations),
            attempt_registrations: load(&self.attempt_registrations),
            overlay_stagings: load(&self.overlay_stagings),
            invariant_state_loads: load(&self.invariant_state_loads),
            invariant_executions: load(&self.invariant_executions),
            prepared_commits: load(&self.prepared_commits),
            attempt_aborts: load(&self.attempt_aborts),
            managed_cleanups: load(&self.managed_cleanups),
            external_dispatch_admissions: load(&self.external_dispatch_admissions),
        }
    }
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}
