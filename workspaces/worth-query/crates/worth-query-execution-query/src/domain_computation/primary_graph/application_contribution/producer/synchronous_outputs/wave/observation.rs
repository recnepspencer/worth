//! Passive observation of wave scheduling decisions on one thread (#254).
//! Recording never changes control flow; it runs only under an installed
//! `WorthQueryExecutionObservation` and clears with it.

/// Adds work the wave charged, only while observed.
pub(super) fn charged(units: u64) {
    crate::domain_computation::execution_observation::record_wave_charged(units);
}

/// Records one event, formatted only while observed.
pub(super) fn record(event: impl FnOnce() -> String) {
    crate::domain_computation::execution_observation::record_wave_event(event);
}

/// Records one task prepare outcome, formatted only while observed.
pub(super) fn record_prepare<S: super::ApplicationSchema>(
    i: usize,
    task: &super::Scheduled<S>,
    prepared: &Result<super::PlannedOutput, super::SynchronousOutputFailure>,
) {
    record(|| {
        let outcome = match prepared {
            Ok(super::PlannedOutput::Prepared(parent)) => {
                if parent.is_some() {
                    "prepared"
                } else {
                    "prepared-none"
                }
            }
            Ok(super::PlannedOutput::Recapture) => "recapture",
            Err(_) => "refused",
        };
        format!(
            "prepare task={i} subject={} family={} source={:?} {outcome}",
            task.subject,
            task.tree.family(),
            task.key.as_ref().map(|key| key.scope),
        )
    });
}
