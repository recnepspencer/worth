//! One media-sweep case: a freshly prepared home reopened with one media
//! fault, the target run under it, the home abandoned, recovered through the
//! production `open` and held to the crash oracle.

pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::mvcc::RelationalPublicationDenial};
pub(crate) use {worth_relational::facade::transactions::CommitResult, worth_relational::facade::transactions::TransactionCommitError};
pub(crate) use {worth_store::physical_runtime::AnchoredRecordPublicationFailure};
pub(crate) use {worth_store_relational::RelationalStoreCheckpointFailure, worth_store_relational::RelationalStoreCheckpointRecordRefusal, worth_store_relational::RelationalStoreHome};

pub(crate) use {crate::batches};
pub(crate) use {crate::crash_oracle::assert_continues, crate::crash_oracle::assert_recovered, crate::crash_oracle::InFlight};
pub(crate) use {crate::durable_run::apply, crate::durable_run::open};
pub(crate) use {crate::media_faults, crate::media_faults::FaultedMedia};
pub(crate) use {crate::media_sweep::prepared_sweep_home::PreparedSweepHome};
pub(crate) use {crate::selected_checkpoint};
pub(crate) use {crate::workload, crate::workload::Script, crate::workload::Step};

/// The label of the create that probes an unresolved runtime.
pub(crate) const LATER_CREATE: usize = 2_000_000;

/// The step a sweep interrupts, by its index in the script.
#[derive(Clone, Copy)]
pub(crate) enum Target {
    Commit(usize),
    BranchTransition(usize),
    Checkpoint(usize),
    RegionLoad(usize),
}

