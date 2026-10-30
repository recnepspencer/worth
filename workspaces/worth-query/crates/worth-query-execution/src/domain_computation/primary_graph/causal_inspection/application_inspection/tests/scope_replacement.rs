//! A real replacement with the same selector cannot inherit the original subject.
use super::{tests::observe, *};
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, installed_reclaiming_authorization_world, live_scope,
        publish_relational_mutation, Account, AccountIdentity, AccountLabel, AccountMembershipTag,
        AccountOwner, AccountStatus, AuthorizationWorld, PrincipalIdentityField,
        TestAccountSourceBinding,
    },
    WorthQueryPrincipalResolutionMode,
};
use std::{collections::BTreeMap, time::Duration};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::{
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntityReference,
        EntitySpec, MutationIntent, RelationSpec, ReplaceEntityIntent, WorkerIntentBatch,
    },
};

fn account_fields(world: &AuthorizationWorld) -> AspectFieldPatch {
    let layout = world.application.runtime.primary_graph().unwrap().layout();
    let locator = |field: (&str, &str, &str)| {
        layout
            .field_locator(field.0, field.1, field.2)
            .unwrap()
            .clone()
    };
    let identity = AccountIdentity::reference();
    let status = AccountStatus::reference();
    let label = AccountLabel::reference();
    let membership = AccountMembershipTag::reference();
    let fields = [
        (
            (identity.entity(), identity.aspect(), identity.field()),
            "account-1",
        ),
        ((status.entity(), status.aspect(), status.field()), "open"),
        ((label.entity(), label.aspect(), label.field()), "primary"),
        (
            (membership.entity(), membership.aspect(), membership.field()),
            "open",
        ),
    ]
    .into_iter()
    .map(|(field, value)| {
        (
            locator(field),
            StringApplicationValueBinding::encode(&value.to_owned()).unwrap(),
        )
    })
    .collect::<BTreeMap<_, _>>();
    AspectFieldPatch::from(fields)
}

#[test]
fn same_scope_reused_slot_cannot_inherit_original_observation_generation() {
    let world = installed_reclaiming_authorization_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let original = observe(&world, &external, &request, false);
    world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(&external, &request, &original)
        .why_observed()
        .reference_only()
        .execute()
        .unwrap()
        .close();
    let old = original.source.retained_scope_selector().subject();
    let readers = world.application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let publish = |batch| {
        world
            .application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request();
                publish_relational_mutation(&host.borrowed_request_for_test(), &world, batch);
            })
            .unwrap()
    };
    publish(
        WorkerIntentBatch::new("inspection-generation-delete").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: old }),
        )),
    );
    assert_eq!(readers.observe().active(), baseline);
    let reclaimed = world
        .application
        .with_host_advancement(|phase| {
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            world
                .application
                .runtime
                .primary_graph()
                .unwrap()
                .integration_handle()
                .with_runtime(&execution, |native| {
                    native.retention().try_run_pass(&execution)
                })
        })
        .unwrap()
        .expect("actual AiWorkflow maintenance must return its genuine admitted progress");
    eprintln!("actual AiWorkflow retention progress: {reclaimed:?}");
    assert!(
        reclaimed.outcome().entity_reclaimed > 0,
        "an actual reclaim must precede any reuse claim"
    );
    let layout = world.application.runtime.primary_graph().unwrap().layout();
    publish(
        WorkerIntentBatch::new("inspection-generation-create").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: old.partition_id,
                kind_id: layout.entity_kind(Account::reference().name()).unwrap(),
                client_key: ClientKey::raw("inspection-generation-recreated"),
                fields: account_fields(&world),
            }),
        )),
    );
    let (fresh, owner) = world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let fresh = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id();
            let owner = selected
                .resolve_entity(
                    &phase,
                    PrincipalIdentityField::reference(),
                    1_u64,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id();
            (fresh, owner)
        })
        .unwrap();
    assert_eq!(
        fresh.storage_id(),
        old.storage_id(),
        "the actual allocator must issue the same physical slot"
    );
    assert!(
        fresh.generation_value() > old.generation_value(),
        "actual reclaimed allocation must issue a higher generation"
    );
    eprintln!("actual same-slot subject reuse: {old:?} -> {fresh:?}");
    publish(
        WorkerIntentBatch::new("inspection-generation-owner").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: fresh.partition_id,
                kind_id: layout
                    .relation(AccountOwner::reference().name())
                    .unwrap()
                    .kind,
                client_key: ClientKey::raw("inspection-generation-current-owner"),
                source: EntityReference::Existing(owner),
                target: EntityReference::Existing(fresh),
                fields: AspectFieldPatch::default(),
            }),
        )),
    );
    let current_original = observe(&world, &external, &request, false);
    assert_eq!(
        current_original.source.retained_scope_selector().subject(),
        fresh
    );
    world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(
            &external,
            &request,
            &current_original,
        )
        .why_observed()
        .reference_only()
        .execute()
        .unwrap()
        .close();
    let denial = world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(&external, &request, &original)
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .unwrap();
    assert!(
        matches!(
            denial.cause,
            stop::InspectionCause::Admission(
                admission::InspectionAdmissionDenial::ForeignIssuedAccess
            )
        ),
        "a new readable generation cannot borrow original observation identity"
    );
    drop(denial);
    current_original.close();
    original.close();
    assert_eq!(readers.observe().active(), baseline);
}
