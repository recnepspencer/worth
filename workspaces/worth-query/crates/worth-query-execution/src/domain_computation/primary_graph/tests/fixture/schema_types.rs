use super::*;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationRetainedEffectBinding, ApplicationStructuredValueBinding,
    ApplicationValueValidationDenial, StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentityBinding, WorthQueryPrincipalMappingStatusBinding,
};

#[path = "exact_preimage_retention.rs"]
pub(super) mod exact_preimage_retention;
#[path = "wrong_field_retention.rs"]
pub(super) mod wrong_field_retention;

worth_query_entity!(pub ExternalMapping for IdentityExecutionSchema);
worth_query_entity!(pub Principal for IdentityExecutionSchema);
worth_query_entity!(pub Account for IdentityExecutionSchema);
worth_query_entity!(pub Activity for IdentityExecutionSchema);
worth_query_aspect!(pub ExternalIdentity for IdentityExecutionSchema, ExternalMapping; identity = AspectIdentity(0x9161103a), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for IdentityExecutionSchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity for IdentityExecutionSchema, Principal; identity = AspectIdentity(0x9161103b), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for IdentityExecutionSchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatusField for IdentityExecutionSchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_relation!(
    pub MappingTarget in IdentityExecutionSchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub IdentityBinding in IdentityExecutionSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);
worth_query_aspect!(pub AccountPolicy for IdentityExecutionSchema, Account; identity = AspectIdentity(0x9161103c), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountIdentity for IdentityExecutionSchema, Account, AccountPolicy:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub AccountStatus for IdentityExecutionSchema, Account, AccountPolicy:
    String => StringApplicationValueBinding, read_write, equality
);
worth_query_field!(
    pub AccountLabel for IdentityExecutionSchema, Account, AccountPolicy:
    String => StringApplicationValueBinding, read_write, equality
);
worth_query_field!(
    pub AccountNote for IdentityExecutionSchema, Account, AccountPolicy:
    optional String => StringApplicationValueBinding, read_write, equality
);
worth_query_field!(
    pub AccountScore for IdentityExecutionSchema, Account, AccountPolicy:
    optional u64 => U64ApplicationValueBinding, read_write, no_equality
);
worth_query_aspect!(pub ActivityFacts for IdentityExecutionSchema, Activity; identity = AspectIdentity(0x9161103d), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ActivityIdentity for IdentityExecutionSchema, Activity, ActivityFacts:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub ActivitySequence for IdentityExecutionSchema, Activity, ActivityFacts:
    u64 => U64ApplicationValueBinding, read_only, no_equality
);
worth_query_relation!(pub AccountOwner in IdentityExecutionSchema, Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub AccountBlocked in IdentityExecutionSchema, Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub AccountPrimaryActivity in IdentityExecutionSchema, Account => Activity; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub AccountSecondaryActivity in IdentityExecutionSchema, Account => Activity; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub AccountAllActivity in IdentityExecutionSchema, Account => Activity; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ActivityAccount in IdentityExecutionSchema, Activity => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_ability!(pub ViewAccount scoped_to Account, in IdentityExecutionSchema);
worth_query_ability!(pub EditAccount scoped_to Account, in IdentityExecutionSchema);
worth_query_ability!(pub ManageOwnership scoped_to Principal, in IdentityExecutionSchema);
worth_query_policy!(pub AccountAccessPolicy in IdentityExecutionSchema);
worth_query_declaration::worth_query_structured_value_binding!(pub AccountActivityBinding for String { identity: "worth.rust.string" });
impl ApplicationRetainedEffectBinding for AccountActivityBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<Self::Value>())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::try_from(value.capacity()).unwrap_or(u64::MAX))
    }
}
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

impl ApplicationRetainedEffectBinding for RetainedStatusNoticeBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(value.0.len()).unwrap_or(u64::MAX)
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
pub struct MutationFreeNotice(pub String);
worth_query_declaration::worth_query_portable_type!(
    MutationFreeNotice => "worth.query.test.mutation-free-notice.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(pub MutationFreeNoticeBinding for MutationFreeNotice { identity: "worth.query.test.mutation-free-notice.v1" });
impl ApplicationRetainedEffectBinding for MutationFreeNoticeBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(value.0.len()).unwrap_or(u64::MAX)
    }
}

impl ApplicationExternalEffectBinding for MutationFreeNoticeBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("test.mutation-free"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 256;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.as_bytes().to_vec()
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
worth_query_operation_expects_fact!(TouchAccountOperation => [AccountStatus]);
worth_query_operation_requires!(MultiTouchOperation => [ViewAccount, EditAccount]);
worth_query_operation_requires!(ChangeOwnershipOperation => [ManageOwnership]);
worth_query_operation_requires!(PatchAccountDraftOperation => [ViewAccount]);
worth_query_operation_writes!(TouchAccountOperation => [AccountStatus, AccountLabel]);
worth_query_operation_writes!(WrongFieldRetentionOperation => [AccountLabel]);
worth_query_operation_writes!(ExactStatusRetentionOperation => [AccountStatus, AccountLabel]);
worth_query_operation_writes!(MultiFieldRetentionOperation => [AccountStatus, AccountLabel]);
// Deliberately wider than the installed contract so authority-ceiling tests
// prove that compile-time capability cannot widen installed authority.
worth_query_operation_writes!(MultiTouchOperation => [AccountStatus, AccountLabel]);
worth_query_operation_writes!(PatchAccountDraftOperation => [AccountNote, AccountScore]);
worth_query_operation_emits!(MultiTouchOperation => [AccountActivityEffect]);
worth_query_operation_emits!(
    TouchAccountOperation => [AccountActivityEffect, LiveActivityEffect]
);
worth_query_operation_emits!(ExactStatusRetentionOperation => [RetainedStatusEffect]);
worth_query_operation_emits!(MutationFreeEmitOperation => [MutationFreeExternalEffect]);
worth_query_operation_reads!(TouchAccountOperation => [AccountStatus, AccountLabel, AccountOwner]);
worth_query_operation_reads!(WrongFieldRetentionOperation => [AccountStatus, AccountLabel]);
worth_query_operation_reads!(ExactStatusRetentionOperation => [AccountStatus, AccountLabel]);
worth_query_operation_reads!(MultiFieldRetentionOperation => [AccountStatus, AccountLabel]);
worth_query_operation_reads!(MultiTouchOperation => [AccountStatus, AccountLabel]);
worth_query_operation_reads!(ChangeOwnershipOperation => [AccountOwner, AccountStatus]);
worth_query_operation_reads!(PatchAccountDraftOperation => [AccountNote, AccountScore]);
worth_query_operation_links!(ChangeOwnershipOperation => [AccountOwner]);
worth_query_operation_unlinks!(ChangeOwnershipOperation => [AccountOwner]);

pub(in crate::domain_computation::primary_graph) type InstalledIdentityBinding =
    WorthQueryInstalledPrincipalBinding<
        IdentityExecutionSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >;

pub(in crate::domain_computation::primary_graph) struct IdentityWorld {
    pub(in crate::domain_computation::primary_graph) application:
        WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    pub(in crate::domain_computation::primary_graph) binding: InstalledIdentityBinding,
}

pub(in crate::domain_computation::primary_graph) fn external_identity(
    subject: &str,
) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://issuer.example", subject).unwrap()
}

pub(in crate::domain_computation::primary_graph) fn live_scope() -> WorthQueryRequestScope {
    let source = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), source.token())
}
