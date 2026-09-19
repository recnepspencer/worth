//! The touched slot must be the record the demand names, not a neighbouring
//! slot that happens to keep the counters plausible.

use super::fixtures::*;
use crate::tests::support::*;

#[test]
fn judgement_tracks_the_nonzero_slot_named_by_the_demand() {
    let runtime = strictness_runtime();
    let mode = create_entity(&runtime, "lax");
    update_entity_and_release_snapshot(&runtime, mode, STRICT_MODE_NAME);
    let bystander_low = create_entity(&runtime, "oversized");
    let target = create_entity(&runtime, COMPLIANT_NAME);
    let _bystander_high = create_entity(&runtime, "oversized");

    assert_ne!(
        target.slot_index(),
        0,
        "the identity court requires a nonzero target slot"
    );

    let mut target_transaction = test_owner_begin_transaction_for_main(&runtime);
    target_transaction
        .push_batch(revalidation_batch("target", [mode, target]))
        .expect("staging stays within configured resource budgets");
    let target_outcome = target_transaction
        .commit(&runtime)
        .expect("the compliant target passes even though both neighbours fail");
    release_test_commit_snapshot(&runtime, &target_outcome);

    let mut bystander_transaction = test_owner_begin_transaction_for_main(&runtime);
    bystander_transaction
        .push_batch(revalidation_batch("bystander", [mode, bystander_low]))
        .expect("staging stays within configured resource budgets");
    let error = bystander_transaction
        .commit(&runtime)
        .expect_err("the oversized bystander must be the record judged");

    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected invariant conflict");
    };
    assert!(
        matches!(
            error.class,
            crate::transactions::data::ConflictClass::InvariantViolation {
                fields: crate::validation::data::InvariantViolationFields::CustomInvariantViolation {
                    ref identity,
                },
                ..
            } if identity.rule_id.as_str() == STRICTNESS_RULE_ID
        ),
        "the negative twin must be rejected by the strictness rule: {:?}",
        error.class
    );
}
