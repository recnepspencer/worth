use crate::domain_computation::primary_graph::tests::fixture::capability::*;
use crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_command_grant;
use crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_future_replacement_grant;
use crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_grant;
use crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_grant_window;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityElevationScenario {
    Active,
    AlternateCurrentGrant,
    ConflictedApprover,
    DistinctCommandResource,
    ExpiringSupport,
    WrongGrant,
}

pub fn bind_elevated_capability(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    scenario: CapabilityElevationScenario,
) {
    if scenario == CapabilityElevationScenario::ExpiringSupport {
        bind_grant_window(bootstrap, "capability-1", 90, 102, 50);
    } else {
        bind_grant(bootstrap);
    }
    bind_command_grant(
        bootstrap,
        "capability-request-command",
        "principal-0",
        if scenario == CapabilityElevationScenario::DistinctCommandResource {
            "account-2"
        } else {
            "account-1"
        },
        CapabilityAction::RequestElevation,
    );
    bind_command_grant(
        bootstrap,
        "capability-approve-command",
        "principal-1",
        "account-1",
        CapabilityAction::ApproveElevation,
    );
    bind_command_grant(
        bootstrap,
        "capability-revoke-command",
        "principal-1",
        "account-1",
        CapabilityAction::RevokeElevation,
    );
    bind_command_grant(
        bootstrap,
        "capability-review-command",
        "principal-2",
        "account-1",
        CapabilityAction::CompleteReview,
    );
    if scenario == CapabilityElevationScenario::WrongGrant {
        bind_future_replacement_grant(bootstrap);
    }
    if scenario == CapabilityElevationScenario::AlternateCurrentGrant {
        bind_grant_window(bootstrap, "capability-2", 90, 110, 50);
    }
    bootstrap
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                CapabilityElevation::reference(),
                WorthQueryApplicationEntityKey::new("elevation-1").unwrap(),
            )
            .field(
                CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
                "elevation-1".to_owned().into(),
            )
            .field(
                CapabilityElevationReason::at("CapabilityElevationReason"),
                "protect-customer".to_owned(),
            )
            .field(
                CapabilityElevationStatusField::at("CapabilityElevationStatusField"),
                CapabilityElevationStatus::Approved,
            )
            .field(
                CapabilityElevationNotBefore::at("CapabilityElevationNotBefore"),
                95,
            )
            .field(
                CapabilityElevationNotAfter::at("CapabilityElevationNotAfter"),
                105,
            ),
        )
        .unwrap();
    bootstrap
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                CapabilityReview::reference(),
                WorthQueryApplicationEntityKey::new("review-1").unwrap(),
            )
            .field(
                CapabilityReviewIdentity::at("CapabilityReviewIdentity"),
                "review-1".to_owned(),
            )
            .field(
                CapabilityReviewKindField::at("CapabilityReviewKindField"),
                CapabilityReviewKind::Elevation,
            )
            .field(
                CapabilityReviewStatusField::at("CapabilityReviewStatusField"),
                CapabilityReviewStatus::Required,
            ),
        )
        .unwrap();
    bind_relation(
        bootstrap,
        CapabilityElevationRequester::reference(),
        "elevation-requester-1",
        "principal-0",
        "elevation-1",
    );
    bind_relation(
        bootstrap,
        CapabilityElevationApprover::reference(),
        "elevation-approver-1",
        "principal-1",
        "elevation-1",
    );
    bind_relation(
        bootstrap,
        CapabilityElevationGrant::reference(),
        "elevation-grant-1",
        "elevation-1",
        if scenario == CapabilityElevationScenario::WrongGrant {
            "capability-2"
        } else {
            "capability-1"
        },
    );
    bind_relation(
        bootstrap,
        CapabilityElevationResource::reference(),
        "elevation-resource-1",
        "elevation-1",
        "account-1",
    );
    bind_relation(
        bootstrap,
        CapabilityElevationReview::reference(),
        "elevation-review-1",
        "elevation-1",
        "review-1",
    );
    bind_relation(
        bootstrap,
        CapabilityReviewResource::reference(),
        "review-resource-1",
        "review-1",
        "account-1",
    );
    if scenario == CapabilityElevationScenario::ConflictedApprover {
        bind_relation(
            bootstrap,
            CapabilityConflictingBeneficiary::reference(),
            "elevation-approver-conflict-1",
            "principal-1",
            "account-1",
        );
    }
}

pub fn bind_relation<Relation, From, To>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        From,
        To,
    >,
    key: &str,
    from: &str,
    to: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            relation,
            key,
            WorthQueryApplicationEntityKey::new(from).unwrap(),
            WorthQueryApplicationEntityKey::new(to).unwrap(),
        ))
        .unwrap();
}
