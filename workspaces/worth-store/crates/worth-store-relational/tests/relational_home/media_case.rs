//! One media-sweep case: a freshly prepared home reopened with one media
//! fault, the target run under it, the home abandoned, recovered through the
//! production `open` and held to the crash oracle.

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalPublicationDenial;
use worth_relational::facade::transactions::{CommitResult, TransactionCommitError};
use worth_store::physical_runtime::AnchoredRecordPublicationFailure;
use worth_store_relational::{
    RelationalStoreCheckpointFailure, RelationalStoreCheckpointRecordRefusal, RelationalStoreHome,
};

use super::batches;
use super::crash_oracle::{assert_continues, assert_recovered, InFlight};
use super::durable_run::{apply, open, served_schema};
use super::media_faults::{self, Fault};
use super::prepared_home::PreparedHome;
use super::selected_checkpoint;
use super::workload::{self, Script, Step};

/// The label of the create that probes an unresolved runtime.
const LATER_CREATE: usize = 2_000_000;

/// The step a sweep interrupts, by its index in the script.
#[derive(Clone, Copy)]
pub(crate) enum Target {
    Commit(usize),
    BranchTransition(usize),
    Checkpoint(usize),
}

impl Target {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Commit(index) | Self::BranchTransition(index) | Self::Checkpoint(index) => index,
        }
    }
}

/// How the faulted target ended.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Ended {
    /// The schedule refused the directive for its role; nothing ran.
    Skipped,
    Acknowledged,
    Refused,
    Unresolved {
        recovered: bool,
    },
}

