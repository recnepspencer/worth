use std::time::{Duration, Instant};

#[path = "fixture/authentication.rs"]
mod authentication;
use authentication::authenticate_external;
#[path = "fixture/account_seed.rs"]
mod account_seed;
#[path = "fixture/authorization_time.rs"]
mod authorization_time;
#[path = "fixture/authorization_world_installation.rs"]
mod authorization_world_installation;
#[path = "fixture/commit_snapshot_closeout.rs"]
mod commit_snapshot_closeout;
pub(in crate::domain_computation::primary_graph) use commit_snapshot_closeout::release_test_commit_snapshot;
#[path = "fixture/product_publication.rs"]
mod product_publication;
pub(in crate::domain_computation::primary_graph) use product_publication::{
    prepare_relational_mutation_on_application, publish_relational_mutation,
    publish_relational_mutation_on_application,
};
#[path = "fixture/capability.rs"]
pub(in crate::domain_computation) mod capability;
#[path = "fixture/capability_access_fixture.rs"]
mod capability_access_fixture;
#[path = "fixture/capability_seed.rs"]
mod capability_seed;
pub(in crate::domain_computation) use capability_seed::CapabilityCompositionScenario;
#[path = "fixture/capability_elevation_seed.rs"]
mod capability_elevation_seed;
#[path = "fixture/capability_population_seed.rs"]
mod capability_population_seed;
pub(in crate::domain_computation::primary_graph) use capability_elevation_seed::CapabilityElevationScenario;
#[path = "fixture/capability_world_installation.rs"]
mod capability_world_installation;
pub(in crate::domain_computation::primary_graph) use capability_access_fixture::admit_touch_account_capability;
#[path = "fixture/capability_status_mutation.rs"]
mod capability_status_mutation;
pub(super) use capability::{
    canonical_governed_input_materialization_count, elevated_account_activity_parameters,
    ApproveCapabilityElevationOperation, ApproveElevationCapability, ApproveElevationInput,
    CapabilityAction, CapabilityDisclosure, CapabilityElevationApprover, CapabilityElevationGrant,
    CapabilityElevationIdentity, CapabilityElevationNotAfter, CapabilityElevationNotBefore,
    CapabilityElevationReason, CapabilityElevationRequester, CapabilityElevationResource,
    CapabilityElevationReview, CapabilityElevationStatus, CapabilityElevationStatusField,
    CapabilityGovernedInputIdentity, CapabilityIdentity, CapabilityPurpose,
    CapabilityRequestContext, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewStatus, CapabilityReviewStatusField,
    CapabilityReviewer, CapabilityStatus, CapabilityStatusField, CapabilityTouchInput,
    CapabilityTouchOperation, CloseElevationInput, CompleteCapabilityReviewOperation,
    CompleteElevationReviewCapability, CompleteElevationReviewInput, ElevatedAccountActivityCause,
    ElevatedAccountActivityQuery, ElevatedAccountActivityResult, ElevatedCapabilityTouchInput,
    ElevatedCapabilityTouchOperation, ElevatedTouchAccountCapability,
    RequestCapabilityElevationOperation, RequestElevationCapability, RequestElevationInput,
    RevokeCapabilityElevationOperation, RevokeElevationCapability, TouchAccountCapability,
};
pub(in crate::domain_computation::primary_graph) use capability_status_mutation::revoke_current_capability;
#[path = "fixture/application_queries.rs"]
mod application_queries;
pub(in crate::domain_computation::primary_graph) use application_queries::AccountSummaryParameters;
pub(super) use application_queries::{
    cross_root_definition, status_parameter, AccountSummaryQuery, AccountSummaryResult,
    CrossRootQuery, GovernedAccountSummaryQuery, OrderedAccountSummaryQuery,
    ScopedAccountSummaryQuery,
};
#[path = "fixture/optional_account_field_query.rs"]
mod optional_account_field_query;
pub(super) use optional_account_field_query::{
    OptionalAccountFieldQuery, OptionalAccountFieldResult,
};
#[path = "fixture/nested_account.rs"]
mod nested_account;
pub(super) use nested_account::{NestedAccountQuery, NestedAccountResult};
#[path = "fixture/forged_selector.rs"]
mod forged_selector;
pub(super) use forged_selector::{ForgedSelectorQuery, ForgedSelectorResult};
#[path = "fixture/live_account_query.rs"]
mod live_account_query;
pub(in crate::domain_computation::primary_graph) use live_account_query::{
    live_account_parameters, LiveAccountActivityCause, LiveAccountActivityQuery,
    LiveAccountActivityResult, LiveActivityEffect, LiveActivityEvent,
};
#[path = "fixture/governed_live_query.rs"]
mod governed_live_query;
pub(in crate::domain_computation::primary_graph) use governed_live_query::{
    governed_live_account_parameters, GovernedLiveAccountActivityCause,
    GovernedLiveAccountActivityQuery, GovernedLiveAccountActivityResult,
};
#[path = "fixture/governed_root_guard_query.rs"]
mod governed_root_guard_query;
pub(in crate::domain_computation::primary_graph) use governed_root_guard_query::{
    ForbiddenRootGuardQuery, GovernedRootGuardQuery,
};
#[path = "fixture/governed_omission_query.rs"]
mod governed_omission_query;
pub(in crate::domain_computation::primary_graph) use governed_omission_query::{
    GovernedAccountOmissionQuery, GovernedAccountOmissionResult,
};
#[path = "fixture/governed_hidden_ordering_query.rs"]
mod governed_hidden_ordering_query;
pub(in crate::domain_computation::primary_graph) use governed_hidden_ordering_query::GovernedHiddenOrderingQuery;
#[path = "fixture/forbidden_hidden_ordering_query.rs"]
mod forbidden_hidden_ordering_query;
pub(in crate::domain_computation::primary_graph) use forbidden_hidden_ordering_query::ForbiddenHiddenOrderingQuery;
#[path = "fixture/forbidden_live_identity_queries.rs"]
mod forbidden_live_identity_queries;
pub(in crate::domain_computation::primary_graph) use forbidden_live_identity_queries::{
    forbidden_live_identity_parameters, ForbiddenLiveScopeIdentityQuery,
    ForbiddenLiveTargetIdentityQuery,
};
#[path = "fixture/invalid_disclosure_queries.rs"]
mod invalid_disclosure_queries;
pub(super) use invalid_disclosure_queries::{
    ForbiddenInfluenceQuery, IncompleteDisclosureQuery, ResultRulePredicateQuery,
};
#[path = "fixture/operation_contracts.rs"]
mod operation_contracts;
#[path = "fixture/schema_types.rs"]
mod schema_types;
#[path = "fixture/world_authentication.rs"]
mod world_authentication;
#[path = "fixture/world_installation.rs"]
mod world_installation;
pub(in crate::domain_computation::primary_graph) use authorization_world_installation::AuthorizationWorld;
pub(in crate::domain_computation) use capability_world_installation::installed_composed_capability_world;
pub(in crate::domain_computation::primary_graph) use capability_world_installation::{
    installed_capability_authorization_world, installed_capability_live_world,
    installed_capability_live_world_with_label, installed_capability_replacement_world,
    installed_capability_world_with_exact_pair_population, installed_capability_world_with_label,
    installed_capability_world_with_same_resource_unrelated, installed_delegated_capability_world,
    installed_delegated_capability_world_at_depth,
    installed_delegated_capability_world_with_unrelated, installed_elevated_capability_live_world,
    installed_elevated_capability_world,
};
pub(in crate::domain_computation::primary_graph) use schema_types::*;
pub(in crate::domain_computation::primary_graph) use world_installation::{
    installed_authorization_world, installed_authorization_world_with_label,
    installed_authorization_world_with_resource_profile, installed_blocked_authorization_world,
};
pub(super) use world_installation::{
    installed_two_principal_authorization_world, installed_world, installed_world_with_policy_fact,
};

