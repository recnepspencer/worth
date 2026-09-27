use worth_foundational::facade::AspectValue;

use crate::authorization::{
    RelationalAuthorizationFieldComparison, RelationalAuthorizationFieldConstraint,
    RelationalAuthorizationFieldOperand, RelationalAuthorizationObservationCounters,
    RelationalAuthorizationObservationFreshness, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathPlan, RelationalAuthorizationPredicate,
};
use crate::facade::config::CascadeDeletePolicy;
use crate::identity::data::EntityId;
use crate::tests::support::{
    aspect_field_locator, aspect_field_patch_from_values, aspect_key, create_entity,
    create_relation, entity_field_aspect, entity_u64_field_aspect, field_key, string_aspect_value,
    AspectSchemaFixture,
};
use crate::transactions::data::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::{forward_traversal, ENTITY_KIND};

const REMAINING_ASPECT: &str = "remaining";
const REMAINING_FIELD: &str = "remaining";
const TEXT_BOUND_ASPECT: &str = "text-bound";
const TEXT_BOUND_FIELD: &str = "text-bound";

#[test]
fn strictly_greater_predicate_observes_the_open_numeric_boundary() {
    let runtime = comparison_runtime();
    let scope = create_entity(&runtime, "predicate-scope");
    let cases = [
        ("greater", Some(5), AspectValue::UInt64(4), true),
        ("equal", Some(4), AspectValue::UInt64(4), false),
        ("smaller", Some(3), AspectValue::UInt64(4), false),
        ("missing", None, AspectValue::UInt64(4), false),
        ("incomparable", Some(5), string_aspect_value("4"), false),
    ];

    for (case, observed, expected, should_match) in cases {
        let principal = create_comparison_entity(&runtime, case, observed, None);
        create_relation(
            &runtime,
            principal,
            scope,
            &format!("{case}-predicate-scope"),
        );

        let evidence = observe_predicate(&runtime, principal, scope, expected);

        assert_eq!(
            evidence.paths()[0].matched(),
            should_match,
            "strictly-greater predicate case {case}"
        );
        assert_eq!(
            evidence.counters(),
            predicate_counters(should_match),
            "strictly-greater predicate case {case} must retain bounded work"
        );
    }
}

#[test]
fn strictly_greater_field_constraint_compares_one_complete_witness() {
    let runtime = comparison_runtime();
    let cases = [
        ("greater", Some(5), Some(4), None, true),
        ("equal", Some(4), Some(4), None, false),
        ("smaller", Some(3), Some(4), None, false),
        ("missing", None, Some(4), None, false),
        ("incomparable", Some(5), None, Some("4"), false),
    ];

    for (case, left_number, right_number, right_text, should_match) in cases {
        let principal =
            create_comparison_entity(&runtime, &format!("{case}-left"), left_number, None);
        let scope =
            create_comparison_entity(&runtime, &format!("{case}-right"), right_number, right_text);
        create_relation(
            &runtime,
            principal,
            scope,
            &format!("{case}-field-constraint"),
        );

        let right_field = if right_text.is_some() {
            text_bound_field()
        } else {
            remaining_field()
        };
        let evidence = observe_field_constraint(&runtime, principal, scope, right_field);

        assert_eq!(
            evidence.paths()[0].matched(),
            should_match,
            "strictly-greater field-constraint case {case}"
        );
        assert_eq!(
            evidence.counters(),
            field_constraint_counters(),
            "strictly-greater field-constraint case {case} must retain bounded work"
        );
    }
}

#[test]
fn strictly_greater_observation_stales_when_the_governing_field_reaches_equality() {
    let runtime = comparison_runtime();
    let principal = create_comparison_entity(&runtime, "drifting-principal", Some(5), None);
    let scope = create_entity(&runtime, "drift-scope");
    create_relation(&runtime, principal, scope, "drift-predicate-scope");
    let admitted = observe_predicate(&runtime, principal, scope, AspectValue::UInt64(4));
    assert!(admitted.paths()[0].matched());
    assert_eq!(admitted.counters(), predicate_counters(true));

    write_comparison_fields(&runtime, principal, Some(4), None);
    let current = runtime.visibility_authority().snapshot();

    assert_eq!(
        runtime.compare_authorization_observation(&admitted, current),
        RelationalAuthorizationObservationFreshness::Stale
    );
    let denied = observe_predicate(&runtime, principal, scope, AspectValue::UInt64(4));
    assert!(!denied.paths()[0].matched());
    assert_eq!(denied.counters(), predicate_counters(false));
}

fn comparison_runtime() -> crate::runtime::RelationalRuntime {
    let mut schema = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    schema.entity_aspects.extend([
        entity_u64_field_aspect(aspect_key(REMAINING_ASPECT), field_key(REMAINING_FIELD)),
        entity_field_aspect(aspect_key(TEXT_BOUND_ASPECT), field_key(TEXT_BOUND_FIELD)),
    ]);
    schema.build_runtime()
}

