//! Real recovery preserves admitted catalog dependencies and selected registry owners.
use super::super::fixtures as physical;
use super::fixtures::*;

pub(super) fn persisted(registry: RelationalSchemaRegistry) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build_durable(TestDurableMedium::new().attachment())
        .expect("valid native runtime configuration")
}

#[test]
fn physical_membership_recipe_and_alternating_derived_catalog_restore_from_real_checkpoints() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut registry = physical::registry();
    let physical_identity = DerivedRelationIdentity {
        schema: registry
            .entity_registration(KindId(1))
            .unwrap()
            .schema_id
            .clone(),
        version: SchemaVersionId(1),
        name: "MembershipInput".into(),
    };
    registry = registry
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: physical_identity.clone(),
            from: KindId(1).into(),
            to: KindId(3).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(5),
                from: KindId(1),
                to: KindId(3),
            }],
            laws: None,
        }])
        .unwrap();
    let handle = registry.derived_relation(&physical_identity).unwrap();
    let extended = RelationalSchemaRegistry::new()
        .extend(registry.clone())
        .unwrap();
    assert_eq!(
        extended.derived_relation(&physical_identity),
        Some(handle.clone())
    );
    let runtime = persisted(extended);
    let born = physical::commit(execution, &runtime, "main", physical::birth(&[1, 0]));
    let member = physical::members(&born, 2)[0];
    let truth = super::laws::current_truth(execution, &runtime);
    let expected = truth
        .1
        .iter()
        .filter_map(|(_, kind, from, to)| {
            (*kind == KindId(5) && *from == member).then_some(NativeMemberKey::Entity(*to))
        })
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 1);
    let (_, recovered) = checkpoint_and_recover_with(execution, &runtime, || persisted(registry));
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".into()));
    let pin = recovered
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    assert_eq!(
        pin.derived_relations_from(execution, &handle, member, None)
            .unwrap()
            .data()
            .endpoints(),
        expected
    );
    assert_eq!(super::laws::current_truth(execution, &recovered), truth);

    // A genuinely longer admitted dependency chain: recipes→membership→recipe.
    let registry = super::metadata_laws::law_registry(false);
    let handle = registry.derived_relation(&identity("Law")).unwrap();
    let extended = RelationalSchemaRegistry::new()
        .extend(registry.clone())
        .unwrap();
    assert_eq!(
        extended.derived_relation(&identity("Law")),
        Some(handle.clone())
    );
    let runtime = persisted(extended);
    let born = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("alternating-catalog").push(entity(1, 0)),
    );
    let carrier = born.created_entity(&created(1, 0)).unwrap();
    let names = [
        read(execution, &runtime, &born.snapshot, key(carrier, 0)).identity(),
        read(execution, &runtime, &born.snapshot, key(carrier, 1)).identity(),
    ];
    let (_, recovered) = checkpoint_and_recover_with(execution, &runtime, || persisted(registry));
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".into()));
    let pin = recovered
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    assert_eq!(
        pin.derived_relations_from(execution, &handle, carrier, None)
            .unwrap()
            .data()
            .endpoints()
            .len(),
        2
    );
    for (side, &name) in names.iter().enumerate() {
        assert_eq!(
            read(execution, &recovered, &snapshot, key(carrier, side)).identity(),
            name
        );
    }
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        recovered.history().commit_envelopes_snapshot()
    );
}

#[test]
fn changed_configured_domain_cannot_replace_the_actual_recovered_root_or_its_handles() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = source_registry();
    let definition = original.derived_relation(&identity("Adjacency")).unwrap();
    let original_domain = original.projected_member_domain(&domain()).unwrap();
    let runtime = persisted(original.clone());
    let born = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("domain-root").push(entity(1, 0)),
    );
    let carrier = born.created_entity(&created(1, 0)).unwrap();
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let (_, compatible) = recover_with(execution, &runtime, || persisted(original));
    let snapshot = snapshot_for_owner_branch(&compatible, &BranchId("main".into()));
    let pin = compatible
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    assert_eq!(
        pin.schema().projected_member_domain(&domain()),
        Some(original_domain.clone())
    );
    assert!(pin
        .derived_relations_from(execution, &definition, key(carrier, 0), None)
        .unwrap()
        .data()
        .endpoints()
        .is_empty());

    let changed = source_registry_with_cases(&["Negative", "Positive", "Third"]);
    let foreign_definition = changed.derived_relation(&identity("Adjacency")).unwrap();
    let foreign_domain = changed.projected_member_domain(&domain()).unwrap();
    let opaque = runtime
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut restored = persisted(changed);
    let before = restored.history().commit_envelopes_snapshot();
    let error = restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    assert_eq!(
        expect_durability_failure(error).class,
        crate::durability::data::RecoveryFailureClass::SchemaMismatch
    );
    assert_eq!(restored.history().commit_envelopes_snapshot(), before);
    // A native restore must enforce the same declared-schema boundary as reopen.
    let before = restored
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let denied = restored
        .durability_recovery()
        .restore_native_checkpoint(execution, &opaque)
        .unwrap_err();
    assert_eq!(
        expect_durability_failure(denied).class,
        crate::durability::data::RecoveryFailureClass::DeclaredSchemaMismatch
    );
    assert_eq!(
        restored
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap(),
        before
    );
    let mut restored = persisted(source_registry());
    restored
        .durability_recovery()
        .restore_native_checkpoint(execution, &opaque)
        .unwrap();
    let snapshot = snapshot_for_owner_branch(&restored, &BranchId("main".into()));
    let pin = restored
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    let actual_domain = pin.schema().projected_member_domain(&domain()).unwrap();
    assert_eq!(actual_domain.parts(), original_domain.parts());
    assert_ne!(actual_domain, foreign_domain);
    assert!(!pin.schema().admits_projected_member_domain(&foreign_domain));
    let error = pin
        .derived_relations_from(execution, &foreign_definition, key(carrier, 0), None)
        .err()
        .unwrap();
    assert_eq!(
        error,
        DerivedRelationReadDenial::Definition(DerivedRelationDenial::ForeignDefinition)
    );
    let actual_definition = pin
        .schema()
        .derived_relation(&identity("Adjacency"))
        .unwrap();
    assert!(pin
        .derived_relations_from(execution, &actual_definition, key(carrier, 0), None)
        .unwrap()
        .data()
        .endpoints()
        .is_empty());
}
