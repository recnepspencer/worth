//! Original currentness and installed invariants still govern historical payloads.
use super::occurrences::{fixture, perform, prepare};
use super::*;
use worth_relational::facade::{
    mvcc::{
        RelationalRetainedEntitySelectionDenial as Denial,
        RelationalRetainedEntitySelectionPreparationError as Error,
    },
    runtime::{InvariantCatalog, InvariantRegistration, InvariantRule, RelationalRuntimeApi},
};

#[test]
fn stale_foreign_and_staged_inputs_refuse_before_selection_publication() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let stale = runtime
        .begin_branch_transaction(&source, RelationalTransactionIntent::ordinary())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5")]);
    let services = runtime.owner_component_services();
    assert_eq!(
        services
            .preparation_port()
            .admit_retained_entity_selection(stale, &source, model)
            .unwrap_err(),
        Denial::TargetNotCurrent
    );
    let mut staged = begin_main_transaction(&runtime);
    staged
        .push_batch(
            WorkerIntentBatch::new("mixed-input").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: model,
                    fields: string_aspect_field_patch([(
                        aspect_key("name"),
                        field_key("label"),
                        "9",
                    )]),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(
        services
            .preparation_port()
            .admit_retained_entity_selection(staged, &source, model)
            .unwrap_err(),
        Denial::TargetNotPristine
    );
    let foreign = fixture().build_runtime();
    let foreign_commit = create(execution, &foreign, "model", "3");
    let foreign_source = foreign
        .admit_branch_basis(&foreign.main_branch_identity())
        .unwrap();
    assert_eq!(
        services
            .preparation_port()
            .admit_retained_entity_selection(
                begin_main_transaction(&runtime),
                &foreign_source,
                model
            )
            .unwrap_err(),
        Denial::ForeignRuntime
    );
    assert_eq!(
        values(execution, &runtime, &current, [model, model]),
        ["5", "5"].map(string_aspect_value)
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id()
            .unwrap(),
        current.commit.commit_id
    );
    let selected = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &source, model),
    );
    assert_eq!(
        values(execution, &runtime, &selected, [model, model]),
        ["3", "3"].map(string_aspect_value)
    );
    for commit in [&old, &current, &selected] {
        release_test_commit_snapshot(&runtime, commit);
    }
    release_test_commit_snapshot(&foreign, &foreign_commit);
}

#[test]
fn selection_rejudges_current_installed_uniqueness_instead_of_treating_commands_as_empty_scope() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut catalog = InvariantCatalog::default();
    catalog
        .registrations
        .push(InvariantRegistration::mutation_sensitive_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("label")),
        ));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(fixture().build_registry())
        .invariant_catalog(catalog)
        .build()
        .unwrap();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let changed = update(execution, &runtime, [(model, "5")]);
    let current = create(execution, &runtime, "outside", "3");
    let outside = changed_entities(&current)[0];
    let target = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let services = runtime.owner_component_services();
    let input = services
        .preparation_port()
        .admit_retained_entity_selection(begin_main_transaction(&runtime), &source, model)
        .unwrap();
    let Err(Error::Preparation(TransactionCommitError::Conflict { error, .. })) = services
        .preparation_port()
        .prepare_retained_entity_selection(execution, input)
    else {
        panic!("the installed uniqueness rule must reject historical collision")
    };
    assert!(
        matches!(&error.class, ConflictClass::InvariantViolation {
            fields: InvariantViolationFields::UniqueEntityField { value, .. }, ..
        } if value == &string_aspect_value("3")),
        "{error:?}"
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .reference(),
        target.reference()
    );
    let lawful = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &target, model),
    );
    assert_eq!(
        values(execution, &runtime, &lawful, [model, outside]),
        ["5", "3"].map(string_aspect_value)
    );
    for commit in [&old, &changed, &current, &lawful] {
        release_test_commit_snapshot(&runtime, commit);
    }
}