fn create_comparison_entity(
    runtime: &crate::runtime::RelationalRuntime,
    name: &str,
    remaining: Option<u64>,
    text_bound: Option<&str>,
) -> EntityId {
    let entity = create_entity(runtime, name);
    if remaining.is_some() || text_bound.is_some() {
        write_comparison_fields(runtime, entity, remaining, text_bound);
    }
    entity
}

fn write_comparison_fields(
    runtime: &crate::runtime::RelationalRuntime,
    entity: EntityId,
    remaining: Option<u64>,
    text_bound: Option<&str>,
) {
    let mut values = Vec::new();
    if let Some(remaining) = remaining {
        values.push((
            aspect_key(REMAINING_ASPECT),
            field_key(REMAINING_FIELD),
            AspectValue::UInt64(remaining),
        ));
    }
    if let Some(text_bound) = text_bound {
        values.push((
            aspect_key(TEXT_BOUND_ASPECT),
            field_key(TEXT_BOUND_FIELD),
            string_aspect_value(text_bound),
        ));
    }
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("write-comparison-fields").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: aspect_field_patch_from_values(values),
                }),
            )),
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(runtime)
        .expect("comparison fields must be valid for the declared fixture schema");
}

fn observe_predicate(
    runtime: &crate::runtime::RelationalRuntime,
    principal: EntityId,
    scope: EntityId,
    expected: AspectValue,
) -> crate::authorization::RelationalAuthorizationObservationEvidence {
    let predicate = RelationalAuthorizationPredicate::compare(
        0,
        ENTITY_KIND,
        remaining_field(),
        RelationalAuthorizationFieldComparison::StrictlyGreater,
        expected,
    );
    observe_path(
        runtime,
        principal,
        scope,
        RelationalAuthorizationPathPlan::new([forward_traversal()], [predicate]),
    )
}

fn observe_field_constraint(
    runtime: &crate::runtime::RelationalRuntime,
    principal: EntityId,
    scope: EntityId,
    right_field: worth_foundational::facade::AspectFieldLocator,
) -> crate::authorization::RelationalAuthorizationObservationEvidence {
    let constraint = RelationalAuthorizationFieldConstraint::new(
        RelationalAuthorizationFieldOperand::new(0, ENTITY_KIND, remaining_field()),
        RelationalAuthorizationFieldComparison::StrictlyGreater,
        RelationalAuthorizationFieldOperand::new(1, ENTITY_KIND, right_field),
    );
    observe_path(
        runtime,
        principal,
        scope,
        RelationalAuthorizationPathPlan::new([forward_traversal()], [])
            .with_field_constraints([constraint]),
    )
}

fn observe_path(
    runtime: &crate::runtime::RelationalRuntime,
    principal: EntityId,
    scope: EntityId,
    path: RelationalAuthorizationPathPlan,
) -> crate::authorization::RelationalAuthorizationObservationEvidence {
    let plan = RelationalAuthorizationObservationPlan::try_new(
        runtime.visibility_authority().snapshot(),
        principal,
        scope,
        ENTITY_KIND,
        ENTITY_KIND,
        [path],
        [],
    )
    .expect("strictly-greater observation plan must be structurally valid");
    runtime
        .observe_authorization(plan)
        .expect("strictly-greater fixture snapshot must be observable")
}

fn remaining_field() -> worth_foundational::facade::AspectFieldLocator {
    aspect_field_locator(aspect_key(REMAINING_ASPECT), field_key(REMAINING_FIELD))
}

fn text_bound_field() -> worth_foundational::facade::AspectFieldLocator {
    aspect_field_locator(aspect_key(TEXT_BOUND_ASPECT), field_key(TEXT_BOUND_FIELD))
}

fn predicate_counters(matched: bool) -> RelationalAuthorizationObservationCounters {
    RelationalAuthorizationObservationCounters {
        paths_evaluated: 1,
        adjacency_lists_read: usize::from(matched),
        adjacency_edges_inspected: usize::from(matched),
        relation_records_inspected: usize::from(matched),
        entity_records_inspected: if matched { 4 } else { 3 },
        predicate_fields_inspected: 1,
        relation_join_index_lookups: 0,
        relation_join_candidates_inspected: 0,
        maximum_frontier_width: 1,
    }
}

fn field_constraint_counters() -> RelationalAuthorizationObservationCounters {
    RelationalAuthorizationObservationCounters {
        paths_evaluated: 1,
        adjacency_lists_read: 1,
        adjacency_edges_inspected: 1,
        relation_records_inspected: 1,
        entity_records_inspected: 5,
        predicate_fields_inspected: 2,
        relation_join_index_lookups: 0,
        relation_join_candidates_inspected: 0,
        maximum_frontier_width: 1,
    }
}
