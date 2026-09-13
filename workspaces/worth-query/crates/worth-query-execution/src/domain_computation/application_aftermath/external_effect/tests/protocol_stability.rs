use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationRetainedEffectBinding,
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{
    InstalledExternalEffectContract, WorthQueryInstallationAdmissionProfile,
    WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

use super::{
    derive_external_effect_correlation_identity, ExternalEffectCorrelationBasis,
    WorthQueryDispatchOutboxRecord,
};

const FAMILY: BoundaryProtocolIdentity = BoundaryProtocolIdentity::new("test.protocol-stability");
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
const BYTES: [u8; 8] = *b"frozen-v";

macro_rules! moved_payload_module {
    ($module:ident) => {
        mod $module {
            use super::*;

            pub(super) struct ProtocolSchema;
            pub(super) struct Payload;
            worth_query_declaration::worth_query_portable_type!(
                Payload => "worth.query.test.protocol-stability-payload.v1"
            );
            worth_query_declaration::worth_query_structured_value_binding!(pub(super) PayloadBinding for Payload { identity: "worth.query.test.protocol-stability-payload.v1" });

            worth_query_declaration::worth_query_operation!(
                pub(super) Notify for ProtocolSchema, input PayloadBinding
            );
            worth_query_declaration::worth_query_effect!(
                pub(super) Notice for ProtocolSchema, payload PayloadBinding
            );
            worth_query_declaration::worth_query_operation_emits!(Notify => [Notice]);

            impl ApplicationRetainedEffectBinding for PayloadBinding {
                fn retained_bytes(_value: &Self::Value) -> u64 {
                    std::mem::size_of::<Self::Value>() as u64
                }
            }

            impl ApplicationExternalEffectBinding for PayloadBinding {
                const PROTOCOL: ApplicationExternalEffectProtocol =
                    ApplicationExternalEffectProtocol::new(FAMILY, VERSION);
                const MAX_EXTERNAL_BYTES: u64 = BYTES.len() as u64;

                fn external_effect_bytes(_value: &Self::Value) -> Vec<u8> {
                    BYTES.to_vec()
                }
            }

            impl ApplicationSchema for ProtocolSchema {
                const OWNER: &'static str = "protocol-stability";
                const NAME: &'static str = "ProtocolSchema";
                const MAJOR: u32 = 1;
                const MINOR: u32 = 0;

                fn declaration() -> Result<
                    ApplicationSchemaDeclaration<Self>,
                    worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
                > {
                    let operation = Notify::reference();
                    ApplicationSchemaDeclarationBuilder::for_schema()
                        .effect(Notice::reference())
                        .operation(
                            operation
                                .definition()
                                .external_effect(
                                    Notice::reference(),
                                    worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                                        "protocol-test-rail",
                                    )
                                    .unwrap(),
                                )
                                .no_aftermath()
                                .finish(),
                        )
                        .operation_decision_fact_budget(operation, 1)
                        .operation_projection_work_budget(operation, 1)
                        .operation_emit(operation, Notice::reference())
                        .build()
                }
            }
        }
    };
}

moved_payload_module!(original_location);
moved_payload_module!(moved_location);

#[test]
fn rust_module_move_cannot_change_installed_or_outbox_protocol_identity() {
    let original_contract = installed_contract::<
        original_location::ProtocolSchema,
        original_location::Notify,
        original_location::Payload,
    >(original_location::Notify::reference());
    let moved_contract = installed_contract::<
        moved_location::ProtocolSchema,
        moved_location::Notify,
        moved_location::Payload,
    >(moved_location::Notify::reference());

    assert_ne!(
        std::any::type_name::<original_location::Payload>(),
        std::any::type_name::<moved_location::Payload>()
    );
    assert_eq!(
        rust_payload_type(&original_contract),
        rust_payload_type(&moved_contract)
    );
    assert_eq!(original_contract.protocol(), moved_contract.protocol());

    let original_payload =
        original_location::PayloadBinding::external_effect_bytes(&original_location::Payload);
    let moved_payload =
        moved_location::PayloadBinding::external_effect_bytes(&moved_location::Payload);
    assert_eq!(original_payload, BYTES);
    assert_eq!(moved_payload, BYTES);

    let original_outbox = outbox(&original_contract, original_payload);
    let moved_outbox = outbox(&moved_contract, moved_payload);
    for record in [&original_outbox, &moved_outbox] {
        assert_eq!(record.protocol_identity(), &FAMILY);
        assert_eq!(record.protocol_version(), VERSION);
        assert_eq!(record.payload(), BYTES);
    }
}

fn installed_contract<Schema, Operation, Input>(
    operation: ApplicationOperationRef<Schema, Operation, Input>,
) -> InstalledExternalEffectContract
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: worth_query_declaration::facade::portable_identity::WorthQueryPortableType + 'static,
{
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ))
    .application_schema(Schema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = worth_query_installation::facade::WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    index
        .bind_application_schema(Schema::declaration().unwrap())
        .unwrap()
        .installed_operation(operation)
        .unwrap()
        .contracts()
        .external_effect()
        .clone()
}

fn rust_payload_type(contract: &InstalledExternalEffectContract) -> &str {
    match contract {
        InstalledExternalEffectContract::Declared {
            rust_payload_type, ..
        } => rust_payload_type.as_str(),
        InstalledExternalEffectContract::None => panic!("test operation must escape"),
    }
}

fn outbox(
    contract: &InstalledExternalEffectContract,
    payload: Vec<u8>,
) -> WorthQueryDispatchOutboxRecord {
    let correlation = derive_external_effect_correlation_identity(ExternalEffectCorrelationBasis {
        correlation_family:
            worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                "protocol-test-rail",
            )
            .unwrap(),
        operation_slot: "Notify",
        operation_version: 1,
        outcome_identity: 9,
        idempotency_key: &[0x5A; 32],
        branch: "main",
    })
    .unwrap();
    WorthQueryDispatchOutboxRecord::from_installed_contract(correlation, contract, payload, 9)
        .unwrap()
}
