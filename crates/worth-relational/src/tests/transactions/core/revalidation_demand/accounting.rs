//! A revalidation demand widens judgement without claiming or paying for
//! mutations it never made.

use super::fixtures::*;
use crate::tests::support::*;
use crate::transactions::data::CommitTopology;
use crate::validation::data::InvariantGroupSet;

#[test]
fn a_demand_reports_no_invalidation_and_keeps_flat_entity_topology() {
    let runtime = strictness_runtime();
    let target = create_entity(&runtime, COMPLIANT_NAME);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("account-demand", [target]))
        .expect("staging stays within configured resource budgets");
    let outcome = transaction
        .commit(&runtime)
        .expect("the compliant record remains valid");

    assert_eq!(
        outcome.structural_summary().invariant_groups,
        InvariantGroupSet::empty(),
        "rejudgement selects rules but must not publicly claim invalidation"
    );
    assert_eq!(
        outcome.structural_summary().commit_topology,
        CommitTopology::FlatEntityBatch,
        "an unchanged-record demand is not a graph mutation"
    );
    release_test_commit_snapshot(&runtime, &outcome);
}

#[test]
fn demand_clone_cost_does_not_scale_with_unrelated_partition_population() {
    let sparse = demand_clone_cost(4);
    let populated = demand_clone_cost(40);

    assert_eq!(
        populated, sparse,
        "one demanded record must clone the same narrow working set regardless of bystander count"
    );
}

#[test]
fn demand_only_rejection_survives_the_narrow_effect_claim() {
    let runtime = strictness_runtime();
    let mode = create_entity(&runtime, "lax");
    update_entity_and_release_snapshot(&runtime, mode, STRICT_MODE_NAME);
    let offending = create_entity(&runtime, "oversized");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("strict-demand", [mode, offending]))
        .expect("staging stays within configured resource budgets");
    let error = transaction
        .commit(&runtime)
        .expect_err("selection must still run the strictness rule");

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
        "the demand must select the strictness rule despite claiming no invalidation: {:?}",
        error.class
    );
}

fn demand_clone_cost(bystanders: usize) -> (usize, usize) {
    let runtime = strictness_runtime();
    let target = create_entity(&runtime, COMPLIANT_NAME);
    for ordinal in 0..bystanders {
        let _ = create_entity(&runtime, &format!("bystander-{ordinal}"));
    }

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("bounded-clone", [target]))
        .expect("staging stays within configured resource budgets");
    let outcome = transaction
        .commit(&runtime)
        .expect("the compliant record remains valid");
    let cost = (
        outcome.complexity_delta().partitions_cloned,
        outcome.complexity_delta().entity_slots_cloned,
    );
    release_test_commit_snapshot(&runtime, &outcome);
    cost
}
