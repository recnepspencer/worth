pub use std::collections::BTreeMap;
pub use std::marker::PhantomData;
pub use std::sync::Arc;

pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::symbols::ClientKey;
pub use worth_relational::facade::transactions::CreatedEntityRef;
pub use worth_relational::facade::transactions::EntityReference;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::role_use::OutputRoleUse;
pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::candidate::WorthQueryApplicationOutputCorrespondenceCandidate;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationOutputProjectionDenial;
pub use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRoleNameDenial;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationCreationPartition;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectEntity;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationRealizedEffect;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleCardinality as Cardinality;

pub use crate::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::tests::declaration::*;

#[test]
fn runtime_role_names_reject_ambiguous_or_unbounded_representations() {
    assert!(matches!(
        WorthQueryApplicationOutputRoleNameDenial::validate(" role"),
        Err(WorthQueryApplicationOutputRoleNameDenial::SurroundingWhitespace)
    ));
    assert!(matches!(
        WorthQueryApplicationOutputRoleNameDenial::validate("role\nmember"),
        Err(WorthQueryApplicationOutputRoleNameDenial::ControlCharacter)
    ));
    assert!(matches!(
        WorthQueryApplicationOutputRoleNameDenial::validate(&"x".repeat(257)),
        Err(
            WorthQueryApplicationOutputRoleNameDenial::RepresentationTooLarge {
                maximum_bytes: 256,
                required_bytes: 257,
            }
        )
    ));
}

/// A stand-in operation binding type for readmitted correspondences.
pub struct Binding;

#[test]
fn checkpoint_roles_reject_invalid_names_unknown_entities_and_duplicates() {
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let checkpoint_role = |role: &str, entity_name: &str| WorthQueryCheckpointOutputRole::new_for_test(role.into(), WorthQueryApplicationOutputPosture::Preserve, entity_name.into(), entity);

    assert!(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
            TypeId::of_for_test::<Binding>(),
            TypeId::of_for_test::<Outputs>(),
            std::iter::empty(),
            &[checkpoint_role(" invalid", "entity")],
            |_| Some(TypeId::of_for_test::<Entity>()),
            &execution,
        )
        .unwrap_err()
        .contains("checkpoint output role  invalid")
    );
    assert!(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
            TypeId::of_for_test::<Binding>(),
            TypeId::of_for_test::<Outputs>(),
            std::iter::empty(),
            &[checkpoint_role("valid", "unknown")],
            |_| None,
            &execution,
        )
        .unwrap_err()
        .contains("names an uninstalled entity unknown")
    );
    assert!(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
            TypeId::of_for_test::<Binding>(),
            TypeId::of_for_test::<Outputs>(),
            std::iter::empty(),
            &[
                checkpoint_role("same", "entity"),
                checkpoint_role("same", "entity"),
            ],
            |_| Some(TypeId::of_for_test::<Entity>()),
            &execution,
        )
        .unwrap_err()
        .contains("role same is duplicated")
    );
}

