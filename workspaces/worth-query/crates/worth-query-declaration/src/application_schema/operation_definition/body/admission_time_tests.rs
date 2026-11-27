//! Cold closure admits only the native, required, unitless timestamp target,
//! so a forged portable record cannot stamp admission time into another codec.
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldPresence, ApplicationOperationBodyCreatedField,
    ApplicationOperationBodyCreation, ApplicationOperationBodyProgram,
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
    ApplicationScalarValueBinding, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
    CanonicalTimestampApplicationValueBinding as Timestamp,
};
use crate::archive_decode_authority::ARCHIVE_DECODE;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::ScalarAspectType;

fn members(
    binding: &str,
    presence: ApplicationFieldPresence,
    unit: Option<&str>,
) -> Vec<ApplicationSchemaMember> {
    let target = ApplicationFieldBindingLocus::new("Ticket", "Facts", "CreatedAt");
    let step = ApplicationOperationBodyCreatedField::AdmissionTime {
        field: target.clone(),
        binding: WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, binding.into()),
    };
    let program = |target| ApplicationSchemaMember::OperationProgram {
        operation: "Open".into(),
        target,
    };
    vec![
        ApplicationSchemaMember::Entity {
            entity: "Desk".into(),
        },
        ApplicationSchemaMember::Entity {
            entity: "Ticket".into(),
        },
        ApplicationSchemaMember::Operation {
            operation: "Open".into(),
            input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "worth.rust.unit".into(),
            ),
        },
        ApplicationSchemaMember::Field {
            content_format: None,
            canonical: None,
            entity: "Ticket".into(),
            aspect: "Facts".into(),
            field: "CreatedAt".into(),
            presence,
            scalar_family: ScalarAspectType::Timestamp,
            value_type: binding.into(),
            unit: unit.map(str::to_owned),
            frame: None,
            writable: false,
            equality_queryable: false,
            unique: false,
        },
        ApplicationSchemaMember::OperationDecisionRead {
            operation: "Open".into(),
            target: ApplicationOperationDecisionReadTarget::Entity {
                entity: "Desk".into(),
            },
        },
        program(ApplicationOperationProgramTarget::Create {
            entity: "Ticket".into(),
        }),
        program(step.write_target()),
        ApplicationSchemaMember::OperationBody {
            operation: "Open".into(),
            body: ApplicationOperationBodyProgram::from_untrusted_in_scope(
                ARCHIVE_DECODE,
                "Desk".into(),
                vec![],
                Some(ApplicationOperationBodyCreation::from_untrusted(
                    ARCHIVE_DECODE,
                    "Ticket".into(),
                    "ticket".into(),
                    vec![step],
                )),
                vec![],
            ),
        },
    ]
}

#[test]
fn only_a_required_unitless_native_timestamp_target_admits_admission_time() {
    let native = Timestamp::IDENTITY;
    let required = ApplicationFieldPresence::Required;
    let validate = super::super::validation::validate_operation_bodies;
    assert_eq!(validate(&members(native.as_str(), required, None)), Ok(()));
    let refused = Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
        "Open",
    ));
    // A custom codec of the same scalar family is not the trusted instant's codec.
    assert_eq!(
        validate(&members("app.custom.timestamp", required, None)),
        refused
    );
    let optional = ApplicationFieldPresence::Optional;
    assert_eq!(validate(&members(native.as_str(), optional, None)), refused);
    assert_eq!(
        validate(&members(native.as_str(), required, Some("utc"))),
        refused
    );
}
