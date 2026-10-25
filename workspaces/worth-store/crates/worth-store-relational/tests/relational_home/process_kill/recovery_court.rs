//! Reopened-home evidence after the parent has killed its parked child.
use super::super::{
    crash_oracle::{assert_continues, assert_recovered, InFlight},
    durable_run::open,
    selected_checkpoint,
    workload::{self, Step},
};
use super::{launch, script, Stop, SEED};
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;

pub(super) fn assert_killed_home_recovers(stop: Stop) {
    let script = script();
    let killed = launch(stop);
    let case = format!("seed {SEED:#x} {stop:?}");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(&killed.location, &execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    let held = killed.acknowledged.len();
    let (in_flight, checkpoint) = match stop {
        Stop::BeforeFirstCheckpoint => (InFlight::Nothing, 0),
        Stop::AboveCheckpoint => (InFlight::Nothing, 1),
        Stop::Commit(Seam::BeforeWalAppend) => (InFlight::NotDurable, 2),
        Stop::Commit(_) => (InFlight::Durable, 2),
        Stop::CheckpointMembers(_) => (InFlight::Nothing, 1),
        Stop::Checkpoint(Seam::BeforeWalAppend) => (InFlight::Nothing, 1),
        Stop::Checkpoint(_) => (InFlight::Nothing, 2),
        Stop::BranchTransition(_)
        | Stop::Archive(_)
        | Stop::RequestDeletion(_)
        | Stop::Delete(_) => {
            let (prefix, _) = stop.plan(&script);
            let checkpoints = script.steps[..prefix]
                .iter()
                .filter(|step| matches!(step, Step::Checkpoint))
                .count() as u64;
            (InFlight::Nothing, checkpoints)
        }
    };
    assert_eq!(killed.live, held, "{case}: the parked commit was live");
    assert_eq!(
        selected_checkpoint::generation(&home),
        checkpoint,
        "{case}: selected checkpoint"
    );
    let commits = assert_recovered(&home, &script, &killed.acknowledged, &in_flight, &case);
    let transition_flight = match stop {
        Stop::BranchTransition(Seam::BeforeWalAppend)
        | Stop::Archive(Seam::BeforeWalAppend)
        | Stop::RequestDeletion(Seam::BeforeWalAppend)
        | Stop::Delete(Seam::BeforeWalAppend) => InFlight::NotDurable,
        Stop::BranchTransition(_)
        | Stop::Archive(_)
        | Stop::RequestDeletion(_)
        | Stop::Delete(_) => InFlight::Durable,
        _ => InFlight::Nothing,
    };
    assert_eq!(
        killed.transition_live,
        killed.transitions.len(),
        "{case}: transition live before durable handoff"
    );
    super::super::transition_oracle::assert_recovered(
        &home,
        &killed.transitions,
        &transition_flight,
        &case,
    );
    assert_continues(&killed.location, home, &script, commits, &execution, &case);
}
