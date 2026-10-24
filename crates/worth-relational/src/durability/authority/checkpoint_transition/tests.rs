use crate::facade::durability::{
    RecoveredCheckpointTransitionDenial as Denial,
    RecoveredCheckpointTransitionError as TransitionError, RecoveredRelationalRuntimeAuthority,
};
use crate::facade::history::BranchId;
use crate::mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

fn recovered_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalRuntime, RecoveredRelationalRuntimeAuthority) {
    let source = durable_runtime_with_test_schema();
    create_entity(execution, &source, "checkpoint-predecessor");
    let checkpoint = source
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    // The restored runtime continues the source's medium, which already holds
    // the history its checkpoint covers.
    let mut target = durable_runtime_with_test_schema();
    bind_durable_medium(&mut target, attached_test_medium(&source).attachment());
    let (_, authority) = target
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &checkpoint)
        .unwrap();
    (target, authority)
}

fn candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}

fn main_basis(runtime: &RelationalRuntime) -> crate::branch::AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}

#[test]
fn acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for admit_successor in [true, false] {
        let (mut runtime, authority) = recovered_world(execution);
        let predecessor = main_basis(&runtime);
        let count = runtime.history().immutable_commit_count();
        let prepared = candidate(execution, &runtime, "acknowledged-successor");
        let transition = runtime
            .durability_recovery()
            .commit_checkpoint_transition(execution, authority, prepared)
            .unwrap();
        let expected_receipt = runtime
            .history()
            .branch_head(&BranchId("main".into()))
            .unwrap();
        assert_eq!(transition.commit(), &expected_receipt);
        let (receipt, successor_authority) = transition.into_parts();
        assert_eq!(runtime.history().immutable_commit_count(), count + 1);
        assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
        assert_eq!(
            runtime
                .preparation_runtime_snapshot()
                .published_snapshot_count(),
            0
        );
        assert_eq!(
            runtime
                .durability()
                .durable_log()
                .iter()
                .filter(|entry| entry.envelope().commit.commit_id == receipt.commit_id)
                .count(),
            1
        );
        runtime
            .durability_authority()
            .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        if admit_successor {
            assert!(successor_authority
                .admit_basis(main_basis(&runtime))
                .is_ok());
        } else {
            assert!(successor_authority.admit_basis(predecessor).is_err());
        }
    }
}

#[test]
fn foreign_recovery_and_foreign_candidate_are_refused_before_effects() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for foreign_recovery in [true, false] {
        let (mut runtime, authority) = recovered_world(execution);
        let (foreign, foreign_authority) = recovered_world(execution);
        let before = main_basis(&runtime).descriptor().clone();
        let before_foreign = main_basis(&foreign).descriptor().clone();
        let count = runtime.history().immutable_commit_count();
        let foreign_count = foreign.history().immutable_commit_count();
        let (supplied_authority, prepared) = if foreign_recovery {
            (
                foreign_authority,
                candidate(execution, &runtime, "foreign-authority"),
            )
        } else {
            (
                authority,
                candidate(execution, &foreign, "foreign-candidate"),
            )
        };
        let TransitionError::Refused(refusal) = runtime
            .durability_recovery()
            .commit_checkpoint_transition(execution, supplied_authority, prepared)
            .unwrap_err()
        else {
            panic!("foreign ownership must refuse before publication")
        };
        assert_eq!(main_basis(&runtime).descriptor(), &before);
        assert_eq!(main_basis(&foreign).descriptor(), &before_foreign);
        assert_eq!(runtime.history().immutable_commit_count(), count);
        assert_eq!(foreign.history().immutable_commit_count(), foreign_count);
        assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
        assert_eq!(foreign.visibility.published_snapshot_handle_count(), 0);
        if foreign_recovery {
            assert!(matches!(
                refusal.denial(),
                Denial::ForeignRecoveryRuntime { .. }
            ));
            assert!(refusal
                .into_authority()
                .admit_basis(main_basis(&foreign))
                .is_ok());
        } else {
            assert!(matches!(
                refusal.denial(),
                Denial::ForeignCandidateRuntime { .. }
            ));
            let retry = candidate(execution, &runtime, "retry-after-foreign-candidate");
            let acknowledged = runtime
                .durability_recovery()
                .commit_checkpoint_transition(execution, refusal.into_authority(), retry)
                .unwrap();
            assert!(acknowledged
                .into_parts()
                .1
                .admit_basis(main_basis(&runtime))
                .is_ok());
        }
    }
}

