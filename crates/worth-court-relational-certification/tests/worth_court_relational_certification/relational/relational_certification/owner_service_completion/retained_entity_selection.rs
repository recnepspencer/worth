//! Real Native selection over two records in the same physical partition.
mod custody;
mod eligibility;
mod occurrences;
mod persistence;
mod provenance;
pub(crate) use occurrences::{fixture, perform};
mod refusals;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
use worth_relational::facade::{
    durability::RelationalCanonicalCommitArtifact,
    identity::{EntityId, KindId, PartitionId},
    mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent},
    runtime::{ProjectionAspectScope, RelationalRuntime},
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, entity_field_aspect, field_key, publications::*, AspectSchemaFixture,
};

#[test]
fn retained_selection_keeps_current_outside_truth_and_intervening_record_history() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_field_aspect(aspect_key("name"), field_key("label"))],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let model_created = create(execution, &runtime, "model", "3");
    let model = changed_entities(&model_created)[0];
    let historical = create(execution, &runtime, "outside", "old");
    let outside = changed_entities(&historical)[0];
    assert_eq!(model.partition_id, outside.partition_id);
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5"), (outside, "current")]);
    let target = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let services = runtime.owner_component_services();
    let transaction = runtime
        .begin_branch_transaction(&target, RelationalTransactionIntent::ordinary())
        .unwrap();
    let selection = services
        .preparation_port()
        .admit_retained_entity_selection(transaction, &source, model)
        .unwrap();
    let candidate = services
        .preparation_port()
        .prepare_retained_entity_selection(execution, selection)
        .unwrap();
    assert_eq!(candidate.prepared_changed_record_count(), Some(1));
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .reference(),
        target.reference()
    );
    let RelationalPublicationOutcome::Performed(performed) = services
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the original Native publication owner must perform selection")
    };
    let selected = services
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    assert_eq!(selected.commit.parents, vec![current.commit.commit_id]);
    assert_ne!(selected.commit.commit_id, historical.commit.commit_id);
    assert_ne!(selected.commit.commit_id, current.commit.commit_id);
    assert_eq!(
        values(execution, &runtime, &selected, [model, outside]),
        ["3", "current"].map(string_aspect_value)
    );
    assert_eq!(
        values(execution, &runtime, &historical, [model, outside]),
        ["3", "old"].map(string_aspect_value)
    );
    assert_eq!(
        values(execution, &runtime, &current, [model, outside]),
        ["5", "current"].map(string_aspect_value)
    );

    // This reads C through the now-D main edition, not through C's separate
    // retained snapshot. Replacing H's entire history would lose this value.
    let previous = runtime
        .read_truth()
        .visible_entity_at_version(execution, model, current.commit.version_id)
        .unwrap()
        .unwrap();
    let state = previous.authoritative_aspect_state.as_ref().unwrap();
    let ContractValidatedAspectValueView::Scalar(value) =
        state.get(&aspect_key("name")).unwrap().view()
    else {
        panic!("the installed scalar contract must remain scalar")
    };
    assert_eq!(value, &string_aspect_value("5"));

    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .map(Result::unwrap)
        .find(|artifact| artifact.commit_id() == selected.commit.commit_id)
        .unwrap();
    let readmitted =
        RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution).unwrap();
    let kind = readmitted.envelope().authority_kind();
    let provenance = kind
        .retained_entity_selection()
        .expect("selection is distinct from authored commands");
    assert_eq!(provenance.entity(), model);
    assert_eq!(provenance.kind(), KindId(1));
    assert_eq!(provenance.source_branch(), source.identity().branch_id());
    assert_eq!(
        provenance.source_generation(),
        source.reference().generation()
    );
    assert_eq!(
        provenance.source_head().selected_commit_id(),
        historical.commit.commit_id.0
    );
    assert_eq!(
        provenance.source_head().roots(),
        source.reference().target().as_basis().unwrap().roots()
    );
    assert!(
        readmitted.envelope().merged_plan.merged_intents.is_empty(),
        "selection must not reify field commands"
    );

    let future = update(execution, &runtime, [(model, "7")]);
    assert_eq!(
        values(execution, &runtime, &future, [model, outside]),
        ["7", "current"].map(string_aspect_value)
    );
    assert_eq!(
        values(execution, &runtime, &selected, [model, outside]),
        ["3", "current"].map(string_aspect_value)
    );
    assert_eq!(
        values(execution, &runtime, &current, [model, outside]),
        ["5", "current"].map(string_aspect_value)
    );
    assert_eq!(
        values(execution, &runtime, &historical, [model, outside]),
        ["3", "old"].map(string_aspect_value)
    );
    for commit in [&model_created, &historical, &current, &selected, &future] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

pub(crate) fn create(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    value: &str,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: string_aspect_field_patch([(
                        aspect_key("name"),
                        field_key("label"),
                        value,
                    )]),
                },
            ))),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}

pub(crate) fn update<'a>(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    values: impl IntoIterator<Item = (EntityId, &'a str)>,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    let mut batch = WorkerIntentBatch::new("ordinary-fields");
    for (entity_id, value) in values {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id,
                fields: string_aspect_field_patch([(
                    aspect_key("name"),
                    field_key("label"),
                    value,
                )]),
            },
        )));
    }
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}

pub(crate) fn values(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit: &CommitResult,
    entities: [EntityId; 2],
) -> [AspectValue; 2] {
    let view = runtime
        .read_truth()
        .project_snapshot(&commit.snapshot)
        .unwrap();
    entities.map(|entity| {
        view.entity_record_with_projection_scope(
            execution,
            entity,
            ProjectionAspectScope::fields(aspect_key("name"), [field_key("label")]),
            |record| {
                record
                    .observe_aspect_field(&aspect_key("name"), &field_key("label"))
                    .and_then(|value| value.value().cloned())
            },
        )
        .unwrap()
        .unwrap()
    })
}
