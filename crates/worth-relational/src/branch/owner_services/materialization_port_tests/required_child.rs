//! Suspension preserves structural identity across a required mixed relation.
use super::*;
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::{runtime::RelationalRuntimeApi, schema::*};
use crate::schema::data::{
    CardinalityContractDeclaration, EndpointKindContractDeclaration, MinimumCardinalityEnforcement,
    PairMinimumSemantics, RelationIntegrityDeclarations,
};
use std::sync::{atomic::AtomicUsize, atomic::Ordering, Arc};

#[path = "required_child/completeness_rule.rs"]
mod completeness_rule;

#[test]
fn retained_root_required_child_suspends_without_authorizing_relation_deletion() {
    run(false);
}

#[test]
fn retained_root_custom_child_completeness_survives_suspension() {
    run(true);
}

fn run(custom: bool) {
    let (runtime, evaluations) = runtime(custom);
    let root_ref = created(KindId(1), "retained-root");
    let child_ref = created(KindId(3), "generated-child");
    let relation_ref = crate::transactions::data::CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("required-child"),
        source: EntityReference::Created(root_ref.clone()),
        target: EntityReference::Created(child_ref.clone()),
    };
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("complete-required-child")
                .push(create(&root_ref))
                .push(create(&child_ref))
                .push(MutationIntent::Create(CreateIntent::Relation(
                    RelationSpec {
                        partition_id: relation_ref.partition_id,
                        kind_id: relation_ref.kind_id,
                        client_key: relation_ref.client_key.clone(),
                        source: relation_ref.source.clone(),
                        target: relation_ref.target.clone(),
                        fields: Default::default(),
                    },
                ))),
        )
        .unwrap();
    let publication = transaction
        .commit(&runtime)
        .expect("required topology publishes together");
    let root = publication.created_entity(&root_ref).unwrap();
    let child = publication.created_entity(&child_ref).unwrap();
    let relation = publication.created_relation(&relation_ref).unwrap();
    let expected = runtime
        .read_truth()
        .read_snapshot(&publication.snapshot)
        .unwrap();

    if custom {
        completeness_rule::deny_invalid_child_value(&runtime, child);
    }

    let mut deletion = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    deletion
        .push_batch(WorkerIntentBatch::new("delete-required-child-link").push(
            MutationIntent::Relation(crate::transactions::data::RelationMutationIntent::Delete(
                crate::transactions::data::DeleteRelationIntent {
                    relation_id: relation,
                },
            )),
        ))
        .unwrap();
    let denied = deletion
        .commit(&runtime)
        .expect_err("ordinary deletion violates the root minimum");
    assert!(matches!(denied,
        crate::transactions::data::TransactionCommitError::Conflict { error, .. }
        if error.code() == crate::diagnostics::data::DiagnosticCode::RelationCardinalityViolation
    ));

    let services = runtime.owner_component_services();
    let (_, basis) = services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before_suspension = evaluations
        .each_ref()
        .map(|count| count.load(Ordering::SeqCst));
    let suspended = services
        .materialization_port()
        .suspend_generated_materialization(&basis, &[child])
        .expect("suspension keeps required incidence while withholding generated payload");
    if custom {
        for (phase, (count, before)) in evaluations.iter().zip(before_suspension).enumerate() {
            assert!(
                count.load(Ordering::SeqCst) > before,
                "phase {phase} validated the complete logical child during suspension"
            );
        }
    }
    assert_eq!(suspended.custody.records().len(), 2);
    assert!(runtime
        .read_truth()
        .read_snapshot(&suspended.commit.snapshot)
        .is_none());
    let (_, unavailable) = services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        services
            .materialization_port()
            .retained_entity_kind(&unavailable, &suspended.custody, root)
            .unwrap(),
        KindId(1)
    );
    let custody = if custom {
        let rejected = services
            .materialization_port()
            .rematerialize_generated_materialization(
                &unavailable,
                suspended.custody,
                vec![RelationalEntityMaterialization {
                    entity_id: child,
                    kind_id: KindId(3),
                    fields: valid_field_patch(false),
                }],
                vec![RelationalRelationMaterialization {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: root,
                    target: child,
                    fields: Default::default(),
                }],
            )
            .expect_err("rematerialization must validate the reconstructed child value");
        let crate::branch::RelationalMaterializationError::Commit(error) = &rejected.error else {
            panic!("expected custom commit denial, got {:?}", rejected.error);
        };
        completeness_rule::assert_custom_violation(error);
        assert!(runtime
            .read_truth()
            .read_snapshot(&suspended.commit.snapshot)
            .is_none());
        rejected.custody
    } else {
        suspended.custody
    };
    let restored = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable,
            custody,
            vec![RelationalEntityMaterialization {
                entity_id: child,
                kind_id: KindId(3),
                fields: valid_field_patch(true),
            }],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: root,
                target: child,
                fields: Default::default(),
            }],
        )
        .expect("exact required child and edge restore under owner custody");
    let actual = runtime
        .read_truth()
        .read_snapshot(&restored.snapshot)
        .unwrap();
    assert_eq!(expected.entities(), actual.entities());
    assert_eq!(expected.relations(), actual.relations());
}

