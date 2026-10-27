use super::*;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField,
    ApplicationOperationBodyCreation, ApplicationOperationBodyProgram,
    ApplicationOperationBodyScopeField,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{AspectValue, ScalarAspectType};

#[test]
fn portable_scope_initializer_requires_matching_declared_units() {
    let scope = ApplicationFieldBindingLocus::new("Item", "Facts", "Current");
    let source = ApplicationFieldBindingLocus::new("Item", "Facts", "Prior");
    let target = ApplicationFieldBindingLocus::new("Audit", "Facts", "Prior");
    let codec = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".to_owned());
    let scope_step = ApplicationOperationBodyScopeField::Assign {
        field: scope.clone(),
        binding: codec.clone(),
        value: AspectValue::UInt64(20),
    };
    let created_step = ApplicationOperationBodyCreatedField::ScopeField {
        field: target.clone(),
        binding: codec,
        scalar_family: ScalarAspectType::UInt64,
        source: source.clone(),
    };
    let field = |locus: &ApplicationFieldBindingLocus, unit: &str| ApplicationSchemaMember::Field {
        canonical: None,
        entity: locus.entity().to_owned(),
        aspect: locus.aspect().to_owned(),
        field: locus.field().to_owned(),
        presence: ApplicationFieldPresence::Required,
        scalar_family: ScalarAspectType::UInt64,
        value_type: "worth.rust.u64".to_owned(),
        unit: Some(unit.to_owned()),
        frame: None,
        writable: true,
        equality_queryable: true,
        unique: false,
    };
    let mut members = vec![
        ApplicationSchemaMember::Entity {
            entity: "Audit".to_owned(),
        },
        ApplicationSchemaMember::Operation {
            operation: "Set".to_owned(),
            input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.rust.unit".to_owned(),
            ),
        },
        field(&scope, "days"),
        field(&source, "days"),
        field(&target, "days"),
        ApplicationSchemaMember::OperationDecisionRead {
            operation: "Set".to_owned(),
            target: scope_step.decision_read(),
        },
        ApplicationSchemaMember::OperationDecisionRead {
            operation: "Set".to_owned(),
            target: created_step.source_read().unwrap(),
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "Set".to_owned(),
            target: scope_step.write_target().unwrap(),
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "Set".to_owned(),
            target: ApplicationOperationProgramTarget::Create {
                entity: "Audit".to_owned(),
            },
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "Set".to_owned(),
            target: created_step.write_target(),
        },
        ApplicationSchemaMember::OperationBody {
            operation: "Set".to_owned(),
            body: ApplicationOperationBodyProgram::from_untrusted_with_creation(
                vec![scope_step],
                Some(ApplicationOperationBodyCreation::from_untrusted(
                    "Audit".to_owned(),
                    "audit".to_owned(),
                    vec![created_step],
                )),
            ),
        },
        ApplicationSchemaMember::Entity {
            entity: "Item".to_owned(),
        },
    ];
    assert_eq!(validate_operation_bodies(&members), Ok(()));
    let ApplicationSchemaMember::Field {
        canonical: None,
        unit,
        ..
    } = &mut members[4]
    else {
        unreachable!()
    };
    *unit = Some("hours".to_owned());
    assert!(matches!(
        validate_operation_bodies(&members),
        Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency { .. })
    ));
}

#[test]
fn portable_optional_clear_refuses_a_required_field() {
    let field = ApplicationFieldBindingLocus::new("Item", "Facts", "Note");
    let step = ApplicationOperationBodyScopeField::ClearOptional {
        field: field.clone(),
        binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".to_owned()),
        scalar_family: ScalarAspectType::String,
    };
    let mut members = vec![
        ApplicationSchemaMember::Operation {
            operation: "Clear".to_owned(),
            input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.rust.unit".to_owned(),
            ),
        },
        ApplicationSchemaMember::Field {
            canonical: None,
            entity: "Item".to_owned(),
            aspect: "Facts".to_owned(),
            field: "Note".to_owned(),
            presence: ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::String,
            value_type: "worth.rust.string".to_owned(),
            unit: None,
            frame: None,
            writable: true,
            equality_queryable: false,
            unique: false,
        },
        ApplicationSchemaMember::OperationDecisionRead {
            operation: "Clear".to_owned(),
            target: step.decision_read(),
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "Clear".to_owned(),
            target: step.write_target().unwrap(),
        },
        ApplicationSchemaMember::OperationBody {
            operation: "Clear".to_owned(),
            body: ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![step]),
        },
        ApplicationSchemaMember::Entity {
            entity: "Item".to_owned(),
        },
    ];
    assert!(matches!(
        validate_operation_bodies(&members),
        Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency { .. })
    ));
    let ApplicationSchemaMember::Field {
        canonical: None,
        presence,
        ..
    } = &mut members[1]
    else {
        unreachable!()
    };
    *presence = ApplicationFieldPresence::Optional;
    assert_eq!(validate_operation_bodies(&members), Ok(()));
}
