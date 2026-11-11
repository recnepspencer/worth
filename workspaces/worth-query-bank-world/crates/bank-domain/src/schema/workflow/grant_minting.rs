use worth_query_decl::facade::{
    application_schema::ApplicationSchemaDeclarationBuilder, worth_query_operation_creates,
    worth_query_operation_links, worth_query_operation_reads,
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

worth_query_operation_reads!(InitiateBusinessPaymentOperation => [AccountAuthorizedUser, AuthorizationAccount]);
worth_query_operation_creates!(InitiateBusinessPaymentOperation => [ApprovedBusinessPaymentGrant]);

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
        .operation_read_field(
            operation(),
            AccountAuthorizationIdentity::at("AccountAuthorizationIdentity"),
        )
        .operation_read_field(operation(), AuthorizationRole::at("AuthorizationRole"))
        .operation_create(operation(), ApprovedBusinessPaymentGrant::reference())
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantAction::at("ApprovedBusinessPaymentGrantAction"),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantPurpose::at("ApprovedBusinessPaymentGrantPurpose"),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantStatus::at("ApprovedBusinessPaymentGrantStatus"),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantWorkflow::at("ApprovedBusinessPaymentGrantWorkflow"),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantNotBefore::at("ApprovedBusinessPaymentGrantNotBefore"),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantNotAfter::at("ApprovedBusinessPaymentGrantNotAfter"),
        )
        .operation_write(
            operation(),
            ApprovedBusinessPaymentGrantDelegationLimit::at(
                "ApprovedBusinessPaymentGrantDelegationLimit",
            ),
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
