//! Created output roles must name an actual create effect in an existing context partition.
use super::*;

#[test]
fn created_role_must_name_an_actual_create_effect_before_commit() {
    let program = Arc::new(());
    let created_ref = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(9),
        client_key: ClientKey::raw("created-output"),
    };
    let created = WorthQueryApplicationEffectEntity::<Schema, Entity>::new_for_test(
        EntityReference::Created(created_ref),
        "entity".to_owned(),
        Some(0),
        Arc::clone(&program),
        PhantomData,
    );
    let mut candidate = prepared_create_candidate();
    candidate
        .bind_for_test(
            OutputRoleUse::fixed_for_test::<OnlyCreated>(),
            &created,
            &program,
        )
        .expect("created handle belongs to the program");
    assert_eq!(
        candidate.validate_effects_for_test(&[]).unwrap_err().kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch
    );
}

#[test]
fn created_role_accepts_a_create_effect_in_an_existing_context_partition() {
    let program = Arc::new(());
    let context_partition = PartitionId::new(7);
    let created = WorthQueryApplicationEffectEntity::<Schema, Entity>::new_for_test(
        EntityReference::Created(CreatedEntityRef {
            partition_id: context_partition,
            kind_id: KindId::new(9),
            client_key: ClientKey::raw("context-created-output"),
        }),
        "entity".to_owned(),
        Some(0),
        Arc::clone(&program),
        PhantomData,
    );
    let effect = WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: KindId::new(9),
        key: "context-created-output".to_owned(),
        fields: BTreeMap::new(),
        partition: WorthQueryApplicationCreationPartition::Context(context_partition),
    };
    let mut candidate = prepared_create_candidate();
    candidate
        .bind_for_test(
            OutputRoleUse::fixed_for_test::<OnlyCreated>(),
            &created,
            &program,
        )
        .unwrap();

    candidate
        .validate_effects_for_test(&[effect])
        .expect("context-partition creation is the declared created output");
}
