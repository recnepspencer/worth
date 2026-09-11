use worth_query_decl::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathBuilder,
    ApplicationSchemaDeclarationBuilder,
};

use crate::model::{CustomerRole, EmployeeRole};
use crate::schema::*;

use super::abilities::*;

pub(crate) fn install_account_ability_policies(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .ability_policy(
            OpenAccount::reference(),
            EmployeeScopePolicy::reference(),
            [employee_path(EmployeeRole::Teller)],
        )
        .ability_policy(
            ServiceInstitutionAccount::reference(),
            EmployeeScopePolicy::reference(),
            [employee_path(EmployeeRole::Teller)],
        )
        .ability_policy(
            AuditInstitution::reference(),
            EmployeeScopePolicy::reference(),
            [employee_path(EmployeeRole::Auditor)],
        )
        .ability_policy(
            DiscoverOwnAccounts::reference(),
            AccountVisibilityPolicy::reference(),
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .allow(Principal::reference()),
            ],
        )
        .ability_policy(
            ViewAccount::reference(),
            AccountVisibilityPolicy::reference(),
            account_view_paths(),
        )
        .ability_policy(
            ViewAccountAccess::reference(),
            AccountVisibilityPolicy::reference(),
            account_management_paths(),
        )
        .ability_policy(
            ManageAccountAccess::reference(),
            AccountMutationScopePolicy::reference(),
            account_management_paths(),
        )
}

pub(crate) fn install_estate_ability_policies(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .ability_policy(
            ViewEstateCase::reference(),
            EstateCapabilityScopePolicy::reference(),
            estate_view_paths(),
        )
        .ability_policy(
            ViewEstateLegalCompliance::reference(),
            EstateCapabilityScopePolicy::reference(),
            estate_governance_paths(),
        )
        .ability_policy(
            ViewEstateMandatoryReview::reference(),
            EstateCapabilityScopePolicy::reference(),
            estate_governance_paths(),
        )
}

pub(crate) fn install_payment_ability_policies(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .ability_policy(
            ViewPayment::reference(),
            AccountVisibilityPolicy::reference(),
            payment_view_paths(),
        )
        .ability_policy(
            SendPersonalFunds::reference(),
            AccountMutationScopePolicy::reference(),
            account_send_paths(),
        )
        .ability_policy(
            InitiateBusinessFunds::reference(),
            AccountMutationScopePolicy::reference(),
            business_paths(CustomerRole::Initiator),
        )
        .ability_policy(
            ApproveBusinessFunds::reference(),
            DistinctApproverPolicy::reference(),
            approval_paths(),
        )
}

fn employee_path(role: EmployeeRole) -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .reverse(AssignmentPrincipal::reference())
        .where_equal(
            AssignmentRole::reference(),
            crate::schema::encoded_bank_value(role),
        )
        .reverse(InstitutionEmployee::reference())
        .allow(Institution::reference())
}

fn estate_view_paths() -> Vec<ApplicationAuthorizationPath> {
    vec![
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(EstateExecutor::reference())
            .allow(EstateCase::reference()),
        estate_assignment_path(EmployeeRole::EstateSpecialist),
    ]
}

fn estate_governance_paths() -> Vec<ApplicationAuthorizationPath> {
    [EmployeeRole::Compliance, EmployeeRole::Legal]
        .map(estate_assignment_path)
        .into()
}

fn estate_assignment_path(role: EmployeeRole) -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .reverse(AssignmentPrincipal::reference())
        .where_equal(
            AssignmentRole::reference(),
            crate::schema::encoded_bank_value(role),
        )
        .forward(EstateAssignment::reference())
        .allow(EstateCase::reference())
}

fn account_view_paths() -> Vec<ApplicationAuthorizationPath> {
    vec![
        personal_owner_path(),
        business_owner_account_path(),
        account_role_path(CustomerRole::PersonalOwner),
        account_role_path(CustomerRole::BusinessOwner),
        account_role_path(CustomerRole::Initiator),
        account_role_path(CustomerRole::Approver),
        account_role_path(CustomerRole::Viewer),
    ]
}

fn account_send_paths() -> Vec<ApplicationAuthorizationPath> {
    vec![
        personal_owner_path(),
        account_role_path(CustomerRole::PersonalOwner),
    ]
}

fn account_management_paths() -> Vec<ApplicationAuthorizationPath> {
    vec![
        personal_owner_path(),
        business_owner_account_path(),
        account_role_path(CustomerRole::BusinessOwner),
    ]
}

fn personal_owner_path() -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(PersonalOwner::reference())
        .allow(Account::reference())
}

fn account_role_path(role: CustomerRole) -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(AccountAuthorizedUser::reference())
        .where_equal(
            AuthorizationRole::reference(),
            crate::schema::encoded_bank_value(role),
        )
        .forward(AuthorizationAccount::reference())
        .allow(Account::reference())
}

fn business_owner_account_path() -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .reverse(BusinessOwner::reference())
        .forward(BusinessAccount::reference())
        .allow(Account::reference())
}

fn business_paths(role: CustomerRole) -> Vec<ApplicationAuthorizationPath> {
    vec![
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .reverse(BusinessOwner::reference())
            .allow(Business::reference()),
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(AccountAuthorizedUser::reference())
            .where_equal(
                AuthorizationRole::reference(),
                crate::schema::encoded_bank_value(role),
            )
            .forward(AuthorizationAccount::reference())
            .reverse(BusinessAccount::reference())
            .allow(Business::reference()),
    ]
}

fn approval_paths() -> Vec<ApplicationAuthorizationPath> {
    vec![
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(AccountAuthorizedUser::reference())
            .where_equal(
                AuthorizationRole::reference(),
                crate::schema::encoded_bank_value(CustomerRole::Approver),
            )
            .forward(AuthorizationAccount::reference())
            .reverse(PaymentSource::reference())
            .allow(PaymentIntent::reference()),
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(PaymentInitiator::reference())
            .deny(PaymentIntent::reference()),
    ]
}

fn payment_view_paths() -> Vec<ApplicationAuthorizationPath> {
    let mut paths = vec![
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(PaymentInitiator::reference())
            .allow(PaymentIntent::reference()),
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(PersonalOwner::reference())
            .reverse(PaymentSource::reference())
            .allow(PaymentIntent::reference()),
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .reverse(BusinessOwner::reference())
            .forward(BusinessAccount::reference())
            .reverse(PaymentSource::reference())
            .allow(PaymentIntent::reference()),
    ];
    paths.extend(
        [
            CustomerRole::PersonalOwner,
            CustomerRole::BusinessOwner,
            CustomerRole::Initiator,
            CustomerRole::Approver,
            CustomerRole::Viewer,
        ]
        .map(|role| {
            ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                .forward(AccountAuthorizedUser::reference())
                .where_equal(
                    AuthorizationRole::reference(),
                    crate::schema::encoded_bank_value(role),
                )
                .forward(AuthorizationAccount::reference())
                .reverse(PaymentSource::reference())
                .allow(PaymentIntent::reference())
        }),
    );
    paths
}
