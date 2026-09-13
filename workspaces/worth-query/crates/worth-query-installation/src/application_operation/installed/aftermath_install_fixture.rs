//! Shared install fixtures for aftermath classification tests.

use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolVersion, CanonicalDigestId,
};
use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract, DeclaredCompensation,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredRecordedInverse, PortableApplicationAftermathContract,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity,
    ApplicationExternalEffectProtocol, ApplicationFieldMarkerIdentity, ApplicationFieldPresence,
    ApplicationFieldRef, ApplicationOperationDecisionReadTarget,
    ApplicationOperationMarkerIdentity, ApplicationOperationProgramTarget, ApplicationOperationRef,
    ApplicationSchema, ApplicationSchemaBindingIdentity, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember, DeclaredApplicationFieldValue,
    U64ApplicationValueBinding, WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use crate::application_aftermath::{
    install_application_aftermath, WorthQueryAftermathInstallationDenial,
    WorthQueryInstalledAftermathContract,
};
use crate::package::WorthQueryPortableInstalledReconciliationProcedureRecord;

use super::super::contract_resolution::{operation_aftermath, operation_external_effect};
use source::FixtureAftermathInstallationSource;

mod source;

pub(crate) struct FixtureSchema;
struct FixtureOperation;
struct FixtureInput;
pub(crate) struct Account;
pub(crate) struct OtherAccount;
pub(crate) struct State;
pub(crate) struct OtherAccountState;
pub(crate) struct OtherState;
pub(crate) struct Balance;
pub(crate) struct OtherAccountBalance;
pub(crate) struct OtherStateBalance;
pub(crate) struct SecretField;
pub(crate) struct OtherBalance;

worth_query_declaration::worth_query_portable_type!(
    FixtureInput => "worth.query.installation-test.aftermath-input"
);
worth_query_declaration::worth_query_structured_value_binding!(
    FixtureInputBinding for FixtureInput { identity: "worth.query.installation-test.aftermath-input" }
);

impl ApplicationOperationMarkerIdentity<FixtureSchema> for FixtureOperation {
    type InputBinding = FixtureInputBinding;
    const IDENTIFIER: &'static str = "FixtureOperation";
}

