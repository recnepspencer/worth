//! The existing native fixture schema and its typed correction contribution.
use super::*;

worth_query_declaration::worth_query_application! {
    pub IdentityExecutionSchema {
        owner: "identity_execution_test", version: (1, 0),
        contributions: [IdentityExecutionContribution],
    }
}
worth_query_declaration::worth_query_application_contribution! {
    pub contribution IdentityExecutionContribution in IdentityExecutionSchema {
        identity: "worth.query.test.identity-execution.members.v1",
        corrections: [correction_source::recorded::RecordedLabelEdit],
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
                .aspect(Account::reference(), AccountMembership::reference())
                .field(Account::reference(), AccountMembershipTag::reference())
                .aspect(Account::reference(), AccountAnnotations::reference())
                .field(Account::reference(), AccountAnnotation::reference())
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
            let schema = optional_output_binding::declare(operation_contracts::install(schema))
                .application_mutation_binding::<
                    program_required_binding::ProgramRequiredMutationBinding,
                >()
                .application_mutation_binding::<CapabilityTouchMutationBinding>()

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
                .application_query_binding::<TestAccountSourceBinding>()
                .application_query(application_queries::account_summary_definition())
                .application_query(application_queries::scoped_account_summary_definition())
                .application_query(application_queries::public_scoped_definition())
                .application_query(application_queries::public_membership_definition())
                .application_query(application_queries::cross_root_definition("open"))
                .application_query(application_queries::governed_account_summary_definition())
                .application_query(application_queries::ordered_account_summary_definition())
                .application_query(optional_account_field_query::optional_account_field_definition())
                .application_query(optional_account_owner_query::definition())
                .application_query(owner_members_query::definition())
                .application_query(nested_account::nested_account_definition())
                .application_query(filtered_activity_query::selected_activity_definition())
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
