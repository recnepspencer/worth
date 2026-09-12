use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::{
    candidate_retained_representation as representation, CandidateItemKind,
    WorthQueryCandidateReservation,
};
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, Account, AccountStatus, AuthorizationWorld,
    IdentityExecutionSchema, MutationFreeEmitInput, MutationFreeEmitOperation,
    MutationFreeExternalEffect, MutationFreeNotice, Principal, TouchAccountInput,
    TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
};

#[test]
fn external_encoded_width_participates_in_candidate_denial() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let requirement = requirements(0, 0, 0, 0, 0, 1, 2);

    let mut accepted =
        reserved_external_builder(&world, &principal, &account, &request, requirement);
    accepted
        .emit_external(
            MutationFreeExternalEffect::reference(),
            MutationFreeNotice(1),
        )
        .expect("one typed and one encoded byte fit");

    let mut denied = reserved_external_builder(&world, &principal, &account, &request, requirement);
    let denial = denied
        .emit_external(
            MutationFreeExternalEffect::reference(),
            MutationFreeNotice(2),
        )
        .expect_err("equal typed footprint with wider encoding must deny");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
    );
    assert!(denied.finish().unwrap().effects.is_empty());
}

#[test]
fn multi_unlink_preflight_preserves_effects_and_reservation_on_denial() {
    let requirement = requirements(0, 0, 0, 1, 0, 0, 0);
    let mut reservation =
        WorthQueryCandidateReservation::admit(requirement, requirement, 1, 0, 1).unwrap();
    let effects: Vec<()> = Vec::new();

    let denial = reservation
        .charge_many(CandidateItemKind::Unlink, 2, 0, 0)
        .expect_err("two unlinks cannot enter capacity one");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
    );
    assert!(effects.is_empty());
    reservation
        .charge(CandidateItemKind::Unlink)
        .expect("bulk denial consumed no unlink reservation");
}

#[test]
fn foreign_delete_denial_preserves_capacity_for_valid_delete() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let requirement = requirements(0, 1, 0, 0, 0, 0, 0);
    let foreign_builder =
        reserved_touch_builder(&world, &principal, &account, &request, requirement);
    let foreign = foreign_builder.existing_entity(&account).unwrap();
    let mut candidate = reserved_touch_builder(&world, &principal, &account, &request, requirement);

    let denial = candidate
        .delete_entity(Account::reference(), &foreign)
        .expect_err("another candidate's target must be foreign");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget
    );
    let valid = candidate.existing_entity(&account).unwrap();
    candidate
        .delete_entity(Account::reference(), &valid)
        .expect("foreign denial consumed no delete reservation");
    assert_eq!(candidate.finish().unwrap().effects.len(), 1);
}

#[test]
fn long_keys_and_created_references_charge_each_retained_copy() {
    let entity_key = "e".repeat(64);
    let relation_key = "r".repeat(48);
    let created = EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(7),
        client_key: ClientKey::raw(entity_key.clone()),
    });

    assert_eq!(
        representation::created_entity(&entity_key, "Account"),
        Some(entity_key.capacity() + entity_key.len() * 2 + "Account".len())
    );
    assert_eq!(
        representation::relation(&relation_key, &created, &created),
        Some(relation_key.len() * 2 + entity_key.len() * 2)
    );
    assert_eq!(
        representation::output_binding("created", "Account", &created),
        Some("created".len() + "Account".len() + entity_key.len())
    );
}

#[test]
fn hostile_key_and_value_capacity_requires_the_exact_owned_capacity_ceiling() {
    let mut hostile_key = String::with_capacity(128);
    hostile_key.push_str("key");
    let mut hostile_value = String::with_capacity(96);
    hostile_value.push_str("value");
    assert!(hostile_key.capacity() > hostile_key.len() * 8);
    assert!(hostile_value.capacity() > hostile_value.len() * 8);

    let hostile_value = AspectValue::String(InternedString::Raw(hostile_value));
    let key_bytes = representation::created_entity(&hostile_key, "A").unwrap();
    let value_bytes = representation::value(&hostile_value);
    let exact_capacity = key_bytes.checked_add(value_bytes).unwrap();

    let insufficient = requirements(1, 0, 0, 0, 1, 0, exact_capacity - 1);
    let mut denied = WorthQueryCandidateReservation::admit(
        insufficient,
        insufficient,
        2,
        u64::try_from(exact_capacity - 1).unwrap(),
        1,
    )
    .unwrap();
    denied
        .charge_retained_representation(CandidateItemKind::Create, key_bytes, 0)
        .unwrap();
    let denial = denied
        .charge_retained_representation(CandidateItemKind::Write, value_bytes, 0)
        .expect_err("one byte below the owned-capacity total must deny");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
    );

    let exact = requirements(1, 0, 0, 0, 1, 0, exact_capacity);
    let mut admitted = WorthQueryCandidateReservation::admit(
        exact,
        exact,
        2,
        u64::try_from(exact_capacity).unwrap(),
        1,
    )
    .unwrap();
    admitted
        .charge_retained_representation(CandidateItemKind::Create, key_bytes, 0)
        .unwrap();
    admitted
        .charge_retained_representation(CandidateItemKind::Write, value_bytes, 0)
        .expect("the exact owned-capacity total must admit");
}

fn reserved_external_builder<'world>(
    world: &'world AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requirement: ApplicationCandidateRequirements,
) -> WorthQueryApplicationEffectProgramBuilder<
    IdentityExecutionSchema,
    MutationFreeEmitOperation,
    MutationFreeEmitInput,
    Account,
> {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MutationFreeEmitOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |_, _| {})
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
        .begin_reserved_effect_program(requirement, requirement)
        .unwrap()
}

fn reserved_touch_builder<'world>(
    world: &'world AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requirement: ApplicationCandidateRequirements,
) -> WorthQueryApplicationEffectProgramBuilder<
    IdentityExecutionSchema,
    TouchAccountOperation,
    TouchAccountInput,
    Account,
> {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
        .begin_reserved_effect_program(requirement, requirement)
        .unwrap()
}

fn requirements(
    creates: usize,
    deletes: usize,
    links: usize,
    unlinks: usize,
    writes: usize,
    emits: usize,
    retained_representation_bytes: usize,
) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(
            creates, deletes, links, unlinks, writes, emits,
        ),
        ApplicationCandidateResourceCeiling::bounded(retained_representation_bytes, 1),
    )
}
