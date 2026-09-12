use std::sync::Arc;

use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::*;
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

struct Schema;
struct Binding;
struct ForeignBinding;
struct Entity;

const PRESERVED: WorthQueryApplicationOutputRole<Binding, Entity, Preserve> =
    WorthQueryApplicationOutputRole::new("preserved");
const CREATED: WorthQueryApplicationOutputRole<Binding, Entity, Create> =
    WorthQueryApplicationOutputRole::new("created");
const RETIRED: WorthQueryApplicationOutputRole<Binding, Entity, Retire> =
    WorthQueryApplicationOutputRole::new("retired");

#[test]
fn duplicate_and_foreign_binding_roles_are_denied_before_commit() {
    let program = Arc::new(());
    let existing = existing_handle(EntityId::new(PartitionId::main(), 1, 0), &program);
    let mut candidate = prepared_candidate();
    candidate
        .bind(PRESERVED, &existing, &program)
        .expect("first exact role binding is accepted");

    assert_eq!(
        candidate
            .bind(PRESERVED, &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole
    );
    let foreign =
        WorthQueryApplicationOutputRole::<ForeignBinding, Entity, Preserve>::new("foreign");
    assert_eq!(
        candidate
            .bind(foreign, &existing, &program)
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
            .bind(PRESERVED, &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget
    );

    let local_existing = existing_handle(EntityId::new(PartitionId::main(), 3, 0), &program);
    assert_eq!(
        candidate
            .bind(CREATED, &local_existing, &program)
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
    candidate.prepare_test_role(PRESERVED, "entity");

    let undeclared = WorthQueryApplicationOutputRole::<Binding, Entity, Preserve>::new("other");
    assert_eq!(
        candidate
            .bind(undeclared, &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole
    );
    assert_eq!(
        candidate.validate_effects(&[]).unwrap_err().kind(),
        WorthQueryApplicationAttemptDenialKind::MissingOutputRole
    );

    let wrong_entity = WorthQueryApplicationEffectEntity::<Schema, Entity> {
        reference: EntityReference::Existing(EntityId::new(PartitionId::main(), 13, 0)),
        entity: "other-entity".to_owned(),
        created_effect: None,
        program: Arc::clone(&program),
        _marker: PhantomData,
    };
    assert_eq!(
        candidate
            .bind(PRESERVED, &wrong_entity, &program)
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
        .bind(RETIRED, &existing, &program)
        .expect("existing handle is eligible for retirement");
    assert_eq!(
        candidate.validate_effects(&[]).unwrap_err().kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch
    );
    candidate
        .validate_effects(&[WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id }])
        .expect("the role and actual retirement agree");
}

#[test]
fn created_role_must_name_an_actual_create_effect_before_commit() {
    let program = Arc::new(());
    let created_ref = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(9),
        client_key: ClientKey::raw("created-output"),
    };
    let created = WorthQueryApplicationEffectEntity::<Schema, Entity> {
        reference: EntityReference::Created(created_ref),
        entity: "entity".to_owned(),
        created_effect: Some(0),
        program: Arc::clone(&program),
        _marker: PhantomData,
    };
    let mut candidate = prepared_create_candidate();
    candidate
        .bind(CREATED, &created, &program)
        .expect("created handle belongs to the program");
    assert_eq!(
        candidate.validate_effects(&[]).unwrap_err().kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch
    );
}

#[test]
fn owner_resolved_creation_projects_the_exact_typed_identity() {
    let program = Arc::new(());
    let created = WorthQueryApplicationEffectEntity::<Schema, Entity> {
        reference: EntityReference::Created(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId::new(10),
            client_key: ClientKey::raw("owner-created-output"),
        }),
        entity: "entity".to_owned(),
        created_effect: Some(0),
        program: Arc::clone(&program),
        _marker: PhantomData,
    };
    let assigned = EntityId::new(PartitionId::main(), 11, 1);
    let mut candidate = prepared_create_candidate();
    candidate.bind(CREATED, &created, &program).unwrap();
    let committed = candidate.seal_with(|_| Some(assigned));

    assert_eq!(committed.entity(CREATED).unwrap().entity_id(), assigned);
}

fn existing_handle(
    entity_id: EntityId,
    program: &Arc<()>,
) -> WorthQueryApplicationEffectEntity<Schema, Entity> {
    WorthQueryApplicationEffectEntity {
        reference: EntityReference::Existing(entity_id),
        entity: "entity".to_owned(),
        created_effect: None,
        program: Arc::clone(program),
        _marker: PhantomData,
    }
}

fn prepared_candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_role(PRESERVED, "entity");
    candidate.prepare_test_role(CREATED, "entity");
    candidate.prepare_test_role(RETIRED, "entity");
    candidate
}

fn prepared_retire_candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_role(RETIRED, "entity");
    candidate
}

fn prepared_create_candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_role(CREATED, "entity");
    candidate
}
