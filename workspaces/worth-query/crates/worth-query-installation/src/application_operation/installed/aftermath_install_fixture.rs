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
    ApplicationExternalEffectProtocol, ApplicationOperationDecisionReadTarget,
    ApplicationOperationMarkerIdentity, ApplicationOperationProgramTarget, ApplicationOperationRef,
    ApplicationSchema, ApplicationSchemaBindingIdentity, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
    WorthQueryExternalEffectCorrelationFamily,
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
mod records;
pub(crate) use records::*;
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;

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

    pub(crate) fn reads<Field: RecordSelection>(mut self, field: Field) -> Self {
        self.declared_reads = vec![decision_read_target(field)];
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
    let declaration =
        ApplicationSchemaDeclarationBuilder::<FixtureSchema>::from_fragment(AftermathFields)
            .expect("the actual inverse fields lower")
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

pub(crate) fn recorded_inverse<Field: RecordSelection>(
    field: Field,
) -> DeclaredCorrectionMechanism<FixtureSchema> {
    recorded_inverse_at(field, 256)
}
pub(crate) fn recorded_inverse_at<Field: RecordSelection>(
    field: Field,
    maximum_encoded_bytes: usize,
) -> DeclaredCorrectionMechanism<FixtureSchema> {
    DeclaredCorrectionMechanism::RecordedInverse(DeclaredRecordedInverse::new(
        "unfreeze",
        DeclaredLoweringCorrespondenceRef::new("geometry-inverse"),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(field)],
            maximum_encoded_bytes,
        ),
    ))
}

fn decision_read_target<Field: RecordSelection>(
    field: Field,
) -> ApplicationOperationDecisionReadTarget {
    ApplicationOperationDecisionReadTarget::Field {
        entity: field.entity().to_owned(),
        aspect: field.aspect().to_owned(),
        field: field.field().to_owned(),
    }
}

pub(crate) fn compensation() -> DeclaredCorrectionMechanism<FixtureSchema> {
    DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new(
        "compensating-journal",
        DeclaredAftermathPostcondition::BusinessPostcondition {
            identity: "settled".into(),
        },
    ))
}
