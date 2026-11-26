//! What a proposal's name preview settles to: a basis that moved never
//! previews another commit's names and refuses retryably; a refused plan
//! refuses with its exact denial; a cancellation stays the step's own.
use super::{naming_basis_current, settle_step, NamePreview};
use worth_relational_contracts::internal_port as contracts;

use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::record_ids::EntityId;
use contracts::lineage::data::{LineageNameDenial, LineageRecordTarget};
use contracts::runtime::operation_control::interruption_event::{
    RelationalInterruptionBoundary, RelationalInterruptionEvent,
};
use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};

fn missing_birth() -> LineageNameDenial {
    LineageNameDenial::MissingBirth {
        target: LineageRecordTarget::Entity(EntityId::new(
            contracts::identity::data::primitives::PartitionId::main(),
            1,
            1,
        )),
        scheme_id: "scheme".to_owned(),
    }
}

fn stale() -> NamePreview {
    NamePreview::Stale {
        basis: Some(CommitId(4)),
        latest: CommitId(5),
    }
}

fn is_stale_conflict(error: &TransactionCommitError) -> bool {
    matches!(
        error,
        TransactionCommitError::Conflict { error, .. }
            if matches!(error.class, ConflictClass::StaleValidationBasis { .. })
    )
}

#[test]
fn a_naming_basis_is_current_only_up_to_the_proposals_basis() {
    let (four, five) = (Some(CommitId(4)), Some(CommitId(5)));
    assert!(naming_basis_current(None, None), "no names, empty branch");
    assert!(naming_basis_current(None, four), "no names yet");
    assert!(naming_basis_current(four, four), "the basis's own names");
    assert!(naming_basis_current(Some(CommitId(3)), four), "older names");
    assert!(!naming_basis_current(five, four), "a later commit's names");
    assert!(
        !naming_basis_current(four, None),
        "names on a branch read as empty"
    );
}

#[test]
fn a_stale_basis_refuses_retryably_and_never_as_a_naming_denial() {
    let refused = settle_step(Some(&stale()), Ok::<_, TransactionCommitError>(1))
        .expect_err("a stale basis refuses");
    assert!(is_stale_conflict(&refused), "{refused:?}");
    assert_eq!(refused.lineage_name_denial_ref(), None);
    // Whatever the step met, a stale preview settles to the stale conflict.
    let step = Err::<(), _>(TransactionCommitError::lineage_name_denial(missing_birth()));
    let refused = settle_step(Some(&stale()), step).expect_err("still stale");
    assert!(is_stale_conflict(&refused), "{refused:?}");
}

#[test]
fn a_refused_plan_wins_over_the_verdict_but_not_over_a_cancellation() {
    let refused = NamePreview::Refused(missing_birth());
    let settled = settle_step(Some(&refused), Ok::<_, TransactionCommitError>(1));
    assert_eq!(
        settled
            .expect_err("the plan refuses")
            .lineage_name_denial_ref(),
        Some(&missing_birth())
    );
    let cancelled = RelationalInterruptionEvent::from_request_stop(
        worth_execution::MapKernelStop::Cancelled,
        RelationalInterruptionBoundary::ProposalValidation,
    )
    .expect("a cancellation is an interruption");
    let step = Err::<(), _>(TransactionCommitError::interrupted(cancelled));
    for preview in [refused, stale()] {
        let settled = settle_step(Some(&preview), step.clone());
        assert!(
            matches!(settled, Err(TransactionCommitError::Interrupted { .. })),
            "{settled:?}"
        );
    }
    assert_eq!(
        settle_step(None, Ok::<_, TransactionCommitError>(7)).ok(),
        Some(7)
    );
}
