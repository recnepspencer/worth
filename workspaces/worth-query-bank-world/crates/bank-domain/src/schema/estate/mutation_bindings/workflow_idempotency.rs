use worth_query_decl::facade::application_capability::ApplicationCapabilityWorkflowIdempotency;

use crate::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        ApproveEstateEmergencyAccessOperation, BankSchema, CompleteEstateMandatoryReviewOperation,
        DelegateEstateCapabilityOperation, RequestEstateEmergencyAccessOperation,
        RevokeEstateCapabilityOperation, RevokeEstateEmergencyAccessOperation,
    },
};

impl ApplicationCapabilityWorkflowIdempotency<BankSchema, EstateAction, BankIdempotencyKey>
    for RequestEstateEmergencyAccessOperation
{
    fn client_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        *super::action_identity::canonical_action_payload(
            "application-request-estate-emergency-access",
            input,
        )
        .derive_identity()
        .bytes()
    }
}

impl ApplicationCapabilityWorkflowIdempotency<BankSchema, EstateAction, BankIdempotencyKey>
    for ApproveEstateEmergencyAccessOperation
{
    fn client_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        *super::action_identity::canonical_action_payload(
            "application-approve-estate-emergency-access",
            input,
        )
        .derive_identity()
        .bytes()
    }
}

impl ApplicationCapabilityWorkflowIdempotency<BankSchema, EstateAction, BankIdempotencyKey>
    for RevokeEstateEmergencyAccessOperation
{
    fn client_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        *super::action_identity::canonical_action_payload(
            "application-revoke-estate-emergency-access",
            input,
        )
        .derive_identity()
        .bytes()
    }
}

impl ApplicationCapabilityWorkflowIdempotency<BankSchema, EstateAction, BankIdempotencyKey>
    for CompleteEstateMandatoryReviewOperation
{
    fn client_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        *super::action_identity::canonical_action_payload(
            "application-complete-estate-mandatory-review",
            input,
        )
        .derive_identity()
        .bytes()
    }
}

impl ApplicationCapabilityWorkflowIdempotency<BankSchema, EstateAction, BankIdempotencyKey>
    for DelegateEstateCapabilityOperation
{
    fn client_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        *super::action_identity::canonical_action_payload(
            "application-delegate-estate-capability",
            input,
        )
        .derive_identity()
        .bytes()
    }
}

impl ApplicationCapabilityWorkflowIdempotency<BankSchema, EstateAction, BankIdempotencyKey>
    for RevokeEstateCapabilityOperation
{
    fn client_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        *super::action_identity::canonical_action_payload(
            "application-revoke-estate-capability",
            input,
        )
        .derive_identity()
        .bytes()
    }
}
