use worth_query_decl::facade::{
    application_schema::ApplicationSchemaDeclarationBuilder, worth_query_operation_creates,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_writes,
};

use crate::schema::{
    AccountAuthorizationIdentity, AccountAuthorizedUser, AuthorizationAccount, AuthorizationRole,
    BankSchema, InitiateBusinessPaymentOperation,
};

use super::{
    ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantAction,
    ApprovedBusinessPaymentGrantDelegationLimit, ApprovedBusinessPaymentGrantGrantee,
    ApprovedBusinessPaymentGrantGrantor, ApprovedBusinessPaymentGrantNotAfter,
    ApprovedBusinessPaymentGrantNotBefore, ApprovedBusinessPaymentGrantPurpose,
    ApprovedBusinessPaymentGrantResource, ApprovedBusinessPaymentGrantStatus,
    ApprovedBusinessPaymentGrantWorkflow,
};

// Initiating a business payment grants its approval workflow to every
// Approver on the source account, so the initiation reads the account's
// authorizations and authors one grant per approver.

worth_query_operation_reads!(
    InitiateBusinessPaymentOperation => [
        AccountAuthorizedUser,
        AuthorizationAccount,
        AccountAuthorizationIdentity,
        AuthorizationRole
    ]
);
worth_query_operation_creates!(InitiateBusinessPaymentOperation => [ApprovedBusinessPaymentGrant]);
worth_query_operation_writes!(
    InitiateBusinessPaymentOperation => [
        ApprovedBusinessPaymentGrantAction,
        ApprovedBusinessPaymentGrantPurpose,
        ApprovedBusinessPaymentGrantStatus,
        ApprovedBusinessPaymentGrantWorkflow,
        ApprovedBusinessPaymentGrantNotBefore,
        ApprovedBusinessPaymentGrantNotAfter,
        ApprovedBusinessPaymentGrantDelegationLimit
    ]
);
worth_query_operation_links!(
    InitiateBusinessPaymentOperation => [
        ApprovedBusinessPaymentGrantResource,
        ApprovedBusinessPaymentGrantGrantor,
        ApprovedBusinessPaymentGrantGrantee
    ]
);

pub(crate) fn install_payment_approval_grant_minting(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = InitiateBusinessPaymentOperation::reference;
    schema
        .operation_read_relation(operation(), AccountAuthorizedUser::reference())
        .operation_read_relation(operation(), AuthorizationAccount::reference())
        .operation_read_field(operation(), AccountAuthorizationIdentity::reference())
        .operation_read_field(operation(), AuthorizationRole::reference())
        .operation_create(operation(), ApprovedBusinessPaymentGrant::reference())
        .operation_write(operation(), ApprovedBusinessPaymentGrantAction::reference())
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantPurpose::reference(),
        )
        .operation_write(operation(), ApprovedBusinessPaymentGrantStatus::reference())
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantWorkflow::reference(),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantNotBefore::reference(),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantNotAfter::reference(),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantDelegationLimit::reference(),
        )
        .operation_link(
            operation(),
            ApprovedBusinessPaymentGrantResource::reference(),
        )
        .operation_link(
            operation(),
            ApprovedBusinessPaymentGrantGrantor::reference(),
        )
        .operation_link(
            operation(),
            ApprovedBusinessPaymentGrantGrantee::reference(),
        )
}
