//! Aftermath fixture schema declaration members.

use super::operations::{bind_operations, FixtureReads};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationAuthorizationPathBuilder, ApplicationEntityRef,
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationOperationMarkerIdentity, ApplicationPolicyRef, ApplicationPrincipalBindingRef,
    ApplicationPrincipalBindingRequirements, ApplicationPrincipalIdentityRequirement,
    ApplicationPrincipalMappingIdentityRequirement, ApplicationPrincipalMappingStatusRequirement,
    ApplicationPrincipalTargetRequirement, ApplicationRelationRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder, OperationEmits,
    OperationRequiresAbility, U64ApplicationValueBinding,
};

use worth_query_declaration::worth_query_effect;
pub(super) struct AftermathFixtureSchema;
pub(super) struct FixtureInput;
pub(super) struct ReleaseEstate;
pub(super) struct Transfer;
pub(super) struct TransferSmall;
pub(super) struct TransferLarge;
pub(super) struct FreezeAccount;
pub(super) struct FreezeAccountFields;
pub(super) struct FreezeAccountFieldsReversed;
pub(super) struct NotifyDeath;
pub(super) struct LegalHold;
pub(super) struct AuditRetention;
pub(super) struct ApproveEmergencyAccess;
pub(super) struct WireTransferFinal;
pub(super) struct FreezeNote;
pub(super) struct FreezeBalance;
pub(super) struct Charge;
worth_query_declaration::worth_query_portable_type!(
    FixtureInput => "worth.query.test.aftermath-fixture-input.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(pub(super) FixtureInputBinding for FixtureInput { identity: "worth.query.test.aftermath-fixture-input.v1" });

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

#[path = "records.rs"]
mod records;
pub(super) use records::*;

pub(super) struct FixtureAbility;
struct FixturePolicy;
struct PrincipalBinding;
/// Payload projected by the fixture's two real external-effect operation slots.
#[derive(Clone, Copy)]
pub(super) struct FixtureExternalNotice(u64);
worth_query_declaration::worth_query_portable_type!(
    FixtureExternalNotice => "worth.query.test.fixture-external-notice.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(pub(super) FixtureExternalNoticeBinding for FixtureExternalNotice { identity: "worth.query.test.fixture-external-notice.v1" });

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
    pub(super) DeathNoticeEffect for AftermathFixtureSchema, payload FixtureExternalNoticeBinding
);
worth_query_effect!(
    pub(super) WireInstructionEffect for AftermathFixtureSchema, payload FixtureExternalNoticeBinding
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

pub(super) type PrincipalRead = PrincipalIdentityField;
pub(super) type FrozenRead = FrozenField;
pub(super) type NoteRead = NoteField;
pub(super) type BalanceRead = BalanceField;
pub(super) fn principal_read() -> PrincipalRead {
    PrincipalRead::at("PrincipalIdentityField")
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
        worth_query_declaration::facade::application_schema::fragment::declare(Self)
    }
}

type FixtureEntityRef = ApplicationEntityRef<AftermathFixtureSchema, FixtureEntity>;
type FixtureBuilder = ApplicationSchemaDeclarationBuilder<AftermathFixtureSchema>;

fn fixture_policy() -> ApplicationPolicyRef<AftermathFixtureSchema, FixturePolicy> {
    ApplicationPolicyRef::from_schema_identifier("FixturePolicy")
}

fn bind_authorization_shape(
    schema: FixtureBuilder,
    entity: FixtureEntityRef,
    ability: ApplicationAbilityRef<AftermathFixtureSchema, FixtureAbility, FixtureEntity>,
) -> FixtureBuilder {
    schema
        .principal_binding(fixture_principal_binding())
        .policy(fixture_policy())
        .ability(ability)
        .ability_policy(
            ability,
            fixture_policy(),
            [ApplicationAuthorizationPathBuilder::from_principal(entity).allow(entity)],
        )
}

fn fixture_principal_binding() -> ApplicationPrincipalBindingRef<
    AftermathFixtureSchema,
    PrincipalBinding,
    FixtureEntity,
    FixtureEntity,
    u64,
    U64ApplicationValueBinding,
> {
    let identity = ExternalIdentityField::at("ExternalIdentityField");
    let status = MappingStatusField::at("MappingStatusField");
    let target = mapping_target();
    let principal_identity = PrincipalIdentityField::at("PrincipalIdentityField");
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

fn mapping_target(
) -> ApplicationRelationRef<AftermathFixtureSchema, MappingTarget, FixtureEntity, FixtureEntity> {
    worth_query_declaration::facade::application_schema::fragment::relation(
        Fixture,
        FixtureEntity,
        MappingTarget,
    )
}

use worth_query_declaration::facade::application_schema::fragment;
impl fragment::Fragment for AftermathFixtureSchema {
    const NAME: &'static str = "AftermathFixtureSchema";
    type Bindings = fragment::NoBindings;
    type Members = (Fixture, Contracts);
}
pub(super) struct Contracts;
impl fragment::Registration<AftermathFixtureSchema> for Contracts {
    fn register(
        schema: FixtureBuilder,
    ) -> Result<
        FixtureBuilder,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let entity =
            ApplicationEntityRef::<AftermathFixtureSchema, FixtureEntity>::from_schema_identifier(
                "FixtureEntity",
            );
        let ability =
            ApplicationAbilityRef::<AftermathFixtureSchema, FixtureAbility, FixtureEntity>::from_schema_identifiers(
                "FixtureAbility",
                "FixtureEntity",
            );
        let reads = FixtureReads {
            frozen: FrozenRead::at("frozen"),
            note: NoteRead::at("note"),
            balance: BalanceRead::at("balance"),
        };

        let schema = bind_authorization_shape(schema, entity, ability);
        Ok(bind_operations(schema, ability, reads))
    }
}
