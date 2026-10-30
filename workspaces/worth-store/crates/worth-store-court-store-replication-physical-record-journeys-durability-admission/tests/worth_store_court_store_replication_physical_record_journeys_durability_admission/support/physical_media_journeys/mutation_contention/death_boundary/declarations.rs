pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::certification::MediaPauseGate,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
};

pub(crate) use crate::mutation_contention::counter_evidence::campaign_counter_projection;
pub(crate) use crate::physical_media_journeys::media_admission;
pub(crate) use crate::physical_media_journeys::mutation_contention::event;

pub(crate) fn contention_admission() -> (FilesystemMediaAdmission, Option<MediaPauseGate>) {
    let admission = media_admission();
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(Vec::new())
        .unwrap()
        .pause_before_lease_release(gate.clone());
    (admission.with_fault_schedule(schedule), Some(gate))
}

pub(crate) fn reach_death_boundary(
    media: MediaOwnedPhysicalRuntime,
    gate: Option<MediaPauseGate>,
) -> ! {
    let gate = gate.expect("certification gate");
    let observer = gate.clone();
    let media_observer = media.observer();
    std::thread::spawn(move || {
        observer.wait_until_reached();
        let context = observer
            .reached_context()
            .expect("release boundary must carry operation context");
        let counters = media_observer.media_counters();
        event(&format!(
            "death-boundary;role={};ordinal={};operation={};handle={};release_attempts={};release_completed={};ownership_releases={};stable_counters={}",
            context.role().metric_name(),
            context.role_ordinal(),
            context
                .operation()
                .map_or_else(|| "none".into(), |value| value.value().to_string()),
            context
                .handle()
                .map_or_else(|| "none".into(), |value| value.generation().to_string()),
            counters.attempts_for(worth_store_physical_backend::MediaOperationRole::ReleaseMutationLease),
            counters.completed_operations_for(worth_store_physical_backend::MediaOperationRole::ReleaseMutationLease),
            counters.ownership_releases(),
            campaign_counter_projection(counters),
        ));
    });
    let _blocked = media.close();
    panic!("lease release unexpectedly crossed an unreleased gate")
}
