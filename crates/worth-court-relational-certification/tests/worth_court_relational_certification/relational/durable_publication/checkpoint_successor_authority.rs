//! Recovery authority advances only through its exact public transition owner.
use super::branch_history_publications::transaction;
use super::branch_history_recovery::current_head;
use std::sync::Arc;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    durability::{
        RecoveredCheckpointTransitionDenial as Denial,
        RecoveredCheckpointTransitionError as TransitionError, RecoveredRelationalRuntimeAuthority,
    },
    history::BranchId,
    mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{batch_create, create_entity, durable_runtime},
};
fn recovered_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    RecoveredRelationalRuntimeAuthority,
    Arc<TestDurableMedium>,
    Arc<worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource>,
){
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    create_entity(execution, &source, "checkpoint-predecessor");
    let (checkpoint, members) =
        worth_testkit_relational::cold_recovery::capture_external_checkpoint(execution, &source);
    let mut target = durable_runtime(&medium);
    let (_, authority) = target
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &checkpoint, members.clone())
        .unwrap();
    (target, authority, medium, members)
}
fn candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> PreparedRelationalCommitCandidate {
    let mut transaction = transaction(runtime, &BranchId("main".into()));
    transaction
        .push_batch(batch_create(name), execution.allocation_policy())
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}
/// C2 snapshot-handle census restored from
/// e4d3b01c8d:crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs.
/// Fixture adaptation: the private visibility and preparation counters are read
/// through the owner's public counted storage census.
fn published_snapshots(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> (usize, usize) {
    let stats = runtime.storage_access().storage_stats(execution).unwrap();
    (
        stats.published_snapshot_handle_count,
        stats.published_snapshot_slot_count,
    )
}
fn main_basis(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}
#[test]
fn acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for admit_successor in [true, false] {
        let (mut runtime, authority, medium, members) = recovered_world(execution);
        let predecessor = main_basis(&runtime);
        let count = runtime.history().immutable_commit_count();
        let prepared = candidate(execution, &runtime, "acknowledged-successor");
        let transition = runtime
            .durability_recovery()
            .commit_checkpoint_transition(execution, authority, prepared)
            .unwrap();
        let expected_receipt = current_head(&runtime, &BranchId("main".into())).unwrap();
        assert_eq!(transition.commit(), &expected_receipt);
        let (receipt, successor_authority) = transition.into_parts();
        assert_eq!(runtime.history().immutable_commit_count(), count + 1);
        assert_eq!(published_snapshots(execution, &runtime), (0, 0));
        assert_eq!(
            medium
                .committed_artifacts()
                .unwrap()
                .iter()
                .filter(|artifact| artifact.commit_id() == receipt.commit_id)
                .count(),
            1
        );
        worth_testkit_relational::cold_recovery::capture_external_carrying(
            execution, &runtime, &members,
        );
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for foreign_recovery in [true, false] {
        let (mut runtime, authority, _medium, _members) = recovered_world(execution);
        let (foreign, foreign_authority, _foreign_medium, _foreign_members) =
            recovered_world(execution);
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
        assert_eq!(published_snapshots(execution, &runtime).0, 0);
        assert_eq!(published_snapshots(execution, &foreign).0, 0);
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for stale_candidate in [true, false] {
        let (mut runtime, authority, _medium, _members) = recovered_world(execution);
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
        assert_eq!(published_snapshots(execution, &runtime).0, 0);
        assert!(refusal.into_authority().admit_basis(predecessor).is_ok());
    }
}

mod durability_refusals;
