//! Cold guard meaning must retain source topology, codec, presence and read closure.
use super::program;
use worth_query_decl::facade::{application_query::*, application_schema::*};
#[test]
fn portable_path_guard_reconstructs_scope_and_refuses_missing_or_changed_correspondence() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(record.clone()).unwrap(),
        *schema.erased()
    );
    for defect in [
        "read",
        "scope",
        "codec",
        "presence",
        "member",
        "direction",
        "guarded",
    ] {
        let mut parts = record.clone().into_parts();
        let mut changed = false;
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if defect == "read"
                        && operation == "Save"
                        && matches!(&*target, ApplicationOperationDecisionReadTarget::Relation { relation, .. } if relation == "ParentRows") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "Save" && defect != "read" =>
                {
                    let mut guards = body.path_input_expectations().to_vec();
                    let guard = &mut guards[0];
                    match defect {
                        "scope" => {
                            guard.source.path =
                                ApplicationQueryRootPathMeaning::from_untrusted_parts(
                                    WorthQueryPortableApplicationQueryRootPathParts {
                                        start_entity: "Parent".into(),
                                        terminal_entity: "Parent".into(),
                                        steps: vec![],
                                        guards: vec![],
                                    },
                                )
                        }
                        "direction" => {
                            guard.source.path =
                                ApplicationQueryRootPathMeaning::from_untrusted_parts(
                                    WorthQueryPortableApplicationQueryRootPathParts {
                                        start_entity: "Row".into(),
                                        terminal_entity: "Parent".into(),
                                        steps: vec![
                                            ApplicationQueryRootPathStep::from_untrusted_fields(
                                                "ParentRows".into(),
                                                "Parent".into(),
                                                "Row".into(),
                                                ApplicationQueryRootPathDirection::Forward,
                                            ),
                                        ],
                                        guards: vec![],
                                    },
                                )
                        }
                        "guarded" => guard.source.path = ApplicationQueryRootPathMeaning::from_untrusted_parts(WorthQueryPortableApplicationQueryRootPathParts {
                            start_entity: "Row".into(), terminal_entity: "Parent".into(), steps: guard.source.path.steps().to_vec(),
                            guards: vec![ApplicationQueryRootPathGuard::from_untrusted_parts(WorthQueryPortableApplicationQueryRootPathGuardParts {
                                after_step: 1, entity: "Parent".into(), aspect: "ParentFacts".into(), field: "Company".into(), scalar_family: worth_foundational::facade::ScalarAspectType::String,
                                value_type: StringApplicationValueBinding::IDENTITY, expected: worth_foundational::facade::AspectValue::String("company-a".into()),
                            })],
                        }),
                        "codec" => guard.binding = U64ApplicationValueBinding::IDENTITY,
                        "presence" => guard.source.presence = ApplicationFieldPresence::Required,
                        "member" => guard.member = " company".into(),
                        _ => unreachable!(),
                    }
                    *body = body.clone().with_untrusted_path_input_expectations(guards);
                    changed = true;
                }
                _ => {}
            }
        }
        assert!(changed);
        parts.members.sort();
        assert_eq!(
            validate_portable_application_schema_freshly(
                WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
            )
            .unwrap_err(),
            ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency {
                operation: "Save".into(),
                dependency: ApplicationOperationDependencyLocus::Body,
            },
            "{defect}"
        );
    }
}