#[test]
fn duplicate_and_foreign_binding_roles_are_denied_before_commit() {
    let program = Arc::new(());
    let existing = existing_handle(EntityId::new(PartitionId::main(), 1, 0), &program);
    let mut candidate = prepared_candidate();
    candidate
        .bind_for_test(OutputRoleUse::fixed_for_test::<Preserved>(), &existing, &program)
        .expect("first exact role binding is accepted");

    assert_eq!(
        candidate
            .bind_for_test(OutputRoleUse::fixed_for_test::<Preserved>(), &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole
    );
    assert_eq!(
        candidate
            .bind_for_test(OutputRoleUse::fixed_for_test::<Foreign>(), &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignOutputRole
    );
}

#[test]
fn foreign_handles_and_wrong_action_references_are_denied() {
    let program = Arc::new(());
    let foreign_program = Arc::new(());
    let existing = existing_handle(EntityId::new(PartitionId::main(), 2, 0), &foreign_program);
    let mut candidate = prepared_candidate();
    assert_eq!(
        candidate
            .bind_for_test(OutputRoleUse::fixed_for_test::<Preserved>(), &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget
    );

    let local_existing = existing_handle(EntityId::new(PartitionId::main(), 3, 0), &program);
    assert_eq!(
        candidate
            .bind_for_test(OutputRoleUse::fixed_for_test::<Created>(), &local_existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch
    );
}

#[test]
fn declaration_inventory_denies_undeclared_missing_and_wrong_entity_roles() {
    let program = Arc::new(());
    let existing = existing_handle(EntityId::new(PartitionId::main(), 12, 0), &program);
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_contract_for_test::<Schema, PreserveOutputs>();

    let undeclared = stray::<PreserveOutputs, Entity>(
        "other",
        WorthQueryApplicationOutputPosture::Preserve,
        Cardinality::ExactlyOne,
    );
    assert_eq!(
        candidate
            .bind_for_test(undeclared, &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole
    );
    assert_eq!(
        candidate.validate_effects_for_test(&[]).unwrap_err().kind(),
        WorthQueryApplicationAttemptDenialKind::MissingOutputRole
    );

    let wrong_entity = WorthQueryApplicationEffectEntity::<Schema, Entity>::new_for_test(EntityReference::Existing(EntityId::new(PartitionId::main(), 13, 0)), "other-entity".to_owned(), None, Arc::clone(&program), PhantomData);
    assert_eq!(
        candidate
            .bind_for_test(
                OutputRoleUse::fixed_for_test::<OnlyPreserved>(),
                &wrong_entity,
                &program
            )
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleEntityMismatch
    );
}

#[test]
fn retire_role_requires_the_matching_delete_effect() {
    let program = Arc::new(());
    let entity_id = EntityId::new(PartitionId::main(), 4, 0);
    let existing = existing_handle(entity_id, &program);
    let mut candidate = prepared_retire_candidate();
    candidate
        .bind_for_test(OutputRoleUse::fixed_for_test::<OnlyRetired>(), &existing, &program)
        .expect("existing handle is eligible for retirement");
    assert_eq!(
        candidate.validate_effects_for_test(&[]).unwrap_err().kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch
    );
    candidate
        .validate_effects_for_test(&[WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id }])
        .expect("the role and actual retirement agree");
    let sealed = candidate.seal_with_for_test(|_| None);
    assert_eq!(
        sealed.retired_entity_ids().collect::<Vec<_>>(),
        vec![entity_id],
        "only the typed retirement validated against its delete enters the retirement selector"
    );
}

#[test]
fn created_role_must_name_an_actual_create_effect_before_commit() {
    let program = Arc::new(());
    let created_ref = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(9),
        client_key: ClientKey::raw("created-output"),
    };
    let created = WorthQueryApplicationEffectEntity::<Schema, Entity>::new_for_test(EntityReference::Created(created_ref), "entity".to_owned(), Some(0), Arc::clone(&program), PhantomData);
    let mut candidate = prepared_create_candidate();
    candidate
        .bind_for_test(OutputRoleUse::fixed_for_test::<OnlyCreated>(), &created, &program)
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
    let created = WorthQueryApplicationEffectEntity::<Schema, Entity>::new_for_test(EntityReference::Created(CreatedEntityRef {
            partition_id: context_partition,
            kind_id: KindId::new(9),
            client_key: ClientKey::raw("context-created-output"),
        }), "entity".to_owned(), Some(0), Arc::clone(&program), PhantomData);
    let effect = WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: KindId::new(9),
        key: "context-created-output".to_owned(),
        fields: BTreeMap::new(),
        partition: WorthQueryApplicationCreationPartition::Context(context_partition),
    };
    let mut candidate = prepared_create_candidate();
    candidate
        .bind_for_test(OutputRoleUse::fixed_for_test::<OnlyCreated>(), &created, &program)
        .unwrap();

    candidate
        .validate_effects_for_test(&[effect])
        .expect("context-partition creation is the declared created output");
}

#[test]
fn owner_resolved_creation_projects_the_exact_typed_identity() {
    let program = Arc::new(());
    let created = WorthQueryApplicationEffectEntity::<Schema, Entity>::new_for_test(EntityReference::Created(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId::new(10),
            client_key: ClientKey::raw("owner-created-output"),
        }), "entity".to_owned(), Some(0), Arc::clone(&program), PhantomData);
    let assigned = EntityId::new(PartitionId::main(), 11, 1);
    let mut candidate = prepared_create_candidate();
    candidate
        .bind_for_test(OutputRoleUse::fixed_for_test::<OnlyCreated>(), &created, &program)
        .unwrap();
    let committed = candidate.seal_with_for_test(|_| Some(assigned));

    assert_eq!(
        committed
            .outputs_of_for_test::<CreateOutputs>()
            .unwrap()
            .entity::<OnlyCreated>()
            .unwrap()
            .entity_id(),
        assigned
    );
    // Reading the created role as another entity cannot be written with a
    // marker; the erased use is still refused.
    let wrong_entity = stray::<CreateOutputs, WrongEntity>(
        "created",
        WorthQueryApplicationOutputPosture::Create,
        Cardinality::ExactlyOne,
    );
    assert_eq!(
        committed.bound_entity_for_test(&wrong_entity).err(),
        Some(WorthQueryApplicationOutputProjectionDenial::EntityMismatch)
    );
}

pub fn existing_handle(
    entity_id: EntityId,
    program: &Arc<()>,
) -> WorthQueryApplicationEffectEntity<Schema, Entity> {
    WorthQueryApplicationEffectEntity::new_for_test(EntityReference::Existing(entity_id), "entity".to_owned(), None, Arc::clone(program), PhantomData)
}

pub fn created_handle(
    key: &'static str,
    kind: u32,
    program: &Arc<()>,
) -> WorthQueryApplicationEffectEntity<Schema, Entity> {
    WorthQueryApplicationEffectEntity::new_for_test(EntityReference::Created(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId::new(kind),
            client_key: ClientKey::raw(key),
        }), "entity".to_owned(), Some(0), Arc::clone(program), PhantomData)
}

pub fn create_effect(key: &'static str, kind: u32) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: KindId::new(kind),
        key: key.to_owned(),
        fields: BTreeMap::new(),
        partition: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationCreationPartition::Issued,
    }
}

pub fn prepared<Contract>() -> WorthQueryApplicationOutputCorrespondenceCandidate
where
    Contract:
        worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract<
            Schema,
        >,
{
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_contract_for_test::<Schema, Contract>();
    candidate
}

pub fn prepared_candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    prepared::<Outputs>()
}

pub fn prepared_retire_candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    prepared::<RetireOutputs>()
}

pub fn prepared_create_candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    prepared::<CreateOutputs>()
}

#[path = "cardinality/mod.rs"]
pub mod cardinality;
#[path = "declaration/mod.rs"]
pub mod declaration;
#[path = "family/mod.rs"]
pub mod family;