macro_rules! entity_identity {
    ($marker:ty, $identifier:literal) => {
        impl ApplicationEntityMarkerIdentity<FixtureSchema> for $marker {
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

macro_rules! aspect_identity {
    ($marker:ty, $entity:ty, $identifier:literal, $identity:expr) => {
        impl ApplicationAspectMarkerIdentity<FixtureSchema, $entity> for $marker {
            const IDENTIFIER: &'static str = $identifier;
            const ASPECT_IDENTITY:
                worth_query_declaration::facade::application_schema::AspectIdentity =
                worth_query_declaration::facade::application_schema::AspectIdentity($identity);
            const CONTRACT_REVISION:
                worth_query_declaration::facade::application_schema::AspectContractRevision =
                worth_query_declaration::facade::application_schema::AspectContractRevision(1);
        }
    };
}

macro_rules! field_identity {
    ($marker:ty, $entity:ty, $aspect:ty, $identifier:literal) => {
        impl ApplicationFieldMarkerIdentity<FixtureSchema, $entity, $aspect> for $marker {
            const IDENTIFIER: &'static str = $identifier;
        }
        impl DeclaredApplicationFieldValue for $marker {
            type Value = u64;
            type Binding = U64ApplicationValueBinding;
            const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
        }
    };
}

entity_identity!(Account, "Account");
entity_identity!(OtherAccount, "OtherAccount");
aspect_identity!(State, Account, "State", 0x9161_2101);
aspect_identity!(OtherAccountState, OtherAccount, "State", 0x9161_2102);
aspect_identity!(OtherState, Account, "OtherState", 0x9161_2103);
field_identity!(Balance, Account, State, "balance");
field_identity!(
    OtherAccountBalance,
    OtherAccount,
    OtherAccountState,
    "balance"
);
field_identity!(OtherStateBalance, Account, OtherState, "balance");
field_identity!(SecretField, Account, State, "secret-field");
field_identity!(OtherBalance, Account, State, "other-balance");

impl ApplicationSchema for FixtureSchema {
    const OWNER: &'static str = "worth-query-installation-tests";
    const NAME: &'static str = "AftermathFixture";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema().build()
    }
}

pub(crate) fn digest(byte: u8) -> CanonicalDigestId {
    CanonicalDigestId::new([byte; 32])
}

/// One operation under aftermath installation, with every axis a test may vary.
///
/// The escaping lane defaults to `None` — the operation declared no
/// operation-definition external-effect slot — and `escaping()` is the twin where it
/// did. Making the lane a visible axis here is the point: it is the operation's
/// own contract, and no aftermath declaration can contradict it (Q8.25-C1).
pub(crate) struct AftermathInstall {
    binding: ApplicationSchemaBindingIdentity,
    operation_slot: &'static str,
    declared_reads: Vec<ApplicationOperationDecisionReadTarget>,
    external_effect: Option<FixtureExternalEffect>,
}

struct FixtureExternalEffect {
    rust_payload_type: WorthQueryPortableTypeIdentity,
    protocol: ApplicationExternalEffectProtocol,
}

impl AftermathInstall {
    pub(crate) fn new(
        binding: ApplicationSchemaBindingIdentity,
        operation_slot: &'static str,
    ) -> Self {
        Self {
            binding,
            operation_slot,
            declared_reads: Vec::new(),
            external_effect: None,
        }
    }

    pub(crate) fn reads<Field>(self) -> Self
    where
        Field: ApplicationFieldMarkerIdentity<FixtureSchema, Account, State, Value = u64>,
    {
        self.reads_at::<Account, State, Field>()
    }

    pub(crate) fn reads_at<Entity, Aspect, Field>(mut self) -> Self
    where
        Entity: ApplicationEntityMarkerIdentity<FixtureSchema>,
        Aspect: ApplicationAspectMarkerIdentity<FixtureSchema, Entity>,
        Field: ApplicationFieldMarkerIdentity<FixtureSchema, Entity, Aspect, Value = u64>,
    {
        self.declared_reads = vec![decision_read_target::<Entity, Aspect, Field>()];
        self
    }

    /// The operation definition declares an external-effect slot on the schema.
    pub(crate) fn escaping(self) -> Self {
        self.escaping_with_protocol(protocol(1))
    }

    pub(crate) fn escaping_with_protocol(
        self,
        protocol: ApplicationExternalEffectProtocol,
    ) -> Self {
        self.escaping_with_contract("fixture::EscapingPayload", protocol)
    }

    pub(crate) fn escaping_with_contract(
        mut self,
        rust_payload_type: &'static str,
        protocol: ApplicationExternalEffectProtocol,
    ) -> Self {
        self.external_effect = Some(FixtureExternalEffect {
            rust_payload_type: WorthQueryPortableTypeIdentity::declared(rust_payload_type),
            protocol,
        });
        self
    }

    pub(crate) fn install(
        &self,
        declared: DeclaredApplicationAftermathContract<FixtureSchema>,
    ) -> Result<WorthQueryInstalledAftermathContract, WorthQueryAftermathInstallationDenial> {
        let mut members = associated_operation_members(self.operation_slot, declared);
        members.extend([
            ApplicationSchemaMember::OperationProgram {
                operation: self.operation_slot.to_owned(),
                target: ApplicationOperationProgramTarget::Create {
                    entity: "FixtureEntity".to_owned(),
                },
            },
            ApplicationSchemaMember::OperationDecisionFactBudget {
                operation: self.operation_slot.to_owned(),
                maximum_fact_count: 8,
            },
            ApplicationSchemaMember::OperationProjectionWorkBudget {
                operation: self.operation_slot.to_owned(),
                maximum_work_units: 16,
            },
        ]);
        members.extend(self.declared_reads.iter().cloned().map(|target| {
            ApplicationSchemaMember::OperationDecisionRead {
                operation: self.operation_slot.to_owned(),
                target,
            }
        }));
        if let Some(external_effect) = &self.external_effect {
            members.push(ApplicationSchemaMember::OperationExternalEffect {
                operation: self.operation_slot.to_owned(),
                effect: "EscapingEffect".to_owned(),
                rust_payload_type: external_effect.rust_payload_type.clone(),
                protocol: external_effect.protocol.clone(),
                maximum_payload_bytes: 64,
                correlation_family: WorthQueryExternalEffectCorrelationFamily::new("escaped-rail")
                    .unwrap(),
            });
        }
        let portable_aftermath = operation_aftermath(&members, self.operation_slot)
            .expect("the fixture aftermath is unambiguous");
        let portable_reconciliation = portable_aftermath
            .as_ref()
            .and_then(PortableApplicationAftermathContract::reconciliation)
            .map(|procedure| {
                WorthQueryPortableInstalledReconciliationProcedureRecord::new(
                    procedure.procedure_slot().to_owned(),
                )
            });
        let source = FixtureAftermathInstallationSource::new(
            self.binding.clone(),
            self.operation_slot.to_owned(),
            self.declared_reads.clone(),
            operation_external_effect(&members, self.operation_slot)
                .expect("the fixture external effect is unambiguous"),
            portable_aftermath,
            portable_reconciliation,
        );
        install_application_aftermath(&source)
            .map(|installed| installed.expect("the fixture declares an aftermath"))
    }
}

fn associated_operation_members(
    operation_slot: &'static str,
    contract: DeclaredApplicationAftermathContract<FixtureSchema>,
) -> Vec<ApplicationSchemaMember> {
    let definition =
        ApplicationOperationRef::<FixtureSchema, FixtureOperation, FixtureInput>::from_declaration(
        )
        .definition()
        .no_external_effect()
        .aftermath(contract)
        .finish();
    let declaration = ApplicationSchemaDeclarationBuilder::<FixtureSchema>::for_schema()
        .operation(definition)
        .build()
        .expect("the matching operation builder associates the aftermath");
    declaration
        .erased()
        .members()
        .iter()
        .cloned()
        .map(|mut member| {
            match &mut member {
                ApplicationSchemaMember::Operation { operation, .. }
                | ApplicationSchemaMember::OperationAftermath { operation, .. } => {
                    *operation = operation_slot.to_owned();
                }
                _ => {}
            }
            member
        })
        .collect()
}

pub(crate) fn protocol(version: u32) -> ApplicationExternalEffectProtocol {
    ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("test.escaping-payload"),
        BoundaryProtocolVersion::new(version),
    )
}

pub(crate) fn binding(
    package: CanonicalDigestId,
    schema: CanonicalDigestId,
    generation: u64,
) -> ApplicationSchemaBindingIdentity {
    ApplicationSchemaBindingIdentity::from_installed_parts(1, generation, package, schema)
}

pub(crate) fn recorded_inverse<Field>() -> DeclaredCorrectionMechanism<FixtureSchema>
where
    Field: ApplicationFieldMarkerIdentity<FixtureSchema, Account, State, Value = u64>,
{
    recorded_inverse_at::<Account, State, Field>(256)
}

pub(crate) fn recorded_inverse_at<Entity, Aspect, Field>(
    maximum_encoded_bytes: usize,
) -> DeclaredCorrectionMechanism<FixtureSchema>
where
    Entity: ApplicationEntityMarkerIdentity<FixtureSchema>,
    Aspect: ApplicationAspectMarkerIdentity<FixtureSchema, Entity>,
    Field: ApplicationFieldMarkerIdentity<FixtureSchema, Entity, Aspect, Value = u64>,
{
    let field =
        ApplicationFieldRef::<FixtureSchema, Entity, Aspect, Field, u64>::from_schema_types();
    DeclaredCorrectionMechanism::RecordedInverse(
        DeclaredRecordedInverse::new(
            "unfreeze",
            DeclaredLoweringCorrespondenceRef::new("geometry-inverse").unwrap(),
            DeclaredAftermathPostcondition::ExactPriorTruth,
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(field)],
                maximum_encoded_bytes,
            )
            .unwrap(),
        )
        .unwrap(),
    )
}

fn decision_read_target<Entity, Aspect, Field>() -> ApplicationOperationDecisionReadTarget
where
    Entity: ApplicationEntityMarkerIdentity<FixtureSchema>,
    Aspect: ApplicationAspectMarkerIdentity<FixtureSchema, Entity>,
    Field: ApplicationFieldMarkerIdentity<FixtureSchema, Entity, Aspect, Value = u64>,
{
    let field =
        ApplicationFieldRef::<FixtureSchema, Entity, Aspect, Field, u64>::from_schema_types();
    ApplicationOperationDecisionReadTarget::Field {
        entity: field.entity().to_owned(),
        aspect: field.aspect().to_owned(),
        field: field.field().to_owned(),
    }
}

pub(crate) fn compensation() -> DeclaredCorrectionMechanism<FixtureSchema> {
    DeclaredCorrectionMechanism::Compensation(
        DeclaredCompensation::new(
            "compensating-journal",
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: "settled".into(),
            },
        )
        .unwrap(),
    )
}
