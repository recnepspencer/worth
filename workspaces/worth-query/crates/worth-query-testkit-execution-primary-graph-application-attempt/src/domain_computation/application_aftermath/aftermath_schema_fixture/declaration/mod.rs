//! Aftermath fixture schema declaration members.

use crate::domain_computation::application_aftermath::aftermath_schema_fixture::operations::bind_operations;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::operations::FixtureReads;
use worth_foundational::facade::BoundaryProtocolIdentity;
use worth_foundational::facade::BoundaryProtocolVersion;
use worth_query_declaration::facade::application_schema::ApplicationAbilityRef;
use worth_query_declaration::facade::application_schema::ApplicationAspectMarkerIdentity;
use worth_query_declaration::facade::application_schema::ApplicationAspectRef;
use worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_declaration::facade::application_schema::ApplicationEntityRef;
use worth_query_declaration::facade::application_schema::ApplicationExternalEffectBinding;
use worth_query_declaration::facade::application_schema::ApplicationExternalEffectProtocol;
use worth_query_declaration::facade::application_schema::ApplicationFieldMarkerIdentity;
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
use worth_query_declaration::facade::application_schema::ApplicationFieldRef;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_declaration::facade::application_schema::ApplicationPolicyRef;
use worth_query_declaration::facade::application_schema::ApplicationPrincipalBindingRef;
use worth_query_declaration::facade::application_schema::ApplicationPrincipalBindingRequirements;
use worth_query_declaration::facade::application_schema::ApplicationPrincipalIdentityRequirement;
use worth_query_declaration::facade::application_schema::ApplicationPrincipalMappingIdentityRequirement;
use worth_query_declaration::facade::application_schema::ApplicationPrincipalMappingStatusRequirement;
use worth_query_declaration::facade::application_schema::ApplicationPrincipalTargetRequirement;
use worth_query_declaration::facade::application_schema::ApplicationRelationRef;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;
use worth_query_declaration::facade::application_schema::BoolApplicationValueBinding;
use worth_query_declaration::facade::application_schema::DeclaredApplicationFieldValue;
use worth_query_declaration::facade::application_schema::EqualityPredicate;
use worth_query_declaration::facade::application_schema::NoEqualityPredicate;
use worth_query_declaration::facade::application_schema::OperationEmits;
use worth_query_declaration::facade::application_schema::OperationReads;
use worth_query_declaration::facade::application_schema::OperationRequiresAbility;
use worth_query_declaration::facade::application_schema::ReadOnly;
use worth_query_declaration::facade::application_schema::ReadWrite;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding;
use worth_query_declaration::worth_query_effect;
pub struct AftermathFixtureSchema;
pub struct FixtureInput;
pub struct ReleaseEstate;
pub struct Transfer;
pub struct TransferSmall;
pub struct TransferLarge;
pub struct FreezeAccount;
pub struct FreezeAccountFields;
pub struct FreezeAccountFieldsReversed;
pub struct NotifyDeath;
pub struct LegalHold;
pub struct AuditRetention;
pub struct ApproveEmergencyAccess;
pub struct WireTransferFinal;
pub struct FreezeNote;
pub struct FreezeBalance;
pub struct Charge;
worth_query_declaration::worth_query_portable_type!(
    FixtureInput => "worth.query.test.aftermath-fixture-input.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(pub FixtureInputBinding for FixtureInput { identity: "worth.query.test.aftermath-fixture-input.v1" });

macro_rules! operation_identity {
    ($operation:ty => $identifier:literal) => {
        impl ApplicationOperationMarkerIdentity<AftermathFixtureSchema> for $operation {
            type InputBinding = FixtureInputBinding;
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

operation_identity!(ReleaseEstate => "ReleaseEstate");
operation_identity!(Transfer => "transfer");
operation_identity!(TransferSmall => "transfer-small");
operation_identity!(TransferLarge => "transfer-large");
operation_identity!(FreezeAccount => "freeze-account");
operation_identity!(FreezeAccountFields => "freeze-account-fields");
operation_identity!(FreezeAccountFieldsReversed => "freeze-account-fields-reversed");
operation_identity!(NotifyDeath => "notify-death");
operation_identity!(LegalHold => "legal-hold");
operation_identity!(AuditRetention => "audit-retention");
operation_identity!(ApproveEmergencyAccess => "ApproveEmergencyAccess");
operation_identity!(WireTransferFinal => "wire-transfer-final");
operation_identity!(FreezeNote => "freeze-note");
operation_identity!(FreezeBalance => "freeze-balance");
operation_identity!(Charge => "charge");

pub struct FixtureEntity;
pub struct FixtureAbility;
pub struct FixturePolicy;
pub struct IdentityAspect;
pub struct ExternalIdentityField;
pub struct MappingStatusField;
pub struct PrincipalIdentityField;
pub struct FrozenField;
pub struct NoteField;
pub struct BalanceField;
pub struct MappingTarget;
pub struct PrincipalBinding;

impl ApplicationEntityMarkerIdentity<AftermathFixtureSchema> for FixtureEntity {
    const IDENTIFIER: &'static str = "FixtureEntity";
}

impl ApplicationAspectMarkerIdentity<AftermathFixtureSchema, FixtureEntity> for IdentityAspect {
    const IDENTIFIER: &'static str = "IdentityAspect";
    const ASPECT_IDENTITY: worth_query_declaration::facade::application_schema::AspectIdentity =
        worth_query_declaration::facade::application_schema::AspectIdentity(0x91612008);
    const CONTRACT_REVISION:
        worth_query_declaration::facade::application_schema::AspectContractRevision =
        worth_query_declaration::facade::application_schema::AspectContractRevision(1);
}

macro_rules! field_identity {
    ($field:ty => $identifier:literal) => {
        impl ApplicationFieldMarkerIdentity<AftermathFixtureSchema, FixtureEntity, IdentityAspect>
            for $field
        {
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

field_identity!(ExternalIdentityField => "ExternalIdentityField");
field_identity!(MappingStatusField => "MappingStatusField");
field_identity!(PrincipalIdentityField => "PrincipalIdentityField");
field_identity!(FrozenField => "frozen");
field_identity!(NoteField => "note");
field_identity!(BalanceField => "balance");

macro_rules! required_field {
    ($($field:ty => $value:ty, $binding:ty);+ $(;)?) => {
        $(impl DeclaredApplicationFieldValue for $field {
            type Value = $value;
            type Binding = $binding;
            const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
        })+
    };
}

required_field!(
    ExternalIdentityField => WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding;
    MappingStatusField => WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding;
    PrincipalIdentityField => u64, U64ApplicationValueBinding;
    FrozenField => bool, BoolApplicationValueBinding;
    NoteField => String, StringApplicationValueBinding;
    BalanceField => u64, U64ApplicationValueBinding;
);

macro_rules! reads_principal {
    ($($op:ty),+ $(,)?) => { $(impl OperationReads<$op> for PrincipalIdentityField {})+ };
}
reads_principal!(
    ReleaseEstate,
    Transfer,
    TransferSmall,
    TransferLarge,
    NotifyDeath,
    LegalHold,
    AuditRetention,
    ApproveEmergencyAccess,
    WireTransferFinal,
    Charge,
);
impl OperationReads<FreezeAccount> for FrozenField {}
impl OperationReads<FreezeAccountFields> for FrozenField {}
impl OperationReads<FreezeAccountFields> for NoteField {}
impl OperationReads<FreezeAccountFieldsReversed> for FrozenField {}
impl OperationReads<FreezeAccountFieldsReversed> for NoteField {}
impl OperationReads<FreezeNote> for NoteField {}
impl OperationReads<FreezeBalance> for BalanceField {}

/// Payload projected by the fixture's two real external-effect operation slots.
#[derive(Clone, Copy)]
pub struct FixtureExternalNotice(u64);
worth_query_declaration::worth_query_portable_type!(
    FixtureExternalNotice => "worth.query.test.fixture-external-notice.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(pub FixtureExternalNoticeBinding for FixtureExternalNotice { identity: "worth.query.test.fixture-external-notice.v1" });

impl ApplicationExternalEffectBinding for FixtureExternalNoticeBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("test.fixture-external-notice"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 8;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.to_be_bytes().to_vec()
    }
}

worth_query_effect!(
    pub DeathNoticeEffect for AftermathFixtureSchema, payload FixtureExternalNoticeBinding
);
worth_query_effect!(
    pub WireInstructionEffect for AftermathFixtureSchema, payload FixtureExternalNoticeBinding
);

impl OperationEmits<NotifyDeath> for DeathNoticeEffect {}
impl OperationEmits<WireTransferFinal> for WireInstructionEffect {}

macro_rules! requires_ability {
    ($($op:ty),+ $(,)?) => { $(impl OperationRequiresAbility<$op> for FixtureAbility {})+ };
}
requires_ability!(
    ReleaseEstate,
    Transfer,
    TransferSmall,
    TransferLarge,
    FreezeAccount,
    FreezeAccountFields,
    FreezeAccountFieldsReversed,
    NotifyDeath,
    LegalHold,
    AuditRetention,
    ApproveEmergencyAccess,
    WireTransferFinal,
    FreezeNote,
    FreezeBalance,
    Charge,
);

pub type PrincipalRead = ApplicationFieldRef<
    AftermathFixtureSchema,
    FixtureEntity,
    IdentityAspect,
    PrincipalIdentityField,
    u64,
    ReadOnly,
    EqualityPredicate,
>;
pub type FrozenRead = ApplicationFieldRef<
    AftermathFixtureSchema,
    FixtureEntity,
    IdentityAspect,
    FrozenField,
    bool,
    ReadOnly,
    NoEqualityPredicate,
>;
pub type NoteRead = ApplicationFieldRef<
    AftermathFixtureSchema,
    FixtureEntity,
    IdentityAspect,
    NoteField,
    String,
    ReadOnly,
    NoEqualityPredicate,
>;
pub type BalanceRead = ApplicationFieldRef<
    AftermathFixtureSchema,
    FixtureEntity,
    IdentityAspect,
    BalanceField,
    u64,
    ReadOnly,
    NoEqualityPredicate,
>;

pub fn principal_read() -> PrincipalRead {
    ApplicationFieldRef::from_schema_types()
}

impl ApplicationSchema for AftermathFixtureSchema {
    const OWNER: &'static str = "aftermath-fixture";
    const NAME: &'static str = "AftermathFixtureSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let entity =
            ApplicationEntityRef::<Self, FixtureEntity>::from_schema_identifier("FixtureEntity");
        let ability =
            ApplicationAbilityRef::<Self, FixtureAbility, FixtureEntity>::from_schema_identifiers(
                "FixtureAbility",
                "FixtureEntity",
            );
        let reads = FixtureReads {
            frozen: ApplicationFieldRef::from_schema_types(),
            note: ApplicationFieldRef::from_schema_types(),
            balance: ApplicationFieldRef::from_schema_types(),
        };
        let schema = bind_entity_shape(entity, &reads);
        let schema = bind_authorization_shape(schema, entity, ability);
        bind_operations(schema, ability, reads)
    }
}

pub type FixtureEntityRef = ApplicationEntityRef<AftermathFixtureSchema, FixtureEntity>;
pub type FixtureBuilder = ApplicationSchemaDeclarationBuilder<AftermathFixtureSchema>;

pub fn fixture_policy() -> ApplicationPolicyRef<AftermathFixtureSchema, FixturePolicy> {
    ApplicationPolicyRef::from_schema_identifier("FixturePolicy")
}

pub fn bind_entity_shape(entity: FixtureEntityRef, reads: &FixtureReads) -> FixtureBuilder {
    ApplicationSchemaDeclarationBuilder::<AftermathFixtureSchema>::for_schema()
        .entity(entity)
        .aspect(
            entity,
            ApplicationAspectRef::<AftermathFixtureSchema, FixtureEntity, IdentityAspect>::from_schema_identifier(
                "IdentityAspect",
            ),
        )
        .field(
            entity,
            ApplicationFieldRef::<
                AftermathFixtureSchema,
                FixtureEntity,
                IdentityAspect,
                ExternalIdentityField,
                WorthQueryExternalPrincipalIdentity,
                ReadOnly,
                EqualityPredicate,
            >::from_schema_types(),
        )
        .field(
            entity,
            ApplicationFieldRef::<
                AftermathFixtureSchema,
                FixtureEntity,
                IdentityAspect,
                MappingStatusField,
                WorthQueryPrincipalMappingStatus,
                ReadWrite,
                NoEqualityPredicate,
            >::from_schema_types(),
        )
        .field(entity, principal_read())
        .field(entity, reads.frozen)
        .field(entity, reads.note)
        .field(entity, reads.balance)
}

pub fn bind_authorization_shape(
    schema: FixtureBuilder,
    entity: FixtureEntityRef,
    ability: ApplicationAbilityRef<AftermathFixtureSchema, FixtureAbility, FixtureEntity>,
) -> FixtureBuilder {
    schema
        .relation(mapping_target(), entity, entity)
        .principal_binding(fixture_principal_binding())
        .policy(fixture_policy())
        .ability(ability)
        .ability_policy(
            ability,
            fixture_policy(),
            [ApplicationAuthorizationPathBuilder::from_principal(entity).allow(entity)],
        )
}

pub fn fixture_principal_binding() -> ApplicationPrincipalBindingRef<
    AftermathFixtureSchema,
    PrincipalBinding,
    FixtureEntity,
    FixtureEntity,
    u64,
    U64ApplicationValueBinding,
> {
    let identity = ApplicationFieldRef::<
        AftermathFixtureSchema,
        FixtureEntity,
        IdentityAspect,
        ExternalIdentityField,
        WorthQueryExternalPrincipalIdentity,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    let status = ApplicationFieldRef::<
        AftermathFixtureSchema,
        FixtureEntity,
        IdentityAspect,
        MappingStatusField,
        WorthQueryPrincipalMappingStatus,
        ReadWrite,
        NoEqualityPredicate,
    >::from_schema_types();
    let target = mapping_target();
    let principal_identity = ApplicationFieldRef::<
        AftermathFixtureSchema,
        FixtureEntity,
        IdentityAspect,
        PrincipalIdentityField,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    ApplicationPrincipalBindingRef::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(identity),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(status),
            target: ApplicationPrincipalTargetRequirement::from_relation(target),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                principal_identity,
            ),
        },
    )
}

pub fn mapping_target(
) -> ApplicationRelationRef<AftermathFixtureSchema, MappingTarget, FixtureEntity, FixtureEntity> {
    ApplicationRelationRef::from_schema_identifiers(
        "MappingTarget",
        "FixtureEntity",
        "FixtureEntity",
        worth_query_installation::facade::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}
