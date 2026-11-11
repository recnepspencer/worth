//! The existing native fixture schema and its typed correction contribution.
use super::*;

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct IdentityExecutionSchema;
impl Fragment for IdentityExecutionSchema {
    const NAME: &'static str = "IdentityExecutionSchema";
    type Bindings = NoBindings;
    type Members = (IdentityExecutionContribution,);
}
impl ApplicationSchema for IdentityExecutionSchema {
    const OWNER: &'static str = "identity_execution_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_declaration::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl IdentityExecutionSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for IdentityExecutionSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct IdentityExecutionContribution;
impl ApplicationSchemaContribution<IdentityExecutionSchema> for IdentityExecutionContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(
            "worth.query.test.identity-execution.members.v1",
        );
    type Members = (
        ExecutionRecords,
        PrincipalRecords,
        GrantRecords,
        ElevationRecords,
        Contracts,
        correction_source::recorded::RecordedLabelEdit,
    );
}
impl IdentityExecutionContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<IdentityExecutionSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<IdentityExecutionSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = schema
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
                .application_query_binding::<AlternateAccountSourceBinding>()
                .application_query_inspection(
                    worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition::<IdentityExecutionSchema, TestAccountSourceBinding>::new(
                        worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract::public(),
                    ).unwrap(),
                )
                .application_query_inspection(
                    inspection_disclosure::governed(),
                )
                .application_query_binding::<diagnostic_comparison::DiagnosticAccountBinding>()
                .application_query_binding::<diagnostic_comparison::GovernedDiagnosticAccountBinding>()
                .application_query_binding::<diagnostic_comparison::DeniedDiagnosticInternalBinding>()
                .application_query_binding::<diagnostic_comparison::DeniedDiagnosticRelationBinding>()
                .application_query_binding::<diagnostic_comparison::DeniedDiagnosticFieldBinding>()
                .application_query_inspection(worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition::<IdentityExecutionSchema, diagnostic_comparison::DiagnosticAccountBinding>::new(worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract::public()).unwrap())
                .application_query_inspection(worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition::<IdentityExecutionSchema, diagnostic_comparison::DeniedDiagnosticInternalBinding>::new(worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract::public()).unwrap())
                .application_query_inspection(worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition::<IdentityExecutionSchema, diagnostic_comparison::DeniedDiagnosticRelationBinding>::new(worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract::public()).unwrap())
                .application_query_inspection(worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition::<IdentityExecutionSchema, diagnostic_comparison::DeniedDiagnosticFieldBinding>::new(worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract::public()).unwrap())
                .application_query_inspection(diagnostic_comparison::governed_metadata())
                .application_query(diagnostic_comparison::definition(diagnostic_comparison::DiagnosticAccountQuery::reference(), diagnostic_comparison::DiagnosticPolicy::Complete))
                .application_query(diagnostic_comparison::definition(diagnostic_comparison::DeniedDiagnosticInternalQuery::reference(), diagnostic_comparison::DiagnosticPolicy::DenyInternal))
                .application_query(diagnostic_comparison::definition(diagnostic_comparison::DeniedDiagnosticRelationQuery::reference(), diagnostic_comparison::DiagnosticPolicy::DenyRelation))
                .application_query(diagnostic_comparison::definition(diagnostic_comparison::DeniedDiagnosticFieldQuery::reference(), diagnostic_comparison::DiagnosticPolicy::DenyField))
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
        })
    }
}