use worth_query_admission::facade::authenticated_principal::*;
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_declaration::{
    worth_query_ability, worth_query_application_schema, worth_query_aspect, worth_query_effect,
    worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_emits,
    worth_query_operation_expects_fact, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_requires, worth_query_operation_unlinks, worth_query_operation_writes,
    worth_query_policy, worth_query_principal_binding, worth_query_relation,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledPrincipalBinding,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryValidatedPortableDomainPackage,
};

use authorization_time::AuthorizationTimeController;

use crate::domain_computation::execution_runtime::{
    WorthQueryApplicationQueryResourceProfile, WorthQueryExecutionRuntime,
    WorthQueryExecutionRuntimeInstaller,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationPrincipalKey;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphApplicationRuntime,
};

worth_query_application_schema! {
    pub schema IdentityExecutionSchema {
        owner: identity_execution_test,
        version: (1, 0),
        members: |schema| {
            let schema = schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(Account::reference())
                .entity(Activity::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .aspect(Account::reference(), AccountPolicy::reference())
                .field(Account::reference(), AccountIdentity::reference())
                .field(Account::reference(), AccountStatus::reference())
                .field(Account::reference(), AccountLabel::reference())
                .field(Account::reference(), AccountNote::reference())
                .field(Account::reference(), AccountScore::reference())
                .aspect(Activity::reference(), ActivityFacts::reference())
                .field(Activity::reference(), ActivityIdentity::reference())
                .field(Activity::reference(), ActivitySequence::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Principal::reference(),
                )
                .relation(
                    AccountOwner::reference(),
                    Principal::reference(),
                    Account::reference(),
                )
                .relation(
                    AccountBlocked::reference(),
                    Principal::reference(),
                    Account::reference(),
                )
                .relation(
                    AccountPrimaryActivity::reference(),
                    Account::reference(),
                    Activity::reference(),
                )
                .relation(
                    AccountSecondaryActivity::reference(),
                    Account::reference(),
                    Activity::reference(),
                )
                .relation(
                    AccountAllActivity::reference(),
                    Account::reference(),
                    Activity::reference(),
                )
                .relation(
                    ActivityAccount::reference(),
                    Activity::reference(),
                    Account::reference(),
                )
                .principal_binding(IdentityBinding::reference())
                .ability(ViewAccount::reference())
                .ability(EditAccount::reference())
                .ability(ManageOwnership::reference())
                .effect(AccountActivityEffect::reference())
                .effect(RetainedStatusEffect::reference())
                .effect(MutationFreeExternalEffect::reference())
                .effect(LiveActivityEffect::reference());
            let schema = operation_contracts::install(schema)
                .policy(AccountAccessPolicy::reference())
                .ability_policy(
                    ViewAccount::reference(),
                    AccountAccessPolicy::reference(),
                    [
                        worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder::from_principal(
                            Principal::reference(),
                        )
                        .forward(AccountOwner::reference())
                        .allow(Account::reference()),
                        worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder::from_principal(
                            Principal::reference(),
                        )
                        .forward(AccountBlocked::reference())
                        .deny(Account::reference()),
                    ],
                )
                .ability_policy(
                    EditAccount::reference(),
                    AccountAccessPolicy::reference(),
                    [
                        worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder::from_principal(
                            Principal::reference(),
                        )
                        .forward(AccountOwner::reference())
                        .allow(Account::reference()),
                        worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder::from_principal(
                            Principal::reference(),
                        )
                        .forward(AccountBlocked::reference())
                        .deny(Account::reference()),
                    ],
                )
                .ability_policy(
                    ManageOwnership::reference(),
                    AccountAccessPolicy::reference(),
                    [worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder::from_principal(
                        Principal::reference(),
                    )
                    .allow(Principal::reference())],
                )
                .application_query(application_queries::account_summary_definition())
                .application_query(application_queries::scoped_account_summary_definition())
                .application_query(application_queries::cross_root_definition("open"))
                .application_query(application_queries::governed_account_summary_definition())
                .application_query(application_queries::ordered_account_summary_definition())
                .application_query(optional_account_field_query::optional_account_field_definition())
                .application_query(nested_account::nested_account_definition())
                .application_query(forged_selector::forged_selector_definition())
                .application_query(live_account_query::live_account_activity_definition())
                .application_query(governed_live_query::governed_live_account_definition())
                .application_query(capability::elevated_account_activity_definition())
                .application_query(governed_omission_query::governed_account_omission_definition())
                .application_query(
                    governed_hidden_ordering_query::governed_hidden_ordering_definition(),
                )
                .application_query(
                    forbidden_hidden_ordering_query::forbidden_hidden_ordering_definition(),
                )
                .application_query(
                    forbidden_live_identity_queries::forbidden_live_scope_identity_definition(),
                )
                .application_query(
                    forbidden_live_identity_queries::forbidden_live_target_identity_definition(),
                )
                .application_query(governed_root_guard_query::governed_root_guard_definition())
                .application_query(governed_root_guard_query::forbidden_root_guard_definition())
                .application_query(invalid_disclosure_queries::incomplete_disclosure_definition())
                .application_query(invalid_disclosure_queries::forbidden_influence_definition())
                .application_query(
                    invalid_disclosure_queries::result_rule_predicate_definition(),
                );
            capability::install(schema)
        }
    }
}
