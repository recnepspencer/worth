#[path = "records.rs"]
pub mod storage_model;
use super::*;
pub use storage_model::*;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
    U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentityBinding, WorthQueryPrincipalMappingStatusBinding,
};

pub mod exact_preimage_retention;
pub mod wrong_field_retention;

pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        IdentityExecutionSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "IdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

worth_query_ability!(pub ViewAccount scoped_to Account, in IdentityExecutionSchema);
worth_query_ability!(pub EditAccount scoped_to Account, in IdentityExecutionSchema);
worth_query_ability!(pub ManageOwnership scoped_to Principal, in IdentityExecutionSchema);
worth_query_policy!(pub AccountAccessPolicy in IdentityExecutionSchema);
worth_query_declaration::worth_query_structured_value_binding!(pub AccountActivityBinding for String { identity: "worth.rust.string" });

worth_query_effect!(pub AccountActivityEffect for IdentityExecutionSchema, payload AccountActivityBinding);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedStatusNotice(pub String);
worth_query_declaration::worth_query_portable_type!(
    RetainedStatusNotice => "worth.query.test.retained-status-notice.v1"
);

pub struct RetainedStatusNoticeBinding;

impl ApplicationStructuredValueBinding for RetainedStatusNoticeBinding {
    type Value = RetainedStatusNotice;

    const IDENTITY_NAME: &'static str = "worth.query.test.retained-status-notice.v1";

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        (!value.0.is_empty()).then_some(()).ok_or_else(|| {
            ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "status notice must not be empty",
            )
        })
    }
}

impl ApplicationExternalEffectBinding for RetainedStatusNoticeBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("test.status-retention"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 256;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.as_bytes().to_vec()
    }
}

worth_query_effect!(pub RetainedStatusEffect for IdentityExecutionSchema, payload RetainedStatusNoticeBinding);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationFreeNotice(pub u8);
worth_query_declaration::worth_query_portable_type!(
    MutationFreeNotice => "worth.query.test.mutation-free-notice.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(pub MutationFreeNoticeBinding for MutationFreeNotice { identity: "worth.query.test.mutation-free-notice.v1" });

impl ApplicationExternalEffectBinding for MutationFreeNoticeBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("test.mutation-free"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 256;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        vec![value.0; usize::from(value.0)]
    }
}

worth_query_effect!(pub MutationFreeExternalEffect for IdentityExecutionSchema, payload MutationFreeNoticeBinding);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TouchAccountInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrongFieldRetentionInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactStatusRetentionInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiFieldRetentionInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiTouchInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeOwnershipInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationFreeEmitInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchAccountDraftInput;

