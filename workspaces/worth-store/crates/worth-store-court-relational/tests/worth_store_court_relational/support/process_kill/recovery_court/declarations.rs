//! Reopened-home evidence after the parent has killed its parked child.
pub(crate) use {crate::crash_oracle::assert_continues, crate::crash_oracle::assert_recovered, crate::crash_oracle::InFlight, crate::durable_run::open, crate::selected_checkpoint, crate::workload, crate::workload::Step};
pub(crate) use {crate::process_kill::launch, crate::process_kill::script, crate::process_kill::Stop, crate::process_kill::SEED};
pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam};

pub(crate) fn assert_killed_home_recovers(stop: Stop) {
    let script = script();
    let killed = launch(stop);
    let case = format!("seed {SEED:#x} {stop:?}");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = open(&killed.location, &execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    let held = killed.acknowledged.len();
    let (in_flight, checkpoint) = match stop {
        crate::process_kill::Stop::BeforeFirstCheckpoint => (crate::crash_oracle::InFlight::Nothing, 0),
        crate::process_kill::Stop::AboveCheckpoint => (crate::crash_oracle::InFlight::Nothing, 1),
        crate::process_kill::Stop::Commit(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend) => (crate::crash_oracle::InFlight::NotDurable, 2),
        crate::process_kill::Stop::Commit(_) => (crate::crash_oracle::InFlight::Durable, 2),
        crate::process_kill::Stop::CheckpointReturned => (crate::crash_oracle::InFlight::Nothing, 2),
        crate::process_kill::Stop::CheckpointAcknowledged(_) => (crate::crash_oracle::InFlight::Nothing, 2),
        crate::process_kill::Stop::CheckpointCarried(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend) => (crate::crash_oracle::InFlight::Nothing, 2),
        crate::process_kill::Stop::CheckpointCarried(_) => (crate::crash_oracle::InFlight::Nothing, 3),
        crate::process_kill::Stop::CheckpointMembers(_) => (crate::crash_oracle::InFlight::Nothing, 1),
        crate::process_kill::Stop::Checkpoint(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend) => (crate::crash_oracle::InFlight::Nothing, 1),
        crate::process_kill::Stop::Checkpoint(_) => (crate::crash_oracle::InFlight::Nothing, 2),
        crate::process_kill::Stop::BranchTransition(_)
        | crate::process_kill::Stop::Archive(_)
        | crate::process_kill::Stop::RequestDeletion(_)
        | crate::process_kill::Stop::Delete(_) => {
            let (prefix, _) = stop.plan(&script);
            let checkpoints = script.steps[..prefix]
                .iter()
                .filter(|step| matches!(step, crate::workload::Step::Checkpoint))
                .count() as u64;
            (crate::crash_oracle::InFlight::Nothing, checkpoints)
        }
    };
    assert_eq!(killed.live, held, "{case}: the parked commit was live");
    assert_eq!(
        crate::selected_checkpoint::generation(&home),
        checkpoint,
        "{case}: selected checkpoint"
    );
    if matches!(
        stop,
        crate::process_kill::Stop::CheckpointCarried(_) | crate::process_kill::Stop::CheckpointAcknowledged(_) | crate::process_kill::Stop::CheckpointReturned
    ) {
        assert_eq!(
            home.cold_storage_reads().member_reads,
            0,
            "{case}: checkpoint reopen loads only the manifest"
        );
    }
    let commits = assert_recovered(&home, &script, &killed.acknowledged, &in_flight, &case);
    let transition_flight = match stop {
        crate::process_kill::Stop::BranchTransition(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend)
        | crate::process_kill::Stop::Archive(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend)
        | crate::process_kill::Stop::RequestDeletion(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend)
        | crate::process_kill::Stop::Delete(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend) => crate::crash_oracle::InFlight::NotDurable,
        crate::process_kill::Stop::BranchTransition(_)
        | crate::process_kill::Stop::Archive(_)
        | crate::process_kill::Stop::RequestDeletion(_)
        | crate::process_kill::Stop::Delete(_) => crate::crash_oracle::InFlight::Durable,
        _ => crate::crash_oracle::InFlight::Nothing,
    };
    assert_eq!(
        killed.transition_live,
        killed.transitions.len(),
        "{case}: transition live before durable handoff"
    );
    crate::transition_oracle::assert_recovered(
        &home,
        &killed.transitions,
        &transition_flight,
        &case,
    );
    assert_continues(&killed.location, home, &script, commits, &execution, &case);
}