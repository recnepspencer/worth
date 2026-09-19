//! A revalidation demand is what makes a rule judge a record the candidate did
//! not change.

use super::fixtures::*;
use crate::tests::support::*;
use crate::transactions::data::{ConflictClass, RevalidateEntityIntent};
use crate::validation::data::InvariantViolationFields;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};

#[test]
fn a_revalidation_demand_makes_a_rule_judge_and_reject_an_unchanged_record() {
    let runtime = strictness_runtime();
    let offending = create_entity(&runtime, "oversized");
    let mode = create_entity(&runtime, "lax");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(strict_mode_batch("adopt-strict", mode).push(revalidate(offending)))
        .expect("staging stays within configured resource budgets");
    let error = transaction
        .commit(&runtime)
        .expect_err("the strict reading rejects the unchanged record");

    assert_rejected_by_strictness_rule(&error);
}

#[test]
fn without_the_demand_the_same_candidate_leaves_the_record_unjudged() {
    let runtime = strictness_runtime();
    let offending = create_entity(&runtime, "oversized");
    let mode = create_entity(&runtime, "lax");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(strict_mode_batch("adopt-strict-only", mode))
        .expect("staging stays within configured resource budgets");
    let outcome = transaction
        .commit(&runtime)
        .expect("the offending record is never touched, so no rule judges it");

    release_test_commit_snapshot(&runtime, &outcome);
    assert_eq!(
        committed_name(&runtime, offending),
        Some("oversized".to_owned()),
        "the unjudged record is also unchanged"
    );
}

#[test]
fn a_demand_alone_is_enough_when_the_committed_mode_is_already_strict() {
    let runtime = strictness_runtime();
    let mode = create_entity(&runtime, "lax");
    update_entity_and_release_snapshot(&runtime, mode, STRICT_MODE_NAME);
    let offending = create_entity(&runtime, "oversized");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("revalidate-only", [mode, offending]))
        .expect("staging stays within configured resource budgets");
    let error = transaction
        .commit(&runtime)
        .expect_err("a candidate that changes nothing can still be rejected");

    assert_rejected_by_strictness_rule(&error);
}

#[test]
fn a_passing_demand_leaves_the_record_and_its_aspect_versions_untouched() {
    let runtime = strictness_runtime();
    let compliant = create_entity(&runtime, COMPLIANT_NAME);
    let before_state = committed_name(&runtime, compliant);
    let before_versions = runtime
        .read_truth()
        .entity_aspect_versions(compliant)
        .expect("aspect versions before the demand");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("revalidate-compliant", [compliant]))
        .expect("staging stays within configured resource budgets");
    let outcome = transaction
        .commit(&runtime)
        .expect("the compliant record passes the strict reading");

    assert!(
        changed_entities(&outcome).is_empty(),
        "a demand authors nothing, so it changes no record: {:?}",
        outcome.changed_records
    );
    assert!(
        outcome.patch().is_empty(),
        "a demand authors nothing, so there is no patch to deliver"
    );
    release_test_commit_snapshot(&runtime, &outcome);
    assert_eq!(committed_name(&runtime, compliant), before_state);
    assert_eq!(
        runtime
            .read_truth()
            .entity_aspect_versions(compliant)
            .expect("aspect versions after the demand"),
        before_versions,
        "no aspect version may move for a record nothing was written to"
    );
}

#[test]
fn a_demand_for_a_record_this_branch_does_not_have_is_refused_as_a_stale_target() {
    let runtime = strictness_runtime();
    let present = create_entity(&runtime, COMPLIANT_NAME);
    let absent = crate::facade::identity::EntityId::new(
        present.partition_id,
        present.local_slot_value() + 4_096,
        present.generation_value(),
    );

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("revalidate-absent", [absent]))
        .expect("staging stays within configured resource budgets");
    let error = transaction
        .commit(&runtime)
        .expect_err("a demand names an existing record or it names nothing");

    let TransactionCommitError::Conflict { error, .. } = &error else {
        panic!("expected a commit conflict, got {error:?}");
    };
    assert!(
        matches!(
            &error.class,
            ConflictClass::StaleTarget {
                target: crate::transactions::data::ExistingRecordTarget::Entity(entity_id),
                ..
            } if *entity_id == absent
        ),
        "expected a stale target naming the absent record, got {:?}",
        error.class
    );
}

fn revalidate(entity_id: crate::facade::identity::EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Revalidate(RevalidateEntityIntent {
        entity_id,
    }))
}

fn strict_mode_batch(label: &str, mode: crate::facade::identity::EntityId) -> WorkerIntentBatch {
    WorkerIntentBatch::new(label).push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: mode,
            fields: crate::transactions::data::AspectFieldPatch::from_locator(
                crate::transactions::data::planned_single_field_locator(
                    AspectKey::new("name").expect("valid test aspect key"),
                    FieldKey::new("name").expect("valid test field key"),
                ),
                AspectValue::String(STRICT_MODE_NAME.into()),
            ),
        },
    )))
}

fn assert_rejected_by_strictness_rule(error: &TransactionCommitError) {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected a commit conflict, got {error:?}");
    };
    assert!(
        matches!(
            &error.class,
            ConflictClass::InvariantViolation {
                fields: InvariantViolationFields::CustomInvariantViolation { identity },
                ..
            } if identity.rule_id.as_str() == STRICTNESS_RULE_ID
        ),
        "expected the strictness rule to be named as the rejecting rule, got {:?}",
        error.class
    );
}

fn committed_name(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
) -> Option<String> {
    let current = runtime
        .read_truth()
        .read_version(runtime.current_version_id());
    read_entity_name(current.get_entity(entity_id).expect("record is present"))
}
