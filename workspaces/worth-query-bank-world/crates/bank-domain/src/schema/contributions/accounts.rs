use worth_query_decl::facade::application_schema::{
    ApplicationOperationDefinition, ApplicationOperationRef, ApplicationSchemaDeclarationBuilder,
};

use crate::authorization::{
    install_account_ability_policies, AuditInstitution, DiscoverOwnAccounts, ManageAccountAccess,
    OpenAccount, ServiceInstitutionAccount, ViewAccount, ViewAccountAccess,
};

use super::super::{
    authentication::*,
    decision_read_manifest::install_account_decision_reads,
    governance::*,
    operations::*,
    program_manifest::{install_account_creation_program, install_authorization_program},
    BankSchema,
};

use worth_query_decl::facade::application_schema::{fragment::*, *};
pub struct BankAccounts;
impl ApplicationSchemaContribution<BankSchema> for BankAccounts {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.bank.accounts.v1");
    type Members = (crate::schema::AccountRecords, Contracts);
}
impl BankAccounts {
    pub const fn reference() -> ApplicationSchemaContributionRef<BankSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<BankSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<BankSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok({
            let schema = install_account_members(schema)
                .principal_binding(BankPrincipalBinding::reference())
                .ability(OpenAccount::reference())
                .ability(DiscoverOwnAccounts::reference())
                .ability(ViewAccount::reference())
                .ability(ViewAccountAccess::reference())
                .ability(ManageAccountAccess::reference())
                .ability(ServiceInstitutionAccount::reference())
                .ability(AuditInstitution::reference())
                .operation(without_external_effect_or_aftermath(
                    CreatePersonalAccountOperation::reference(),
                ))
                .application_mutation_binding::<CreatePersonalAccountMutationBinding>()
                .operation(without_external_effect_or_aftermath(
                    CreateBusinessAccountOperation::reference(),
                ))
                .application_mutation_binding::<CreateBusinessAccountMutationBinding>()
                .operation(without_external_effect_or_aftermath(
                    GrantAccountAuthorizationOperation::reference(),
                ))
                .application_mutation_binding::<GrantAccountAccessMutationBinding>()
                .operation(without_external_effect_or_aftermath(
                    RevokeAccountAuthorizationOperation::reference(),
                ))
                .application_mutation_binding::<RevokeAccountAccessMutationBinding>();
            let schema = install_account_creation_program(schema);
            let schema = install_authorization_program(schema);
            let schema = install_account_decision_reads(schema)
                .policy(AccountVisibilityPolicy::reference())
                .policy(AccountMutationScopePolicy::reference())
                .policy(EmployeeScopePolicy::reference())
                .application_query(crate::queries::account_authorized_users_definition())
                .application_query_binding::<crate::queries::AccountAuthorizedUsersQueryBinding>()
                .application_query(crate::queries::account_discovery_definition())
                .application_query_binding::<crate::queries::AccountDiscoveryQueryBinding>()
                .application_query(crate::queries::current_accounts_definition())
                .application_query_binding::<crate::queries::CurrentAccountsQueryBinding>()
                .managed_index(worth_query_decl::facade::application_query::ApplicationManagedIndexDefinition::<BankSchema, crate::queries::CurrentAccountsIndex>::new())
                .application_query(crate::queries::account_detail_definition())
                .application_query_binding::<crate::queries::AccountDetailQueryBinding>()
                .application_query(crate::queries::account_summary_definition())
                .application_query_binding::<crate::queries::AccountSummaryQueryBinding>()
                .application_query(crate::queries::account_activity_definition())
                .application_query_binding::<crate::queries::AccountActivityQueryBinding>()
                .application_query(crate::queries::institution_audit_definition())
                .application_query_binding::<crate::queries::InstitutionAuditQueryBinding>();
            super::super::program_deployment::install(install_account_operation_abilities(
                install_account_ability_policies(schema),
            ))
        })
    }
}

fn install_account_members(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
}

fn install_account_operation_abilities(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_requires_ability(
            CreatePersonalAccountOperation::reference(),
            OpenAccount::reference(),
        )
        .operation_requires_ability(
            CreateBusinessAccountOperation::reference(),
            OpenAccount::reference(),
        )
        .operation_requires_ability(
            GrantAccountAuthorizationOperation::reference(),
            ManageAccountAccess::reference(),
        )
        .operation_requires_ability(
            RevokeAccountAuthorizationOperation::reference(),
            ManageAccountAccess::reference(),
        )
}

fn without_external_effect_or_aftermath<Operation, Input>(
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
) -> ApplicationOperationDefinition<BankSchema, Operation, Input> {
    operation
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
}
