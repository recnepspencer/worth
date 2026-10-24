#[path = "fixture/own_write_computation.rs"]
pub(in crate::domain_computation::primary_graph) mod own_write_computation;
#[path = "fixture/retained_output_capacity.rs"]
pub(in crate::domain_computation::primary_graph) mod retained_output_capacity;
use std::time::{Duration, Instant};

#[path = "fixture/authentication.rs"]
mod authentication;
pub(in crate::domain_computation::primary_graph) use authentication::authenticate_external;
#[path = "fixture/account_seed.rs"]
mod account_seed;
#[path = "fixture/authorization_time.rs"]
mod authorization_time;
#[path = "fixture/authorization_world_installation.rs"]
mod authorization_world_installation;
#[path = "fixture/commit_snapshot_closeout.rs"]
mod commit_snapshot_closeout;
pub(in crate::domain_computation::primary_graph) use commit_snapshot_closeout::release_test_commit_snapshot;
#[path = "fixture/installed_layout.rs"]
mod installed_layout;
pub(in crate::domain_computation::primary_graph) use installed_layout::installed_layout;
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
    elevated_account_activity_parameters, ApproveCapabilityElevationOperation,
    ApproveElevationCapability, ApproveElevationInput, CapabilityAction, CapabilityActionBinding,
    CapabilityDisclosure, CapabilityDisclosureBinding, CapabilityElevationApprover,
    CapabilityElevationGrant, CapabilityElevationIdentity, CapabilityElevationNotAfter,
    CapabilityElevationNotBefore, CapabilityElevationReason, CapabilityElevationRequester,
    CapabilityElevationResource, CapabilityElevationReview, CapabilityElevationStatus,
    CapabilityElevationStatusBinding, CapabilityElevationStatusField, CapabilityIdentity,
    CapabilityPurpose, CapabilityPurposeBinding, CapabilityRequestContext,
    CapabilityReviewIdentity, CapabilityReviewKindField, CapabilityReviewResource,
    CapabilityReviewStatus, CapabilityReviewStatusBinding, CapabilityReviewStatusField,
    CapabilityReviewer, CapabilityStatus, CapabilityStatusBinding, CapabilityStatusField,
    CapabilityTouchInput, CapabilityTouchOperation, CloseElevationInput,
    CompleteCapabilityReviewOperation, CompleteElevationReviewCapability,
    CompleteElevationReviewInput, ElevatedAccountActivityCause, ElevatedAccountActivityQuery,
    ElevatedAccountActivityResult, ElevatedCapabilityTouchInput, ElevatedCapabilityTouchOperation,
    ElevatedTouchAccountCapability, RequestCapabilityElevationOperation,
    RequestElevationCapability, RequestElevationInput, RevokeCapabilityElevationOperation,
    RevokeElevationCapability, TouchAccountCapability,
};
pub(in crate::domain_computation::primary_graph) use capability_status_mutation::revoke_current_capability;
#[path = "fixture/application_queries.rs"]
mod application_queries;
#[path = "fixture/current_output_source.rs"]
mod current_output_source;
#[path = "fixture/filtered_activity_query.rs"]
mod filtered_activity_query;
pub(super) use application_queries::{
    cross_root_definition, AccountSummaryQuery, GovernedAccountSummaryQuery,
    OrderedAccountSummaryQuery, PublicAccountMembershipQuery, PublicAccountMembershipResult,
    ScopedAccountSummaryQuery,
};
pub(in crate::domain_computation::primary_graph) use application_queries::{
    status_parameter, AccountSummaryParameters, AccountSummaryResult, CrossRootQuery,
    PublicScopedAccountSummaryQuery,
};
pub(in crate::domain_computation::primary_graph) use current_output_source::TestAccountSourceBinding;
pub(super) use filtered_activity_query::{
    selected_activity_parameters, SelectedActivityParameters, SelectedActivityQuery,
    SelectedActivityResult,
};
#[path = "fixture/optional_account_field_query.rs"]
mod optional_account_field_query;
pub(super) use optional_account_field_query::{
    OptionalAccountFieldQuery, OptionalAccountFieldResult,
};
#[path = "fixture/optional_account_owner_query.rs"]
mod optional_account_owner_query;
#[path = "fixture/owner_members_query.rs"]
mod owner_members_query;
pub(super) use optional_account_owner_query::{
    OptionalAccountOwnerQuery, OptionalAccountOwnerResult,
};
pub(super) use owner_members_query::{OwnerMembersQuery, OwnerMembersResult};
#[path = "fixture/nested_account.rs"]
mod nested_account;
pub(in crate::domain_computation::primary_graph) use nested_account::NestedAccountQuery;
pub(super) use nested_account::NestedAccountResult;
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
#[path = "fixture/capability_touch_binding.rs"]
mod capability_touch_binding;
#[path = "fixture/handler_installation.rs"]
mod handler_installation;
#[path = "fixture/operation_contracts.rs"]
mod operation_contracts;
#[path = "fixture/optional_output_binding.rs"]
mod optional_output_binding;
pub(in crate::domain_computation::primary_graph) use optional_output_binding::{
    OptionalCompanion, OptionalOutputInput, OptionalOutputMutationBinding, OptionalOutputOperation,
    OptionalOutputPlan, OptionalOutputs, OptionalSubject,
};
#[path = "fixture/program_required_binding.rs"]
mod program_required_binding;
pub(super) use capability_touch_binding::CapabilityTouchMutationBinding;
pub(in crate::domain_computation::primary_graph) use program_required_binding::{
    ProgramRequiredInput, ProgramRequiredMutationBinding,
};
pub(super) use program_required_binding::{
    ProgramRequiredOperation, ProgramRequiredSiblingBinding,
};
#[path = "fixture/program_activation.rs"]
mod program_activation;
#[path = "fixture/program_roster.rs"]
mod program_roster;
pub(in crate::domain_computation::primary_graph) use program_activation::seed_program_activation;
pub(in crate::domain_computation::primary_graph) use program_roster::{
    installed_program_support, installed_provenance_support, rostered_program_revision,
    unadmitted_program_revision,
};
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
pub(in crate::domain_computation) use world_installation::installed_authorization_world;
pub(in crate::domain_computation::primary_graph) use world_installation::{
    installed_authorization_world_with_label, installed_authorization_world_with_product_resources,
    installed_authorization_world_with_resource_profile, installed_blocked_authorization_world,
    installed_two_principal_authorization_world,
};
pub(super) use world_installation::{
    installed_world, installed_world_with_policy_fact, restored_world,
};

use worth_query_admission::facade::authenticated_principal::*;
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_declaration::{
    worth_query_ability, worth_query_aspect, worth_query_effect, worth_query_entity,
    worth_query_field, worth_query_operation, worth_query_operation_emits,
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

#[path = "fixture/schema_declaration.rs"]
mod schema_declaration;
pub(in crate::domain_computation) use schema_declaration::IdentityExecutionSchema;
mod advancement_host;
#[path = "fixture/correction_source.rs"]
pub(in crate::domain_computation::primary_graph) mod correction_source;
pub(crate) use advancement_host::AdvancementHost;
mod reconstruction_execution;
pub(in crate::domain_computation::primary_graph) use reconstruction_execution::isolated_request_owner;
