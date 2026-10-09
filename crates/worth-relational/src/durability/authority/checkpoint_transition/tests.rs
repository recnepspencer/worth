use crate::facade::durability::{
    RecoveredCheckpointTransitionDenial as Denial,
    RecoveredCheckpointTransitionError as TransitionError, RecoveredRelationalRuntimeAuthority,
};
use crate::facade::history::BranchId;
use crate::mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome};
use crate::publication::data::DeferredPublicationSettlementError;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

fn recovered_world() -> (RelationalRuntime, RecoveredRelationalRuntimeAuthority) {
    let source = persisted_runtime_with_test_schema();
    create_entity(&source, "checkpoint-predecessor");
    let checkpoint = source
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let mut target = persisted_runtime_with_test_schema();
    let (_, authority) = target
        .durability_recovery()
        .restore_native_checkpoint_with_authority(&checkpoint)
        .unwrap();
    (target, authority)
}

fn candidate(runtime: &RelationalRuntime, name: &str) -> PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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
    for admit_successor in [true, false] {
        let (mut runtime, authority) = recovered_world();
        let predecessor = main_basis(&runtime);
        let count = runtime.history().immutable_commit_count();
        let prepared = candidate(&runtime, "acknowledged-successor");
        let transition = runtime
            .durability_recovery()
            .commit_checkpoint_transition(authority, prepared)
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
    for foreign_recovery in [true, false] {
        let (mut runtime, authority) = recovered_world();
        let (foreign, foreign_authority) = recovered_world();
        let before = main_basis(&runtime).descriptor().clone();
        let before_foreign = main_basis(&foreign).descriptor().clone();
        let count = runtime.history().immutable_commit_count();
        let foreign_count = foreign.history().immutable_commit_count();
        let (supplied_authority, prepared) = if foreign_recovery {
            (foreign_authority, candidate(&runtime, "foreign-authority"))
        } else {
            (authority, candidate(&foreign, "foreign-candidate"))
        };
        let TransitionError::Refused(refusal) = runtime
            .durability_recovery()
            .commit_checkpoint_transition(supplied_authority, prepared)
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
            let retry = candidate(&runtime, "retry-after-foreign-candidate");
            let acknowledged = runtime
                .durability_recovery()
                .commit_checkpoint_transition(refusal.into_authority(), retry)
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
    for stale_candidate in [true, false] {
        let (mut runtime, authority) = recovered_world();
        let predecessor = main_basis(&runtime);
        let early = stale_candidate.then(|| candidate(&runtime, "prepared-before-movement"));
        create_entity(&runtime, "ordinary-movement");
        let prepared = early.unwrap_or_else(|| candidate(&runtime, "prepared-after-movement"));
        let before = main_basis(&runtime).descriptor().clone();
        let count = runtime.history().immutable_commit_count();
        let TransitionError::Refused(refusal) = runtime
            .durability_recovery()
            .commit_checkpoint_transition(authority, prepared)
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
fn deferred_transition_retains_exact_repair_custody_across_foreign_refusal() {
    let (mut runtime, authority) = recovered_world();
    let prepared = candidate(&runtime, "deferred-transition");
    runtime.durability.arm_append_failure();
    let TransitionError::DurabilityDeferred(pending) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(authority, prepared)
        .unwrap_err()
    else {
        panic!("the performed transition must retain its deferred native settlement")
    };
    let receipt = pending
        .cause()
        .deferred_settlement()
        .unwrap()
        .commit()
        .clone();
    let successor = main_basis(&runtime).descriptor().clone();
    let count = runtime.history().immutable_commit_count();
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 1);
    let denial = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    assert!(
        matches!(denial, crate::durability::data::RelationalNativeCheckpointCaptureDenial::Durability(ref error) if error.class == crate::durability::data::RecoveryFailureClass::PerformedPublicationRequiresSettlement)
    );
    let mut foreign = persisted_runtime_with_test_schema();
    let refusal = foreign
        .durability_recovery()
        .repair_checkpoint_transition(*pending)
        .unwrap_err();
    assert!(matches!(
        refusal.cause(),
        DeferredPublicationSettlementError::ForeignRuntime { .. }
    ));
    assert_eq!(foreign.history().immutable_commit_count(), 0);
    let acknowledged = runtime
        .durability_recovery()
        .repair_checkpoint_transition(refusal.into_transition())
        .unwrap();
    assert_eq!(acknowledged.commit(), &receipt);
    assert_eq!(main_basis(&runtime).descriptor(), &successor);
    assert_eq!(runtime.history().immutable_commit_count(), count);
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
    assert!(acknowledged
        .into_parts()
        .1
        .admit_basis(main_basis(&runtime))
        .is_ok());
}

#[test]
fn repair_carries_performed_successor_even_after_owner_repair_and_later_movement() {
    let (mut runtime, authority) = recovered_world();
    let prepared = candidate(&runtime, "captured-successor");
    runtime.durability.arm_append_failure();
    let TransitionError::DurabilityDeferred(pending) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(authority, prepared)
        .unwrap_err()
    else {
        panic!("append fault must retain a performed transition")
    };
    let performed_basis = main_basis(&runtime);
    let settlement = pending.cause().deferred_settlement().unwrap().clone();
    let receipt = runtime
        .repair_deferred_publication_settlement(&settlement)
        .unwrap();
    create_entity(&runtime, "later-ordinary-movement");
    let current = main_basis(&runtime);
    assert_ne!(current.descriptor(), performed_basis.descriptor());
    let count = runtime.history().immutable_commit_count();
    let acknowledged = runtime
        .durability_recovery()
        .repair_checkpoint_transition(*pending)
        .unwrap();
    assert_eq!(acknowledged.commit(), &receipt);
    assert_eq!(runtime.history().immutable_commit_count(), count);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
    // The retained route is acknowledged again, but ambient current state
    // cannot silently receive the captured transition's recovery authority.
    let (_, successor_authority) = acknowledged.into_parts();
    assert!(successor_authority.admit_basis(current).is_err());
}
