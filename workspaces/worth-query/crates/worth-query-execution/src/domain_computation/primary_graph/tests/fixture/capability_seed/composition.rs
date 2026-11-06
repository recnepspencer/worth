use super::super::capability::*;
use super::super::{Account, IdentityExecutionSchema, Principal};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

#[derive(Clone, Copy, Debug)]
pub(in crate::domain_computation) enum CapabilityCompositionScenario {
    Lawful,
    MissingAssignment,
    ExplicitDeny,
    ConflictingBeneficiary,
    RequestActor,
    PriorActor,
    UnrelatedActorRecords,
    AccumulatedProhibitions,
}

pub(in crate::domain_computation::primary_graph) fn bind_composed_grant(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    scenario: CapabilityCompositionScenario,
) {
    super::bind_grant(bootstrap);
    bind_action_records(bootstrap);
    if !matches!(scenario, CapabilityCompositionScenario::MissingAssignment) {
        bind_account_actor(
            bootstrap,
            super::super::AccountOwner::reference(),
            "composed-assignment",
        );
    }
    bind_decision_prohibitions(bootstrap, scenario);
    bind_actor_prohibitions(bootstrap, scenario);
}

fn bind_decision_prohibitions(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    scenario: CapabilityCompositionScenario,
) {
    if matches!(
        scenario,
        CapabilityCompositionScenario::ExplicitDeny
            | CapabilityCompositionScenario::AccumulatedProhibitions
    ) {
        bind_account_actor(
            bootstrap,
            CapabilityExplicitDeny::reference(),
            "composed-explicit-deny",
        );
    }
    if matches!(
        scenario,
        CapabilityCompositionScenario::ConflictingBeneficiary
            | CapabilityCompositionScenario::AccumulatedProhibitions
    ) {
        bind_account_actor(
            bootstrap,
            CapabilityConflictingBeneficiary::reference(),
            "composed-conflicting-beneficiary",
        );
    }
}

fn bind_actor_prohibitions(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    scenario: CapabilityCompositionScenario,
) {
    if matches!(
        scenario,
        CapabilityCompositionScenario::RequestActor
            | CapabilityCompositionScenario::AccumulatedProhibitions
    ) {
        bind_action_actor(
            bootstrap,
            CapabilityRequestActor::reference(),
            "composed-request-actor",
            "selected-request",
        );
    }
    if matches!(
        scenario,
        CapabilityCompositionScenario::PriorActor
            | CapabilityCompositionScenario::AccumulatedProhibitions
    ) {
        bind_action_actor(
            bootstrap,
            CapabilityPriorActor::reference(),
            "composed-prior-actor",
            "selected-prior",
        );
    }
    if matches!(
        scenario,
        CapabilityCompositionScenario::UnrelatedActorRecords
    ) {
        bind_action_actor(
            bootstrap,
            CapabilityRequestActor::reference(),
            "unrelated-request-actor",
            "other-request",
        );
        bind_action_actor(
            bootstrap,
            CapabilityPriorActor::reference(),
            "unrelated-prior-actor",
            "other-prior",
        );
    }
}

fn bind_action_records(bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>) {
    for record in [
        "selected-request",
        "selected-prior",
        "other-request",
        "other-prior",
    ] {
        bootstrap
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    CapabilityActionRecord::reference(),
                    WorthQueryApplicationEntityKey::new(record).unwrap(),
                )
                .field(
                    CapabilityActionRecordIdentity::at("CapabilityActionRecordIdentity"),
                    record.to_owned(),
                ),
            )
            .unwrap();
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                CapabilityActionResource::reference(),
                format!("{record}-resource"),
                WorthQueryApplicationEntityKey::new(record).unwrap(),
                WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            ))
            .unwrap();
    }
}

fn bind_account_actor<Relation>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        Principal,
        Account,
    >,
    key: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            relation,
            key,
            WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
            WorthQueryApplicationEntityKey::new("account-1").unwrap(),
        ))
        .unwrap();
}

fn bind_action_actor<Relation>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        Principal,
        CapabilityActionRecord,
    >,
    key: &str,
    record: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            relation,
            key,
            WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
            WorthQueryApplicationEntityKey::new(record).unwrap(),
        ))
        .unwrap();
}
