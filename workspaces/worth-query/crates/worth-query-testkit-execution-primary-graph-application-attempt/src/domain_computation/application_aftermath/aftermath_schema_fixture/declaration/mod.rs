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

#[path = "../records.rs"]
mod records;
pub use records::*;

pub struct FixtureAbility;
struct FixturePolicy;
struct PrincipalBinding;
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

pub type PrincipalRead = PrincipalIdentityField;
pub type FrozenRead = FrozenField;
pub type NoteRead = NoteField;
pub type BalanceRead = BalanceField;
pub fn principal_read() -> PrincipalRead {
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
pub struct Contracts;
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

mod principal_binding;
use principal_binding::fixture_principal_binding;
