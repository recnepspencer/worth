//! Disposal requires exact deletion and exclusive-pointer contracts, including after transport.
use super::{program, retiring_actions::*, schema::*, selection_schema::*};
use worth_query_decl::facade::application_schema::*;
#[test]
fn typed_retiring_replacement_refuses_nonexclusive_incoming_ownership() {
    let relation =
        ApplicationRelationRef::<Schema, CurrentSnapshot, Row, Snapshot>::from_schema_identifiers(
            "CurrentSnapshot",
            "Row",
            "Snapshot",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        );
    let result = Retire::reference()
        .body()
        .in_scope(Row::reference())
        .unwrap()
        .create_in_scope(Snapshot::reference(), "snapshot")
        .unwrap()
        .replace_and_retire_from_scope(relation, "current");
    assert!(matches!(
        result,
        Err(ApplicationOperationBodyAuthoringDenial::InvalidCreationLink)
    ));
}
#[test]
fn cold_retiring_replacement_cannot_drop_delete_unlink_or_exclusivity() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(record.clone()).unwrap(),
        *schema.erased()
    );
    for defect in [
        "delete",
        "unlink",
        "source-bound",
        "target-bound",
        "read",
        "direction",
    ] {
        let mut parts = record.clone().into_parts();
        let mut changed = false;
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationProgram { operation, target }
                    if operation == "Retire"
                        && ((defect == "delete"
                            && matches!(&*target,ApplicationOperationProgramTarget::Delete{entity} if entity=="Snapshot"))
                            || (defect == "unlink"
                                && matches!(&*target,ApplicationOperationProgramTarget::Unlink{relation,..} if relation=="CurrentSnapshot"))) =>
                {
                    *target = ApplicationOperationProgramTarget::Delete {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if operation == "Retire"
                        && defect == "read"
                        && matches!(&*target,ApplicationOperationDecisionReadTarget::Relation{relation,..} if relation=="CurrentSnapshot") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::Relation {
                    relation,
                    integrity,
                    ..
                } if relation == "CurrentSnapshot"
                    && matches!(defect, "source-bound" | "target-bound") =>
                {
                    let old = integrity.cardinality;
                    integrity.cardinality = ApplicationRelationCardinality::new(
                        None,
                        if defect == "source-bound" {
                            None
                        } else {
                            old.source_max
                        },
                        None,
                        if defect == "target-bound" {
                            None
                        } else {
                            old.target_max
                        },
                        None,
                        None,
                    );
                    changed = true;
                }
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "Retire" && defect == "direction" =>
                {
                    let creation = body.creation().unwrap();
                    let mut links = creation.links().to_vec();
                    links[0].direction = ApplicationOperationBodyCreationLinkDirection::ToScope;
                    *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                        "Row".into(),
                        body.scope_fields().to_vec(),
                        Some(
                            ApplicationOperationBodyCreation::from_untrusted(
                                creation.entity().into(),
                                creation.key().into(),
                                creation.fields().to_vec(),
                            )
                            .with_untrusted_links(links),
                        ),
                        vec![],
                    );
                    changed = true;
                }
                _ => {}
            }
        }
        assert!(changed, "{defect}");
        parts.members.sort();
        assert_eq!(
            validate_portable_application_schema_freshly(
                WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
            )
            .unwrap_err(),
            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency {
                // Keep also requires the shared source bound and precedes Retire.
                operation: if defect == "source-bound" {
                    "Keep"
                } else {
                    "Retire"
                }
                .into(),
                dependency: ApplicationOperationDependencyLocus::Body,
            },
            "{defect}"
        );
    }
}
#[test]
fn inert_retiring_profile_refuses_preserved_predecessor_signed_and_overlapping_targets() {
    use worth_query_decl::facade::application_operation::{
        ApplicationMutationOutputPosture, ApplicationMutationOutputRoleCardinality,
        ApplicationMutationOutputRoleDescription,
    };
    let schema = program::schema();
    let body = schema
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::OperationBody { operation, body } if operation == "Retire" => {
                Some(body.clone())
            }
            _ => None,
        })
        .unwrap();
    assert!(body.has_supported_retiring_replacement());
    let output = ApplicationOperationBodyOutput {
        binding: "inert-profile".into(),
        role: ApplicationMutationOutputRoleDescription {
            name: "previous".into(),
            entity: "Snapshot".into(),
            posture: ApplicationMutationOutputPosture::Preserve,
            cardinality: ApplicationMutationOutputRoleCardinality::ExactlyOne,
        },
        target: ApplicationOperationBodyOutputTarget::Related {
            relation: "CurrentSnapshot".into(),
            from: "Row".into(),
            to: "Snapshot".into(),
        },
    };
    assert!(!body
        .clone()
        .with_untrusted_outputs(vec![output])
        .has_supported_retiring_replacement());
    let creation = body.creation().unwrap();
    let signed = ApplicationOperationBodyCreation::from_untrusted(
        creation.entity().into(),
        creation.key().into(),
        creation.fields().to_vec(),
    )
    .with_untrusted_links(creation.links().to_vec())
    .with_untrusted_signing(Some(ApplicationOperationBodySigningSpec::from_untrusted(
        "verify".into(),
        ApplicationFieldBindingLocus::new("Snapshot", "Facts", "Actor"),
        ApplicationFieldBindingLocus::new("Snapshot", "Facts", "Provenance"),
    )));
    assert!(!ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        body.scope_fields().to_vec(),
        Some(signed),
        vec![]
    )
    .has_supported_retiring_replacement());
    let path=worth_query_decl::facade::application_query::ApplicationQueryRootPathMeaning::from_untrusted_parts(worth_query_decl::facade::application_query::WorthQueryPortableApplicationQueryRootPathParts {start_entity:"Row".into(),terminal_entity:"Snapshot".into(),steps:vec![worth_query_decl::facade::application_query::ApplicationQueryRootPathStep::from_untrusted_fields("CurrentSnapshot".into(),"Row".into(),"Snapshot".into(),worth_query_decl::facade::application_query::ApplicationQueryRootPathDirection::Forward)],guards:vec![]});
    assert!(!body
        .with_untrusted_path_value_assignments(vec![ApplicationOperationBodyPathValueAssignment {
            target: ApplicationOperationBodyPathFieldSource {
                path,
                field: ApplicationFieldBindingLocus::new(
                    "Snapshot",
                    "SnapshotFacts",
                    "SnapshotValue"
                ),
                presence: ApplicationFieldPresence::Required
            },
            binding: U64ApplicationValueBinding::IDENTITY,
            value: "inert-profile".into()
        }])
        .has_supported_retiring_replacement());
}
