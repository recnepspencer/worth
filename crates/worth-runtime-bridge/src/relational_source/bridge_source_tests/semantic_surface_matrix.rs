use crate::facade::TruthDeltaSurfaceKind;
use worth_foundational::facade::{
    AspectBinding, AspectIdentity, AspectKey, AuthoritativeAspectChangeKind, ScalarAspectType,
};

use crate::relational_source::relational_test_support::{
    aspect_key, changed_entities, create_entity_outcome, create_relation_outcome,
    entity_summary_struct_aspect, field_key, string_aspect_field_patch, AspectSchemaFixture,
};
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::schema::DeclaredAspectContractBinding;
use worth_relational::facade::transactions::{
    CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::support::{bridge_envelopes_at_current_observation, runtime_with_test_schema};

#[test]
fn real_entity_and_relation_transactions_preserve_semantic_binding_surfaces() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = changed_entities(&create_entity_outcome(&runtime, "source"))[0];
    let target = changed_entities(&create_entity_outcome(&runtime, "target"))[0];

    create_relation_outcome(&runtime, source, target, "edge");
    let commit = runtime
        .publication()
        .latest_bundle()
        .unwrap()
        .commit
        .commit_id;
    let envelope = bridge_envelopes_at_current_observation(runtime, [commit], resource_request)
        .pop()
        .expect("relation publication");
    let items = envelope.patch_body().canonical_items();

    assert_semantic_item(
        items,
        &AspectBinding::RelationField {
            field: worth_foundational::facade::FieldKey::new("label").unwrap(),
        },
        AuthoritativeAspectChangeKind::WholeAspectSet,
        TruthDeltaSurfaceKind::AuthoritativeAspect,
    );
    assert_semantic_item(
        items,
        &AspectBinding::RelationSourceEndpoint,
        AuthoritativeAspectChangeKind::RelationSourceEndpoint,
        TruthDeltaSurfaceKind::EntityRelationEndpoint,
    );
    assert_semantic_item(
        items,
        &AspectBinding::RelationTargetEndpoint,
        AuthoritativeAspectChangeKind::RelationTargetEndpoint,
        TruthDeltaSurfaceKind::EntityRelationEndpoint,
    );
    assert_semantic_item(
        items,
        &AspectBinding::LifecycleTransition,
        AuthoritativeAspectChangeKind::LifecycleCreate,
        TruthDeltaSurfaceKind::LifecycleTransition,
    );
}

#[test]
fn real_entity_transaction_preserves_field_lifecycle_and_structural_surfaces() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let mut fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    fixture.entity_aspects.extend([
        entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        structural_binding("region", 91, AspectBinding::StructuralRegion),
        structural_binding("partition", 92, AspectBinding::StructuralPartition),
        structural_binding("facet", 93, AspectBinding::StructuralFacet),
    ]);
    let runtime = fixture.build_runtime();
    let mut creation =
        crate::relational_source::relational_test_support::test_owner_begin_transaction_for_main(
            &runtime,
        );
    creation
        .push_batch(
            WorkerIntentBatch::new("structural-create").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: worth_relational::facade::identity::KindId(1),
                    client_key: worth_relational::facade::symbols::ClientKey::raw("solid"),
                    fields: string_aspect_field_patch([
                        (aspect_key("name"), field_key("name"), "solid"),
                        (aspect_key("summary"), field_key("title"), "solid"),
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let created = creation
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("real structural create");
    let entity = changed_entities(&created)[0];
    let structural_commit = runtime
        .publication()
        .latest_bundle()
        .unwrap()
        .commit
        .commit_id;
    let mut transaction =
        crate::relational_source::relational_test_support::test_owner_begin_transaction_for_main(
            &runtime,
        );
    transaction
        .push_batch(
            WorkerIntentBatch::new("summary-field-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: string_aspect_field_patch([
                        (aspect_key("summary"), field_key("title"), "solid"),
                        (aspect_key("summary"), field_key("status"), "ready"),
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let field_commit = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("real struct-field update")
        .commit
        .commit_id;
    let mut envelopes = bridge_envelopes_at_current_observation(
        runtime,
        [structural_commit, field_commit],
        resource_request,
    );
    let field_publication = envelopes.pop().expect("field publication");
    let structural_publication = envelopes.pop().expect("structural publication");
    let items = structural_publication.patch_body().canonical_items();

    assert_semantic_item(
        items,
        &AspectBinding::LifecycleTransition,
        AuthoritativeAspectChangeKind::LifecycleCreate,
        TruthDeltaSurfaceKind::LifecycleTransition,
    );
    assert_semantic_item(
        items,
        &AspectBinding::StructuralRegion,
        AuthoritativeAspectChangeKind::StructuralCreate,
        TruthDeltaSurfaceKind::EntityRegion,
    );
    assert_semantic_item(
        items,
        &AspectBinding::StructuralPartition,
        AuthoritativeAspectChangeKind::StructuralCreate,
        TruthDeltaSurfaceKind::EntityPartition,
    );
    assert_semantic_item(
        items,
        &AspectBinding::StructuralFacet,
        AuthoritativeAspectChangeKind::StructuralCreate,
        TruthDeltaSurfaceKind::EntityFacet,
    );
    assert_semantic_item(
        field_publication.patch_body().canonical_items(),
        &AspectBinding::EntityField {
            field: field_key("summary"),
        },
        AuthoritativeAspectChangeKind::FieldSet,
        TruthDeltaSurfaceKind::EntityField,
    );
}

fn structural_binding(
    key: &str,
    identity: u64,
    binding: AspectBinding,
) -> DeclaredAspectContractBinding {
    let key = AspectKey::new(key).unwrap();
    DeclaredAspectContractBinding {
        binding,
        contract: worth_foundational::facade::aspects()
            .contract()
            .for_key(key)
            .identified_by(AspectIdentity(identity))
            .at_revision(
                worth_foundational::facade::aspects()
                    .vocabulary()
                    .revision(1),
            )
            .scalar(ScalarAspectType::String),
    }
}

fn assert_semantic_item(
    items: &[crate::facade::BridgeCommittedPatchItem],
    binding: &AspectBinding,
    kind: AuthoritativeAspectChangeKind,
    surface: TruthDeltaSurfaceKind,
) {
    assert!(
        items.iter().any(|item| {
            item.surface_kind() == surface
                && item
                    .semantic_change()
                    .is_some_and(|change| change.binding() == binding && change.kind() == kind)
        }),
        "missing {binding:?} / {kind:?} / {surface:?} from {items:#?}"
    );
}