/// Run `target` once on a home holding `prefix`, with `lead` replayed after
/// the faulted open and `fault` armed just before the target.
pub(crate) fn run_case(
    script: &Script,
    prefix: &[Step],
    lead: &[Step],
    target: Target,
    fault: Fault,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) -> Ended {
    let prepared = PreparedHome::prepare(prefix, execution);
    let Some((home, activations)) = media_faults::open_with_faults(
        prepared.location(),
        served_schema(prefix),
        &[fault],
        execution,
    ) else {
        return Ended::Skipped;
    };
    apply(&home, execution, lead);
    let before = workload::artifacts_of(home.runtime());
    let mut acknowledged_transitions = super::branch_lifecycle::artifacts_of_home(&home);
    let mut transition_flight = InFlight::Nothing;
    let checkpoint_before = selected_checkpoint::generation(&home);
    let transition_state_before = matches!(target, Target::BranchTransition(_))
        .then(|| super::branch_lifecycle::state_of(home.runtime(), &script.records));
    media_faults::arm(&activations);
    let (ended, acknowledged, in_flight, checkpoints) = match target {
        Target::Commit(index) => {
            let outcome = script.steps[index].commit(home.runtime(), execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: the fault did not fire"
            );
            let (ended, acknowledged, in_flight) =
                commit_outcome(&home, before, outcome, execution, case);
            (ended, acknowledged, in_flight, vec![checkpoint_before])
        }
        Target::Checkpoint(_) => {
            let outcome = home.checkpoint(ExecutionAllocationPolicy::SystemAllocation);
            assert!(
                activations[0].is_consumed(),
                "{case}: the fault did not fire"
            );
            let (ended, checkpoints) = checkpoint_outcome(outcome, checkpoint_before, case);
            (ended, before, InFlight::Nothing, checkpoints)
        }
        Target::BranchTransition(index) => {
            let outcome = script.steps[index].lifecycle(home.runtime(), execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: transition fault did not fire"
            );
            if outcome.is_err() {
                assert_eq!(
                    Some(super::branch_lifecycle::state_of(
                        home.runtime(),
                        &script.records
                    )),
                    transition_state_before,
                    "{case}: stopped transition changed live state"
                );
            }
            let (ended, acknowledged, flight) = super::transition_oracle::outcome(
                &home,
                &acknowledged_transitions,
                outcome,
                execution,
                case,
            );
            acknowledged_transitions = acknowledged;
            transition_flight = flight;
            (ended, before, InFlight::Nothing, vec![checkpoint_before])
        }
    };
    home.certification_abort();

    let recovered = open(prepared.location(), execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    let selected = selected_checkpoint::generation(&recovered);
    assert!(
        checkpoints.contains(&selected),
        "{case}: checkpoint {selected} selected, expected one of {checkpoints:?}"
    );
    let commits = assert_recovered(&recovered, script, &acknowledged, &in_flight, case);
    let transition_landed = super::transition_oracle::assert_recovered(
        &recovered,
        &acknowledged_transitions,
        &transition_flight,
        case,
    );
    let landed = match target {
        Target::Commit(_) => commits > acknowledged.len(),
        Target::Checkpoint(_) => selected > checkpoint_before,
        Target::BranchTransition(_) => transition_landed,
    };
    assert_continues(
        prepared.location(),
        recovered,
        script,
        commits,
        execution,
        case,
    );
    match ended {
        Ended::Unresolved { .. } => Ended::Unresolved { recovered: landed },
        other => other,
    }
}

/// Classify a faulted commit and require the runtime to agree with it: an
/// acknowledged commit is installed, a refused or unresolved one is not, and
/// an unresolved fate refuses every later commit the same way.
fn commit_outcome(
    home: &RelationalStoreHome,
    before: Vec<Vec<u8>>,
    outcome: Result<CommitResult, TransactionCommitError>,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) -> (Ended, Vec<Vec<u8>>, InFlight) {
    let now = workload::append_live_artifacts(home.runtime(), &before);
    match outcome {
        Ok(_) => {
            assert_eq!(
                now.len(),
                before.len() + 1,
                "{case}: acknowledged, not installed"
            );
            (Ended::Acknowledged, now, InFlight::Nothing)
        }
        Err(TransactionCommitError::PublicationDenied {
            denial: RelationalPublicationDenial::DurablePublicationRefused(_),
            ..
        }) => {
            assert!(now == before, "{case}: a refused commit was installed");
            (Ended::Refused, before, InFlight::Refused)
        }
        Err(error) => {
            let fate = error
                .durability_unresolved_fate()
                .unwrap_or_else(|| panic!("{case}: unexpected commit error {error:?}"))
                .clone();
            assert!(now == before, "{case}: an unresolved commit was installed");
            assert_eq!(
                super::crash_oracle::commit_subject(&fate).0 .0,
                before.len() as u64 + 1,
                "{case}"
            );
            // A later commit that would otherwise publish is refused with the
            // same unresolved fate.
            let later = workload::commit(
                home.runtime(),
                execution,
                &BranchId("main".to_owned()),
                batches::create(LATER_CREATE),
            );
            let later_fate = later
                .as_ref()
                .err()
                .and_then(|error| error.durability_unresolved_fate());
            assert!(
                later_fate == Some(&fate),
                "{case}: a later commit was not refused as unresolved: {:?}",
                later.map(|_| ())
            );
            (
                Ended::Unresolved { recovered: false },
                before,
                InFlight::Unresolved(fate),
            )
        }
    }
}

/// Classify a faulted checkpoint and the generations recovery may select:
/// the new one once its record is published, the old one when the
/// publication was proven to have had no effect, otherwise either.
fn checkpoint_outcome(
    outcome: Result<
        worth_store_relational::RelationalStoreCheckpoint,
        RelationalStoreCheckpointFailure,
    >,
    old: u64,
    case: &str,
) -> (Ended, Vec<u64>) {
    let new = old + 1;
    match outcome {
        Ok(checkpoint) => {
            assert_eq!(checkpoint.generation(), new, "{case}");
            (Ended::Acknowledged, vec![new])
        }
        // The record is selected; only Store's own checkpoint did not finish.
        Err(RelationalStoreCheckpointFailure::PhysicalCheckpoint { .. }) => {
            (Ended::Acknowledged, vec![new])
        }
        Err(RelationalStoreCheckpointFailure::Record(
            RelationalStoreCheckpointRecordRefusal::Publication(
                AnchoredRecordPublicationFailure::ProvenNoEffect(_),
            ),
        )) => (Ended::Refused, vec![old]),
        Err(_) => (Ended::Unresolved { recovered: false }, vec![old, new]),
    }
}
