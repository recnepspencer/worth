use crate::facade::durability::RecoveryVerificationMode;
use crate::facade::history::BranchId;
use crate::facade::transactions::{
    EntityReference, MutationIntent, RelationMutationIntent, UpdateRelationEndpointsIntent,
    WorkerIntentBatch,
};
use crate::tests::support::{
    aspect_key, create_entity, create_relation, delete_relation_on_branch, field_key,
    runtime_with_test_schema, update_entity,
};
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, CanonicalFieldPath, InternedString, LocatorAuthority,
};

use super::{allow_plan, authorization_fixture, forward_traversal, ENTITY_KIND, RELATION_KIND};
use crate::authorization::{
    RelationalAuthorizationDependencyDenial, RelationalAuthorizationDurableDependencies,
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathPlan,
    RelationalAuthorizationPredicate, MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES,
};

#[test]
fn planned_locator_is_resolved_to_native_authoritative_revision() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let path = RelationalAuthorizationPathPlan::new(
        [forward_traversal(), forward_traversal()],
        [RelationalAuthorizationPredicate::new(
            1,
            ENTITY_KIND,
            AspectFieldLocator::new(
                LocatorAuthority::Planned,
                aspect_key("name"),
                CanonicalFieldPath::single(field_key("name")),
            ),
            AspectValue::String(InternedString::Raw("approver".to_owned())),
        )],
    );
    let plan = RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        fixture.principal,
        fixture.scope,
        ENTITY_KIND,
        ENTITY_KIND,
        [path],
        [],
    )
    .expect("planned locator participates in the installed path");
    let evidence = fixture.runtime.observe_authorization(plan).unwrap();
    assert!(evidence.paths()[0].matched());
    fixture
        .runtime
        .capture_authorization_durable_dependencies(&evidence)
        .expect("planned locator is stamped against its authoritative source field");
}

#[test]
fn source_native_stamp_survives_unrelated_publication_and_wire_roundtrip() {
    let fixture = authorization_fixture();
    let before = stamp_now(&fixture);
    let wire = before.to_wire_string().expect("bounded stamp encodes");
    assert!(wire.len() <= MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES * 2);
    let recovered = RelationalAuthorizationDurableDependencies::from_wire_string(&wire)
        .expect("canonical stamp decodes after restart");
    assert!(before.matches(&recovered));
    assert!(recovered
        .source_revisions_match(
            &fixture.runtime,
            &fixture.runtime.visibility_authority().snapshot()
        )
        .unwrap());

    let unrelated = create_entity(&fixture.runtime, "unrelated");
    let other = create_entity(&fixture.runtime, "other");
    create_relation(&fixture.runtime, unrelated, other, "unrelated-edge");
    assert!(recovered.matches(&stamp_now(&fixture)));
    assert!(recovered
        .source_revisions_match(
            &fixture.runtime,
            &fixture.runtime.visibility_authority().snapshot()
        )
        .unwrap());
}

#[test]
fn source_native_stamp_compares_after_checkpoint_recovery() {
    let runtime = runtime_with_test_schema();
    let principal = create_entity(&runtime, "principal");
    let role = create_entity(&runtime, "approver");
    let scope = create_entity(&runtime, "payment");
    create_relation(&runtime, principal, role, "principal-role");
    create_relation(&runtime, role, scope, "role-scope");
    let before_evidence = runtime
        .observe_authorization(allow_plan(
            runtime.visibility_authority().snapshot(),
            principal,
            scope,
            [],
        ))
        .unwrap();
    let wire = runtime
        .capture_authorization_durable_dependencies(&before_evidence)
        .unwrap()
        .to_wire_string()
        .unwrap();
    runtime.durability_authority().checkpoint().unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let mut recovered = runtime_with_test_schema();
    recovered.durability_recovery().recover(recovery).unwrap();
    let fresh_evidence = recovered
        .observe_authorization(allow_plan(
            recovered.visibility_authority().snapshot(),
            principal,
            scope,
            [],
        ))
        .unwrap();
    let fresh = recovered
        .capture_authorization_durable_dependencies(&fresh_evidence)
        .unwrap();
    assert!(
        RelationalAuthorizationDurableDependencies::from_wire_string(&wire)
            .unwrap()
            .matches(&fresh)
    );
    assert!(
        RelationalAuthorizationDurableDependencies::from_wire_string(&wire)
            .unwrap()
            .source_revisions_match(&recovered, &recovered.visibility_authority().snapshot())
            .unwrap()
    );
}