fn created(kind_id: KindId, name: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id,
        client_key: crate::symbols::data::ClientKey::raw(name),
    }
}

fn create(reference: &CreatedEntityRef) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: reference.partition_id,
        kind_id: reference.kind_id,
        client_key: reference.client_key.clone(),
        fields: valid_field_patch(true),
    }))
}

fn runtime(custom: bool) -> (crate::runtime::RelationalRuntime, [Arc<AtomicUsize>; 3]) {
    let evaluations = std::array::from_fn(|_| Arc::new(AtomicUsize::new(0)));
    let mut registry = RelationalSchemaRegistry::new();
    for (kind_id, kind_name) in [(KindId(1), "retained-root"), (KindId(3), "generated-child")] {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id,
                kind_name: kind_name.into(),
                schema_id: SchemaId("mixed-required".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    crate::tests::support::entity_bool_field_aspect(
                        crate::tests::support::aspect_key("valid"),
                        crate::tests::support::field_key("valid"),
                    ),
                ]),
            })
            .unwrap();
    }
    registry = registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "required-child".into(),
            schema_id: SchemaId("mixed-required".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "root-child-kinds".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(3)],
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                }],
                vec![CardinalityContractDeclaration {
                    contract_id: "required-root-child".into(),
                    source_min: Some(1),
                    source_max: Some(1),
                    target_min: None,
                    target_max: None,
                    pair_min: None,
                    pair_max: None,
                    pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
                    minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
                }],
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        })
        .unwrap();
    let mut builder = RelationalRuntimeApi::builder().schema_registry(registry);
    if custom {
        use crate::validation::data::InvariantExecutionPoint;
        for (index, (execution_point, identity)) in [
            (
                InvariantExecutionPoint::CommitBoundary,
                "test.retained-root.child-completeness.commit",
            ),
            (
                InvariantExecutionPoint::MutationSensitive,
                "test.retained-root.child-completeness.mutation",
            ),
            (
                InvariantExecutionPoint::SnapshotPublication,
                "test.retained-root.child-completeness.publication",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            builder = builder.custom_invariant(
                crate::facade::runtime::CustomInvariantRegistration::new(
                    completeness_rule::RequiredChild {
                        execution_point,
                        identity,
                        evaluations: Arc::clone(&evaluations[index]),
                    },
                )
                .unwrap(),
            );
        }
    }
    (builder.build(), evaluations)
}

fn valid_field_patch(value: bool) -> crate::transactions::data::AspectFieldPatch {
    crate::transactions::data::AspectFieldPatch::from_locator(
        crate::transactions::data::planned_single_field_locator(
            crate::tests::support::aspect_key("valid"),
            crate::tests::support::field_key("valid"),
        ),
        worth_foundational::facade::AspectValue::Bool(value),
    )
}