worth_query_declaration::worth_query_portable_type!(
    TouchAccountInput => "worth.query.test.touch-account-input.v1"
);
worth_query_declaration::worth_query_portable_type!(
    WrongFieldRetentionInput => "worth.query.test.wrong-field-retention-input.v1"
);
worth_query_declaration::worth_query_portable_type!(
    ExactStatusRetentionInput => "worth.query.test.exact-status-retention-input.v1"
);
worth_query_declaration::worth_query_portable_type!(
    MultiFieldRetentionInput => "worth.query.test.multi-field-retention-input.v1"
);
worth_query_declaration::worth_query_portable_type!(
    MultiTouchInput => "worth.query.test.multi-touch-input.v1"
);
worth_query_declaration::worth_query_portable_type!(
    ChangeOwnershipInput => "worth.query.test.change-ownership-input.v1"
);
worth_query_declaration::worth_query_portable_type!(
    MutationFreeEmitInput => "worth.query.test.mutation-free-emit-input.v1"
);
worth_query_declaration::worth_query_portable_type!(
    PatchAccountDraftInput => "worth.query.test.patch-account-draft-input.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(pub TouchAccountInputBinding for TouchAccountInput { identity: "worth.query.test.touch-account-input.v1" });
worth_query_operation!(pub TouchAccountOperation for IdentityExecutionSchema, input TouchAccountInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub WrongFieldRetentionInputBinding for WrongFieldRetentionInput { identity: "worth.query.test.wrong-field-retention-input.v1" });
worth_query_operation!(pub WrongFieldRetentionOperation for IdentityExecutionSchema, input WrongFieldRetentionInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub ExactStatusRetentionInputBinding for ExactStatusRetentionInput { identity: "worth.query.test.exact-status-retention-input.v1" });
worth_query_operation!(pub ExactStatusRetentionOperation for IdentityExecutionSchema, input ExactStatusRetentionInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub MultiFieldRetentionInputBinding for MultiFieldRetentionInput { identity: "worth.query.test.multi-field-retention-input.v1" });
worth_query_operation!(pub MultiFieldRetentionOperation for IdentityExecutionSchema, input MultiFieldRetentionInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub MultiTouchInputBinding for MultiTouchInput { identity: "worth.query.test.multi-touch-input.v1" });
worth_query_operation!(pub MultiTouchOperation for IdentityExecutionSchema, input MultiTouchInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub ChangeOwnershipInputBinding for ChangeOwnershipInput { identity: "worth.query.test.change-ownership-input.v1" });
worth_query_operation!(pub ChangeOwnershipOperation for IdentityExecutionSchema, input ChangeOwnershipInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub MutationFreeEmitInputBinding for MutationFreeEmitInput { identity: "worth.query.test.mutation-free-emit-input.v1" });
worth_query_operation!(pub MutationFreeEmitOperation for IdentityExecutionSchema, input MutationFreeEmitInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub PatchAccountDraftInputBinding for PatchAccountDraftInput { identity: "worth.query.test.patch-account-draft-input.v1" });
worth_query_operation!(pub PatchAccountDraftOperation for IdentityExecutionSchema, input PatchAccountDraftInputBinding);
worth_query_operation_requires!(TouchAccountOperation => [ViewAccount]);
worth_query_operation_requires!(WrongFieldRetentionOperation => [ViewAccount]);
worth_query_operation_requires!(ExactStatusRetentionOperation => [ViewAccount]);
worth_query_operation_requires!(MultiFieldRetentionOperation => [ViewAccount]);

worth_query_operation_requires!(MultiTouchOperation => [ViewAccount, EditAccount]);
worth_query_operation_requires!(ChangeOwnershipOperation => [ManageOwnership]);
worth_query_operation_requires!(PatchAccountDraftOperation => [ViewAccount]);

worth_query_declaration::worth_query_operation_deletes!(TouchAccountOperation => [Account]);

// Deliberately wider than the installed contract so authority-ceiling tests
// prove that compile-time capability cannot widen installed authority.

worth_query_operation_emits!(MultiTouchOperation => [AccountActivityEffect]);
worth_query_operation_emits!(
    TouchAccountOperation => [AccountActivityEffect, LiveActivityEffect]
);
worth_query_operation_emits!(ExactStatusRetentionOperation => [RetainedStatusEffect]);
worth_query_operation_emits!(MutationFreeEmitOperation => [MutationFreeExternalEffect]);
worth_query_operation_reads!(TouchAccountOperation => [Account, AccountStatus, AccountOwner]);
// Moved from the ordinary admission court: the orphan rule keeps the marker with its types.
impl
    worth_query_declaration::facade::application_schema::OperationExpectsFact<TouchAccountOperation>
    for AccountLabel
{
}
worth_query_operation_reads!(WrongFieldRetentionOperation => [AccountStatus]);
worth_query_operation_reads!(ExactStatusRetentionOperation => [AccountStatus]);
worth_query_operation_reads!(MultiFieldRetentionOperation => [AccountStatus]);
worth_query_operation_reads!(MultiTouchOperation => [AccountStatus]);
worth_query_operation_reads!(ChangeOwnershipOperation => [AccountOwner, AccountStatus]);
worth_query_operation_reads!(PatchAccountDraftOperation => [AccountNote, AccountScore]);
worth_query_operation_links!(ChangeOwnershipOperation => [AccountOwner]);
worth_query_operation_unlinks!(ChangeOwnershipOperation => [AccountOwner]);

pub type InstalledIdentityBinding = WorthQueryInstalledPrincipalBinding<
    IdentityExecutionSchema,
    IdentityBinding,
    ExternalMapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
>;

pub struct IdentityWorld {
    pub application: WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    pub binding: InstalledIdentityBinding,
}

pub fn external_identity(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://issuer.example", subject).unwrap()
}

pub fn live_scope() -> WorthQueryRequestScope {
    let source = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
}

use worth_query_declaration::facade::application_schema::{
    fragment::RecordSelection, ApplicationPrincipalBindingRef,
    ApplicationPrincipalBindingRequirements, ApplicationPrincipalIdentityRequirement,
    ApplicationPrincipalMappingIdentityRequirement, ApplicationPrincipalMappingStatusRequirement,
    ApplicationPrincipalTargetRequirement,
};