#[test]
fn predicate_field_change_back_cannot_revive_old_stamp() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let evidence = fixture
        .runtime
        .observe_authorization(allow_plan(snapshot, fixture.principal, fixture.scope, []))
        .expect("original path is observable");
    let role = evidence.paths()[0]
        .witness()
        .and_then(|witness| witness.entity_at(1))
        .expect("approved role is in the original witness");
    let before = fixture
        .runtime
        .capture_authorization_durable_dependencies(&evidence)
        .expect("exact original provenance");
    update_entity(&fixture.runtime, role, "temporarily-disabled");
    update_entity(&fixture.runtime, role, "approver");
    assert!(!before.matches(&stamp_now(&fixture)));
    assert!(!before
        .source_revisions_match(
            &fixture.runtime,
            &fixture.runtime.visibility_authority().snapshot()
        )
        .unwrap());
}

#[test]
fn adjacency_add_remove_cannot_revive_old_stamp() {
    let fixture = authorization_fixture();
    let before = stamp_now(&fixture);
    let detour = create_entity(&fixture.runtime, "detour");
    let edge = create_relation(&fixture.runtime, fixture.principal, detour, "detour-edge");
    delete_relation_on_branch(&fixture.runtime, edge, BranchId("main".to_owned()));
    assert!(!before.matches(&stamp_now(&fixture)));
    assert!(!before
        .source_revisions_match(
            &fixture.runtime,
            &fixture.runtime.visibility_authority().snapshot()
        )
        .unwrap());
}

#[test]
fn relation_retarget_and_return_cannot_revive_old_stamp() {
    let fixture = authorization_fixture();
    let before = stamp_now(&fixture);
    let alternative = create_entity(&fixture.runtime, "alternative-scope");
    let role = role_for(&fixture);
    retarget(&fixture, role, alternative);
    retarget(&fixture, role, fixture.scope);
    assert!(!before.matches(&stamp_now(&fixture)));
    assert!(!before
        .source_revisions_match(
            &fixture.runtime,
            &fixture.runtime.visibility_authority().snapshot()
        )
        .unwrap());
}

#[test]
fn noncanonical_or_oversized_wire_is_denied() {
    let fixture = authorization_fixture();
    let wire = stamp_now(&fixture).to_wire_string().unwrap();
    assert_eq!(
        RelationalAuthorizationDurableDependencies::from_wire_string(&wire.to_uppercase())
            .unwrap_err(),
        RelationalAuthorizationDependencyDenial::MalformedWire
    );
    assert_eq!(
        RelationalAuthorizationDurableDependencies::from_wire_string(
            &"0".repeat(MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES * 2 + 2)
        )
        .unwrap_err(),
        RelationalAuthorizationDependencyDenial::ByteBudgetExceeded
    );
}

fn stamp_now(fixture: &super::AuthorizationFixture) -> RelationalAuthorizationDurableDependencies {
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let evidence = fixture
        .runtime
        .observe_authorization(allow_plan(snapshot, fixture.principal, fixture.scope, []))
        .expect("the current policy can be observed");
    fixture
        .runtime
        .capture_authorization_durable_dependencies(&evidence)
        .expect("the current exact root has native provenance")
}

fn role_for(fixture: &super::AuthorizationFixture) -> crate::identity::data::EntityId {
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    fixture
        .runtime
        .observe_authorization(allow_plan(snapshot, fixture.principal, fixture.scope, []))
        .unwrap()
        .paths()[0]
        .witness()
        .unwrap()
        .entity_at(1)
        .unwrap()
}

fn retarget(
    fixture: &super::AuthorizationFixture,
    role: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
) {
    let mut transaction =
        crate::tests::support::test_owner_begin_transaction_for_main(&fixture.runtime);
    transaction
        .push_batch(WorkerIntentBatch::new("retarget-approval-path").push(
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: fixture.role_scope_relation,
                    kind_id: RELATION_KIND,
                    source: EntityReference::Existing(role),
                    target: EntityReference::Existing(target),
                },
            )),
        ))
        .unwrap();
    transaction.commit(&fixture.runtime).unwrap();
}
