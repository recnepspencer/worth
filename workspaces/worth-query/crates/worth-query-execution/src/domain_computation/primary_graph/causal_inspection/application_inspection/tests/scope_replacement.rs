//! A real replacement with the same selector cannot inherit the original subject.
use super::{tests::observe, *};
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, live_scope, publish_relational_mutation, Account,
        AccountIdentity, AccountLabel, AccountMembershipTag, AccountStatus,
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
        AspectFieldPatch, EntityMutationIntent, EntitySpec, MutationIntent, ReplaceEntityIntent,
        WorkerIntentBatch,
    },
};

#[test]
fn equal_scope_selector_cannot_replace_original_full_subject_identity() {
    let world = installed_authorization_world(true);
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
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host.borrowed_request_for_test();
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
            publish_relational_mutation(
                &execution,
                &world,
                WorkerIntentBatch::new("inspection-scope-replacement").push(
                    MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
                        entity_id: old,
                        replacement: EntitySpec {
                            partition_id: old.partition_id,
                            kind_id: layout.entity_kind(Account::reference().name()).unwrap(),
                            client_key: ClientKey::raw("inspection-replacement"),
                            fields: AspectFieldPatch::from(fields),
                        },
                    })),
                ),
            );
        })
        .unwrap();
    let fresh = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&phase.test_host_request())
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
        })
        .unwrap();
    assert_eq!(fresh.partition_id, old.partition_id);
    assert_ne!(
        fresh, old,
        "canonical replacement must produce a distinct actual full identity"
    );
    eprintln!("actual replaced subject: {old:?} -> {fresh:?}");
    if fresh.storage_id() == old.storage_id() {
        assert_ne!(
            fresh.generation, old.generation,
            "actual physical reuse must change generation"
        );
    }
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
        "equal selector must refuse before replacement READ authorization"
    );
    drop(denial);
    assert_eq!(readers.observe().active(), baseline);
}
