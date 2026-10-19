//! Transported graph-target writes cannot lose their closure or typed field contract.
use super::program;
use worth_query_decl::facade::application_schema::*;
#[test]
fn typed_finish_refuses_duplicate_declared_path_destinations() {
    use super::{
        path_value_targets::Copy,
        schema::{ParentRevision, ParentRows, Row, RowValue, Schema},
    };
    use worth_query_decl::facade::application_query::ApplicationQueryRootPath;
    let op = Copy::reference();
    let operands = op
        .requirement_operands()
        .field("value", RowValue::reference())
        .unwrap();
    let expression = worth_foundational::expression_api::expressions()
        .reading_within(ApplicationOperationBodyExpression::PROFILE)
        .parse("value")
        .unwrap();
    let value = ApplicationOperationBodyValue::<Schema, Copy, U64ApplicationValueBinding>::admit(
        "value", operands, expression,
    )
    .unwrap();
    let path = || ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference());
    let body = op
        .body()
        .assign_value_at_path(path(), ParentRevision::reference(), &value)
        .unwrap()
        .assign_value_at_path(path(), ParentRevision::reference(), &value)
        .unwrap()
        .finish();
    assert!(matches!(
        body,
        Err(ApplicationOperationBodyAuthoringDenial::DuplicateAssignment)
    ));
}
#[test]
fn computed_path_write_readmission_refuses_weakened_target_correspondence() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(record.clone()).unwrap(),
        *schema.erased()
    );
    for defect in [
        "read",
        "write",
        "scope",
        "binding",
        "value",
        "presence",
        "optional",
        "readonly",
        "unit",
        "duplicate",
    ] {
        let mut parts = record.clone().into_parts();
        let mut changed = false;
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if defect == "read"
                        && operation == "Bump"
                        && matches!(&*target, ApplicationOperationDecisionReadTarget::Relation { relation, .. } if relation == "ParentRows") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::OperationProgram { operation, target }
                    if defect == "write"
                        && operation == "Bump"
                        && matches!(&*target, ApplicationOperationProgramTarget::Write { field, .. } if field == "ParentRevision") =>
                {
                    *target = ApplicationOperationProgramTarget::Create {
                        entity: "Parent".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::Field {
                    entity,
                    field,
                    writable,
                    unit,
                    presence,
                    ..
                } if entity == "Parent" && field == "ParentRevision" => match defect {
                    "optional" => {
                        *presence = ApplicationFieldPresence::Optional;
                        changed = true;
                    }
                    "readonly" => {
                        *writable = false;
                        changed = true;
                    }
                    "unit" => {
                        *unit = Some("revision-units".into());
                        changed = true;
                    }
                    _ => {}
                },
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "Bump"
                        && matches!(
                            defect,
                            "scope" | "binding" | "value" | "presence" | "duplicate"
                        ) =>
                {
                    let mut steps = body.path_value_assignments().to_vec();
                    match defect {
                        "scope" => steps[0].target.path = worth_query_decl::facade::application_query::ApplicationQueryRootPathMeaning::from_untrusted_parts(worth_query_decl::facade::application_query::WorthQueryPortableApplicationQueryRootPathParts {start_entity:"Parent".into(),terminal_entity:"Parent".into(),steps:vec![],guards:vec![]}),
                        "binding" => steps[0].binding = StringApplicationValueBinding::IDENTITY,
                        "value" => steps[0].value = "not-declared".into(),
                        "presence" => steps[0].target.presence = ApplicationFieldPresence::Optional,
                        "duplicate" => steps.push(steps[0].clone()),
                        _ => unreachable!(),
                    }
                    *body = body.clone().with_untrusted_path_value_assignments(steps);
                    changed = true;
                }
                _ => {}
            }
        }
        assert!(changed, "{defect}");
        parts.members.sort();
        let denial = validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap_err();
        assert_eq!(
            denial,
            match defect {
                "duplicate" => ApplicationSchemaDeclarationDenial::DuplicateMember,
                "unit" => ApplicationSchemaDeclarationDenial::MissingUnit,
                "write" | "optional" | "readonly" =>
                    ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
                _ => ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency,
            },
            "{defect}"
        );
    }
}