#[test]
fn moved_source_and_stale_candidate_cannot_extend_recovery_lineage() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for stale_candidate in [true, false] {
        let (mut runtime, authority) = recovered_world(execution);
        let predecessor = main_basis(&runtime);
        let early =
            stale_candidate.then(|| candidate(execution, &runtime, "prepared-before-movement"));
        create_entity(execution, &runtime, "ordinary-movement");
        let prepared =
            early.unwrap_or_else(|| candidate(execution, &runtime, "prepared-after-movement"));
        let before = main_basis(&runtime).descriptor().clone();
        let count = runtime.history().immutable_commit_count();
        let TransitionError::Refused(refusal) = runtime
            .durability_recovery()
            .commit_checkpoint_transition(execution, authority, prepared)
            .unwrap_err()
        else {
            panic!("movement must refuse without performing another commit")
        };
        if stale_candidate {
            assert!(matches!(refusal.denial(), Denial::Publication(outcome)
                if matches!(outcome.as_ref(), RelationalPublicationOutcome::Stale(_))));
        } else {
            assert!(matches!(refusal.denial(), Denial::SourceNotRecovered(_)));
        }
        assert_eq!(main_basis(&runtime).descriptor(), &before);
        assert_eq!(runtime.history().immutable_commit_count(), count);
        assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
        assert!(refusal.into_authority().admit_basis(predecessor).is_ok());
    }
}

#[test]
fn refused_durable_transition_returns_its_authority_and_moves_nothing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut runtime, authority) = recovered_world(execution);
    let predecessor = main_basis(&runtime);
    let count = runtime.history().immutable_commit_count();
    let medium = attached_test_medium(&runtime);
    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    let prepared = candidate(execution, &runtime, "refused-transition");
    let TransitionError::Refused(refusal) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, prepared)
        .unwrap_err()
    else {
        panic!("a refused durable publication is a pre-effect transition refusal")
    };
    assert!(matches!(refusal.denial(), Denial::Publication(outcome)
    if matches!(outcome.as_ref(), RelationalPublicationOutcome::Denied(
        crate::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_)
    ))));
    assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor());
    assert_eq!(runtime.history().immutable_commit_count(), count);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
    assert_eq!(medium.head(), PatchStreamPosition(1));

    let authority = refusal.into_authority();
    let retried = candidate(execution, &runtime, "retried-transition");
    let transition = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, retried)
        .expect("the refused position was never consumed");
    assert_eq!(medium.head(), PatchStreamPosition(2));
    assert_eq!(runtime.history().immutable_commit_count(), count + 1);
    assert!(transition
        .into_parts()
        .1
        .admit_basis(main_basis(&runtime))
        .is_ok());
}

#[test]
fn unresolved_durable_transition_installs_nothing_and_names_its_fate() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut runtime, authority) = recovered_world(execution);
    let predecessor = main_basis(&runtime);
    let count = runtime.history().immutable_commit_count();
    let medium = attached_test_medium(&runtime);
    medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);
    let prepared = candidate(execution, &runtime, "unresolved-transition");
    let TransitionError::Refused(refusal) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, prepared)
        .unwrap_err()
    else {
        panic!("an unresolved durable publication installs no successor")
    };
    let Denial::Publication(outcome) = refusal.denial() else {
        panic!("the refusal names the publication outcome")
    };
    let RelationalPublicationOutcome::DurabilityUnresolved(fate) = outcome.as_ref() else {
        panic!("the medium could not say whether the transition is durable")
    };
    let offered = medium.offered().pop().expect("the medium was asked first");
    assert_eq!(fate.position(), offered.position());
    assert_eq!(fate.digest(), offered.payload_digest());
    assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor());
    assert_eq!(runtime.history().immutable_commit_count(), count);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
}
