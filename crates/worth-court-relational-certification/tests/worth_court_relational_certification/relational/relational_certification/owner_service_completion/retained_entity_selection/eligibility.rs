//! Full identity and actual retained schema authority determine eligibility.
use super::occurrences::{fixture, perform, prepare};
use super::*;
use worth_relational::facade::{
    mvcc::{
        RelationalRetainedEntitySelectionDenial as Denial,
        RelationalRetainedEntitySelectionPreparationError as Error,
    },
    schema::{
        FreeFormSchemaDiffIntent, HistoricalInterpretationSensitivity, ProposedSchemaTransition,
        SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind, SchemaElementRef,
        SchemaPublicationImpact, SchemaStratum, SchemaSubscriberImpact, SchemaVersionId,
    },
};

#[test]
fn retired_and_recreated_entities_cannot_borrow_each_others_historical_payload() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let original = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut retirement = begin_main_transaction(&runtime);
    retirement
        .push_batch(
            WorkerIntentBatch::new("retire").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: original,
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let retired = retirement.commit(execution, &runtime).unwrap();
    let recreated = create(execution, &runtime, "model", "3");
    let replacement = changed_entities(&recreated)[0];
    assert_ne!(replacement, original);
    let services = runtime.owner_component_services();
    for (entity, denial) in [
        (original, Denial::IdentityChanged),
        (replacement, Denial::HistoricalSourceUnavailable),
    ] {
        let input = services
            .preparation_port()
            .admit_retained_entity_selection(begin_main_transaction(&runtime), &source, entity)
            .unwrap();
        assert!(
            matches!(services.preparation_port().prepare_retained_entity_selection(execution, input), Err(Error::Admission(actual)) if actual == denial)
        );
    }
    let current = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut prior_savepoint = begin_main_transaction(&runtime);
    let savepoint = prior_savepoint.create_savepoint().unwrap();
    prior_savepoint.rollback_to_savepoint(savepoint).unwrap();
    assert_eq!(
        services
            .preparation_port()
            .admit_retained_entity_selection(prior_savepoint, &current, replacement)
            .unwrap_err(),
        Denial::TargetNotPristine
    );
    let lawful = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &current, replacement),
    );
    assert_eq!(
        values(execution, &runtime, &lawful, [replacement, replacement]),
        ["3", "3"].map(string_aspect_value)
    );
    for commit in [&old, &retired, &recreated, &lawful] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

#[test]
fn historical_schema_is_not_readmitted_as_the_current_schema_after_a_real_transition() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let schema = fixture();
    let runtime = schema.build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let target_schema = AspectSchemaFixture {
        schema_version_id: SchemaVersionId(2),
        ..schema.clone()
    };
    let transition = ProposedSchemaTransition {
        source_schema_id: schema.schema_id.clone(),
        source_schema_version_id: schema.schema_version_id,
        target_schema_id: target_schema.schema_id.clone(),
        target_schema_version_id: target_schema.schema_version_id,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Schema,
                target_schema.schema_id.clone(),
                target_schema.schema_version_id,
                None,
                "schema revision",
            ),
            vec![SchemaStratum::PublicationContract],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail:
                    "reissue the schema authority as revision 2 with unchanged executable contracts"
                        .into(),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    };
    let transaction = runtime
        .begin_branch_schema_transition(&source, transition, None, target_schema.build_registry())
        .unwrap();
    let changed = transaction.commit(execution, &runtime).unwrap();
    let services = runtime.owner_component_services();
    assert_eq!(
        services
            .preparation_port()
            .admit_retained_entity_selection(begin_main_transaction(&runtime), &source, model)
            .unwrap_err(),
        Denial::SchemaChanged
    );
    let current = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let lawful = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &current, model),
    );
    assert_eq!(
        values(execution, &runtime, &lawful, [model, model]),
        ["3", "3"].map(string_aspect_value)
    );
    for commit in [&old, &changed, &lawful] {
        release_test_commit_snapshot(&runtime, commit);
    }
}
