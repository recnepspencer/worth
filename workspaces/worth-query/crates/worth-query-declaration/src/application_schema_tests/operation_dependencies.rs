//! Real declaration closure errors must retain their rejected operation and target.
use super::*;
use crate::application_schema::{
    ApplicationMutationPreconditionFamily, ApplicationMutationPreconditionTarget,
    ApplicationOperationDecisionReadTarget as Read, ApplicationOperationDependencyLocus as Locus,
    ApplicationOperationProgramTarget as Program, ApplicationOperationProgramTarget,
    ApplicationRelationIntegrity, ApplicationSchemaMember as Member, ApplicationSchemaMember,
};

fn write_members() -> Vec<ApplicationSchemaMember> {
    let declaration = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .entity(entity())
        .aspect(entity(), aspect())
        .field(entity(), field())
        .operation(
            schema_operation()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .build()
        .unwrap();
    let mut members = declaration.erased().members().to_vec();
    members.push(ApplicationSchemaMember::OperationProgram {
        operation: "SchemaOperation".to_owned(),
        target: ApplicationOperationProgramTarget::Write {
            entity: "Entity".to_owned(),
            aspect: "Aspect".to_owned(),
            field: "Field".to_owned(),
        },
    });
    members
}

#[test]
fn operation_dependency_denial_identifies_the_missing_write_field() {
    let mut members = write_members();
    ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members.clone())
        .build()
        .expect("complete native declaration succeeds");
    members.retain(|member| !matches!(member, ApplicationSchemaMember::Field { field, .. } if field == "Field"));
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
        .build()
        .unwrap_err();
    assert!(matches!(&denial,
        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency {
            operation, dependency: Locus::ProgramTarget(Program::Write { entity, aspect, field })
        } if operation == "SchemaOperation" && entity == "Entity" && aspect == "Aspect" && field == "Field"));
    let diagnostic = denial.to_string();
    for locus in ["SchemaOperation", "Entity", "Aspect", "Field"] {
        assert!(
            diagnostic.contains(locus),
            "native declaration denial lost {locus}: {diagnostic}"
        );
    }
}

#[test]
fn operation_dependency_denial_identifies_the_missing_operation() {
    let mut members = write_members();
    members.retain(|member| !matches!(member, Member::Operation { .. }));
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
        .build()
        .unwrap_err();
    assert!(matches!(denial,
        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency {
            operation, dependency: Locus::ProgramTarget(Program::Write { field, .. })
        } if operation == "SchemaOperation" && field == "Field"));
}

fn read_members(target: Read) -> Vec<Member> {
    let mut members = write_members();
    members.retain(|member| !matches!(member, Member::OperationProgram { .. }));
    members.push(Member::OperationDecisionRead {
        operation: "SchemaOperation".to_owned(),
        target,
    });
    members
}

#[test]
fn decision_read_denial_identifies_the_missing_field() {
    let mut members = read_members(Read::Field {
        entity: "Entity".to_owned(),
        aspect: "Aspect".to_owned(),
        field: "Field".to_owned(),
    });
    ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members.clone())
        .build()
        .expect("complete read closure succeeds");
    members.retain(|member| !matches!(member, Member::Field { .. }));
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
        .build()
        .unwrap_err();
    assert!(matches!(denial,
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency {
            operation, dependency: Locus::DecisionReadTarget(Read::Field { entity, aspect, field })
        } if operation == "SchemaOperation" && entity == "Entity" && aspect == "Aspect" && field == "Field"));
}

#[test]
fn decision_read_denial_identifies_the_missing_relation_and_endpoints() {
    let mut members = read_members(Read::Relation {
        relation: "Owns".to_owned(),
        from: "Entity".to_owned(),
        to: "Entity".to_owned(),
    });
    members.push(Member::Relation {
        relation: "Owns".to_owned(),
        from: "Entity".to_owned(),
        to: "Entity".to_owned(),
        integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    });
    ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members.clone())
        .build()
        .expect("complete related read closure succeeds");
    members.retain(|member| !matches!(member, Member::Relation { .. }));
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
        .build()
        .unwrap_err();
    assert!(matches!(denial,
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency {
            operation, dependency: Locus::DecisionReadTarget(Read::Relation { relation, from, to })
        } if operation == "SchemaOperation" && relation == "Owns" && from == "Entity" && to == "Entity"));
}

#[test]
fn mutation_precondition_denial_identifies_its_required_field_read() {
    let mut members = read_members(Read::Field {
        entity: "Entity".to_owned(),
        aspect: "Aspect".to_owned(),
        field: "Field".to_owned(),
    });
    members.push(Member::OperationMutationPrecondition {
        operation: "SchemaOperation".to_owned(),
        target: ApplicationMutationPreconditionTarget::field(
            ApplicationMutationPreconditionFamily::ExpectedFact,
            "Entity",
            "Aspect",
            "Field",
        ),
    });
    ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members.clone())
        .build()
        .expect("matching native read and precondition succeed");
    members.retain(|member| !matches!(member, Member::OperationDecisionRead { .. }));
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
        .build()
        .unwrap_err();
    let ApplicationSchemaDeclarationDenial::MissingOperationMutationPreconditionDependency {
        operation,
        dependency: Locus::MutationPrecondition(target),
    } = denial
    else {
        panic!("expected the native missing precondition-read cause: {denial:?}");
    };
    assert_eq!(operation, "SchemaOperation");
    assert_eq!(
        target.family(),
        ApplicationMutationPreconditionFamily::ExpectedFact
    );
    assert_eq!(
        (target.entity(), target.aspect(), target.field_name()),
        ("Entity", "Aspect", "Field")
    );
}