impl Target {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Commit(index)
            | Self::BranchTransition(index)
            | Self::Checkpoint(index)
            | Self::RegionLoad(index) => index,
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

/// Run `target` once on a restored prefix, with `lead` replayed after
/// the faulted open and the admitted schedule armed just before the target.
pub(crate) fn run_case(
    script: &Script,
    prepared: &PreparedSweepHome,
    lead: &[Step],
    target: Target,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) -> Ended {
    let (home, activations) = media.open(prepared.location(), prepared.schema(), execution);
    apply(&home, execution, lead);
    let before = crate::workload::artifacts_of(home.runtime());
    let mut acknowledged_transitions = crate::branch_lifecycle::artifacts_of_home(&home);
    let mut transition_flight = crate::crash_oracle::InFlight::Nothing;
    let checkpoint_before = crate::selected_checkpoint::generation(&home);
    let transition_state_before = matches!(target, crate::media_case::Target::BranchTransition(_))
        .then(|| crate::branch_lifecycle::state_of(home.runtime(), &script.records));
    if matches!(target, crate::media_case::Target::RegionLoad(_)) {
        crate::residency_media::prepare_cold_media_read(&home);
    }
    crate::media_faults::arm(&activations);
    let (ended, acknowledged, in_flight, checkpoints) = match target {
        crate::media_case::Target::Commit(index) => {
            let outcome = script.steps[index].commit(home.runtime(), execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: the fault did not fire"
            );
            let (ended, acknowledged, in_flight) =
                commit_outcome(&home, before, outcome, execution, case);
            (ended, acknowledged, in_flight, vec![checkpoint_before])
        }
        crate::media_case::Target::Checkpoint(_) => {
            let outcome = home.checkpoint(execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: the fault did not fire"
            );
            let (ended, checkpoints) = checkpoint_outcome(outcome, checkpoint_before, case);
            (ended, before, crate::crash_oracle::InFlight::Nothing, checkpoints)
        }
        crate::media_case::Target::RegionLoad(_) => {
            let outcome = crate::residency_media::read_first(&home, script, execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: region fault did not fire"
            );
            let ended = match outcome {
                Ok(rendered) => {
                    assert_eq!(rendered, crate::residency_media::expected_first(script, before.len()), "{case}: faulted read differs from memory");
                    crate::media_case::Ended::Acknowledged
                },
                Err(worth_relational::facade::mvcc::RelationalTransactionReadDenial::StorageUnavailable(failure)) => {
                    assert!(matches!(failure.subject, worth_relational::facade::storage::RelationalStorageSubject::Region(_)
                        | worth_relational::facade::storage::RelationalStorageSubject::Symbols { .. }), "{case}: {failure:?}");
                    crate::media_case::Ended::Refused
                }
                Err(other) => panic!("{case}: region read returned {other:?}"),
            };
            // One-shot media faults prove no damage to content. A retry must
            // reach the reference, including if another subject failed first.
            let expected = &script.states[before.len()];
            assert_eq!(
                &crate::workload::states_of(home.runtime(), &script.records),
                expected
            );
            (ended, before, crate::crash_oracle::InFlight::Nothing, vec![checkpoint_before])
        }
        crate::media_case::Target::BranchTransition(index) => {
            let outcome = script.steps[index].lifecycle(home.runtime(), execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: transition fault did not fire"
            );
            if outcome.is_err() {
                assert_eq!(
                    Some(crate::branch_lifecycle::state_of(
                        home.runtime(),
                        &script.records
                    )),
                    transition_state_before,
                    "{case}: stopped transition changed live state"
                );
            }
            let (ended, acknowledged, flight) = crate::transition_oracle::outcome(
                &home,
                &acknowledged_transitions,
                outcome,
                execution,
                case,
            );
            acknowledged_transitions = acknowledged;
            transition_flight = flight;
            (ended, before, crate::crash_oracle::InFlight::Nothing, vec![checkpoint_before])
        }
    };
    home.certification_abort();

    let recovered = open(prepared.location(), execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    let selected = crate::selected_checkpoint::generation(&recovered);
    assert!(
        checkpoints.contains(&selected),
        "{case}: checkpoint {selected} selected, expected one of {checkpoints:?}"
    );
    let commits = assert_recovered(&recovered, script, &acknowledged, &in_flight, case);
    let transition_landed = crate::transition_oracle::assert_recovered(
        &recovered,
        &acknowledged_transitions,
        &transition_flight,
        case,
    );
    let landed = match target {
        crate::media_case::Target::Commit(_) => commits > acknowledged.len(),
        crate::media_case::Target::Checkpoint(_) => selected > checkpoint_before,
        crate::media_case::Target::RegionLoad(_) => false,
        crate::media_case::Target::BranchTransition(_) => transition_landed,
    };
    if matches!(target, crate::media_case::Target::RegionLoad(_)) {
        recovered
            .runtime()
            .durability_authority()
            .checkpoint(execution)
            .unwrap();
        crate::durable_run::assert_lifecycle_prefix(&recovered, script, commits, case);
    }
    assert_continues(
        prepared.location(),
        recovered,
        script,
        commits,
        execution,
        case,
    );
    match ended {
        crate::media_case::Ended::Unresolved { .. } => crate::media_case::Ended::Unresolved { recovered: landed },
        other => other,
    }
}

/// Classify a faulted commit and require the runtime to agree with it: an
/// acknowledged commit is installed, a refused or unresolved one is not, and
/// an unresolved fate refuses every later commit the same way.
pub(crate) fn commit_outcome(
    home: &RelationalStoreHome,
    before: Vec<Vec<u8>>,
    outcome: Result<CommitResult, TransactionCommitError>,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) -> (Ended, Vec<Vec<u8>>, InFlight) {
    let now = crate::workload::append_live_artifacts(home.runtime(), &before);
    match outcome {
        Ok(_) => {
            assert_eq!(
                now.len(),
                before.len() + 1,
                "{case}: acknowledged, not installed"
            );
            (crate::media_case::Ended::Acknowledged, now, crate::crash_oracle::InFlight::Nothing)
        }
        Err(worth_relational::facade::transactions::TransactionCommitError::PublicationDenied {
            denial: worth_relational::facade::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_),
            ..
        }) => {
            assert!(now == before, "{case}: a refused commit was installed");
            (crate::media_case::Ended::Refused, before, crate::crash_oracle::InFlight::Refused)
        }
        Err(worth_relational::facade::transactions::TransactionCommitError::Preparation { error, .. })
            if matches!(
                error.reason(),
                worth_relational::facade::transactions::CommitPreparationReason::StorageUnavailable(
                    _
                )
            ) =>
        {
            // A media fault can now refuse a declared cold demand before the
            // durable step. Recovery must prove there was no journal append.
            assert!(
                now == before,
                "{case}: unavailable cold storage published a commit"
            );
            (crate::media_case::Ended::Refused, before, crate::crash_oracle::InFlight::NotDurable)
        }
        Err(error) => {
            let fate = error
                .durability_unresolved_fate()
                .unwrap_or_else(|| panic!("{case}: unexpected commit error {error:?}"))
                .clone();
            assert!(now == before, "{case}: an unresolved commit was installed");
            assert_eq!(
                crate::crash_oracle::commit_subject(&fate).0 .0,
                before.len() as u64 + 1,
                "{case}"
            );
            // A later commit that would otherwise publish is refused with the
            // same unresolved fate.
            let later = crate::workload::commit(
                home.runtime(),
                execution,
                &BranchId("main".to_owned()),
                crate::batches::create(LATER_CREATE),
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
                crate::media_case::Ended::Unresolved { recovered: false },
                before,
                crate::crash_oracle::InFlight::Unresolved(fate),
            )
        }
    }
}

/// Classify a faulted checkpoint and the generations recovery may select:
/// the new one once its record is published, the old one when the
/// publication was proven to have had no effect, otherwise either.
pub(crate) fn checkpoint_outcome(
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
            (crate::media_case::Ended::Acknowledged, vec![new])
        }
        // The record is selected; only Store's own checkpoint did not finish.
        Err(worth_store_relational::RelationalStoreCheckpointFailure::PhysicalCheckpoint { .. }) => {
            (crate::media_case::Ended::Acknowledged, vec![new])
        }
        Err(worth_store_relational::RelationalStoreCheckpointFailure::Record(
            worth_store_relational::RelationalStoreCheckpointRecordRefusal::Publication(
                worth_store::physical_runtime::AnchoredRecordPublicationFailure::ProvenNoEffect(_),
            ),
        )) => (crate::media_case::Ended::Refused, vec![old]),
        Err(_) => (crate::media_case::Ended::Unresolved { recovered: false }, vec![old, new]),
    }
}