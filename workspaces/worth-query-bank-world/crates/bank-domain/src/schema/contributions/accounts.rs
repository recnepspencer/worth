use worth_query_decl::facade::{
    application_schema::{
        ApplicationOperationDefinition, ApplicationOperationRef,
        ApplicationSchemaDeclarationBuilder,
    },
    worth_query_application_contribution,
};

use crate::authorization::{
    install_account_ability_policies, AuditInstitution, DiscoverOwnAccounts, ManageAccountAccess,
    OpenAccount, ServiceInstitutionAccount, ViewAccount, ViewAccountAccess,
};

use super::super::{
    authentication::*,
    decision_read_manifest::install_account_decision_reads,
    entities::*,
    fields::*,
    governance::*,
    operations::*,
    program_manifest::{install_account_creation_program, install_authorization_program},
    relations::*,
    BankSchema,
};

worth_query_application_contribution! {
    pub contribution BankAccounts in BankSchema {
        identity: "worth.bank.accounts.v1",
        members: |schema| {
            let schema = install_account_members(schema)
                .principal_binding(BankPrincipalBinding::reference())
                .ability(OpenAccount::reference())
                .ability(DiscoverOwnAccounts::reference())
                .ability(ViewAccount::reference())
                .ability(ViewAccountAccess::reference())
                .ability(ManageAccountAccess::reference())
                .ability(ServiceInstitutionAccount::reference())
                .ability(AuditInstitution::reference())
                .operation(without_external_effect_or_aftermath(CreatePersonalAccountOperation::reference()))
                .application_mutation_binding::<CreatePersonalAccountMutationBinding>()
                .operation(without_external_effect_or_aftermath(CreateBusinessAccountOperation::reference()))
                .operation(without_external_effect_or_aftermath(GrantAccountAuthorizationOperation::reference()))
                .operation(without_external_effect_or_aftermath(RevokeAccountAuthorizationOperation::reference()));
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
                .application_query(crate::queries::account_detail_definition())
                .application_query_binding::<crate::queries::AccountDetailQueryBinding>()
                .application_query(crate::queries::account_summary_definition())
                .application_query_binding::<crate::queries::AccountSummaryQueryBinding>()
                .application_query(crate::queries::account_activity_definition())
                .application_query_binding::<crate::queries::AccountActivityQueryBinding>()
                .application_query(crate::queries::institution_audit_definition())
                .application_query_binding::<crate::queries::InstitutionAuditQueryBinding>();
            install_account_operation_abilities(install_account_ability_policies(schema))
        }
    }
}

fn install_account_members(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .entity(Institution::reference())
        .entity(ExternalPrincipalMapping::reference())
        .entity(Principal::reference())
        .entity(Customer::reference())
        .entity(Business::reference())
        .entity(Account::reference())
        .entity(AccountAuthorization::reference())
        .entity(EmployeeAssignment::reference())
        .aspect(
            ExternalPrincipalMapping::reference(),
            ExternalPrincipalIdentity::reference(),
        )
        .aspect(Principal::reference(), PrincipalIdentity::reference())
        .aspect(Account::reference(), Identity::reference())
        .aspect(Institution::reference(), InstitutionIdentity::reference())
        .aspect(Business::reference(), BusinessIdentity::reference())
        .aspect(Account::reference(), AccountProfile::reference())
        .aspect(Account::reference(), AccountState::reference())
        .aspect(
            AccountAuthorization::reference(),
            AuthorizationScope::reference(),
        )
        .aspect(
            AccountAuthorization::reference(),
            AuthorizationIdentity::reference(),
        )
        .aspect(EmployeeAssignment::reference(), EmployeeScope::reference())
        .field(
            ExternalPrincipalMapping::reference(),
            ExternalIdentityKey::reference(),
        )
        .field(Principal::reference(), PrincipalIdentityField::reference())
        .field(
            ExternalPrincipalMapping::reference(),
            ExternalMappingStatus::reference(),
        )
        .field(Account::reference(), AccountIdentity::reference())
        .field(
            Institution::reference(),
            InstitutionIdentityField::reference(),
        )
        .field(Business::reference(), BusinessIdentityField::reference())
        .field(Account::reference(), AccountDisplayName::reference())
        .field(Account::reference(), Kind::reference())
        .field(Account::reference(), AccountingRevision::reference())
        .field(Account::reference(), Status::reference())
        .field(
            AccountAuthorization::reference(),
            AuthorizationRole::reference(),
        )
        .field(
            AccountAuthorization::reference(),
            AccountAuthorizationIdentity::reference(),
        )
        .field(
            EmployeeAssignment::reference(),
            EmployeeAssignmentIdentityField::reference(),
        )
        .field(EmployeeAssignment::reference(), AssignmentRole::reference())
        .relation(
            ExternalPrincipal::reference(),
            ExternalPrincipalMapping::reference(),
            Principal::reference(),
        )
        .relation(
            PrincipalCustomer::reference(),
            Principal::reference(),
            Customer::reference(),
        )
        .relation(
            PersonalOwner::reference(),
            Principal::reference(),
            Account::reference(),
        )
        .relation(
            BusinessOwner::reference(),
            Business::reference(),
            Principal::reference(),
        )
        .relation(
            BusinessAccount::reference(),
            Business::reference(),
            Account::reference(),
        )
        .relation(
            AccountAuthorizedUser::reference(),
            Principal::reference(),
            AccountAuthorization::reference(),
        )
        .relation(
            AuthorizationAccount::reference(),
            AccountAuthorization::reference(),
            Account::reference(),
        )
        .relation(
            InstitutionEmployee::reference(),
            Institution::reference(),
            EmployeeAssignment::reference(),
        )
        .relation(
            AssignmentPrincipal::reference(),
            EmployeeAssignment::reference(),
            Principal::reference(),
        )
        .relation(
            InstitutionAccount::reference(),
            Institution::reference(),
            Account::reference(),
        )
        .relation(
            InstitutionCashAccount::reference(),
            Institution::reference(),
            Account::reference(),
        )
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
