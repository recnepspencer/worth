//! Fresh readmission cannot discard peer authority or alias creation-site identity.
use super::program;
use worth_query_decl::facade::{application_query::*, application_schema::*};
fn replace_root(
    body: &ApplicationOperationBodyProgram,
    root: ApplicationOperationBodyCreation,
) -> ApplicationOperationBodyProgram {
    ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        body.scope_fields().to_vec(),
        Some(root),
        body.requirements().to_vec(),
    )
    .with_untrusted_values(body.values().to_vec())
    .with_untrusted_path_value_assignments(body.path_value_assignments().to_vec())
    .with_untrusted_outputs(body.outputs().to_vec())
}
#[test]
fn portable_independent_sites_require_exact_scope_keys_source_and_effect_closure() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert!(validate_portable_application_schema_freshly(record.clone()).is_ok());
    for defect in [
        "duplicate-key",
        "nested-peer",
        "wrong-scope",
        "reusable-path",
        "missing-actor-read",
        "missing-link",
        "wrong-principal",
    ] {
        let mut parts = record.clone().into_parts();
        let mut changed = false;
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "Reuse" && !defect.starts_with("missing-") =>
                {
                    let root = body.creation().unwrap();
                    let mut peers = root.peers().to_vec();
                    match defect {
                        "duplicate-key" => {
                            let peer = &peers[0];
                            peers[0] = ApplicationOperationBodyCreation::from_untrusted(
                                peer.entity().into(),
                                root.key().into(),
                                peer.fields().to_vec(),
                            )
                            .with_untrusted_path_links(peer.path_links().to_vec());
                        }
                        "nested-peer" => {
                            peers[0] = peers[0]
                                .clone()
                                .with_untrusted_peers(vec![peers[0].clone()])
                        }
                        "wrong-scope" => {
                            let mut paths = peers[0].path_links().to_vec();
                            paths[0].path = ApplicationQueryRootPathMeaning::from_untrusted_parts(
                                WorthQueryPortableApplicationQueryRootPathParts {
                                    start_entity: "Parent".into(),
                                    terminal_entity: "Parent".into(),
                                    steps: vec![],
                                    guards: vec![],
                                },
                            );
                            peers[0] = peers[0].clone().with_untrusted_path_links(paths);
                        }
                        "reusable-path" => {
                            let path = ApplicationOperationBodyCreationPathLink {
                                path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
                                    WorthQueryPortableApplicationQueryRootPathParts {
                                        start_entity: "Row".into(),
                                        terminal_entity: "Row".into(),
                                        steps: vec![],
                                        guards: vec![],
                                    },
                                ),
                                link: ApplicationOperationBodyCreationLink {
                                    posture: ApplicationOperationBodyCreationLinkPosture::Append,
                                    ..root.links()[0].clone()
                                },
                            };
                            *body = replace_root(
                                body,
                                root.clone().with_untrusted_path_links(vec![path]),
                            );
                            changed = true;
                            continue;
                        }
                        "wrong-principal" => {
                            let peer = &peers[0];
                            let mut fields = peer.fields().to_vec();
                            let ApplicationOperationBodyCreatedField::PrincipalIdentity {
                                mutation_binding,
                                ..
                            } = &mut fields[2]
                            else {
                                panic!("actor source");
                            };
                            *mutation_binding = "another-binding".into();
                            peers[0] = ApplicationOperationBodyCreation::from_untrusted(
                                peer.entity().into(),
                                peer.key().into(),
                                fields,
                            )
                            .with_untrusted_path_links(peer.path_links().to_vec());
                        }
                        _ => unreachable!(),
                    }
                    *body = replace_root(body, root.clone().with_untrusted_peers(peers));
                    changed = true;
                }
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if operation == "Reuse"
                        && defect == "missing-actor-read"
                        && matches!(&*target,ApplicationOperationDecisionReadTarget::Field{entity,field,..} if entity=="Principal"&&field=="PrincipalIdentity") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::OperationProgram { operation, target }
                    if operation == "Reuse"
                        && defect == "missing-link"
                        && matches!(&*target,ApplicationOperationProgramTarget::Link{relation,..} if relation=="ParentJournal") =>
                {
                    *target = ApplicationOperationProgramTarget::Create {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                _ => {}
            }
        }
        assert!(changed, "{defect}");
        parts.members.sort();
        let expected = if defect == "missing-actor-read" {
            ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency
        } else {
            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency
        };
        assert_eq!(
            validate_portable_application_schema_freshly(
                WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
            )
            .unwrap_err(),
            expected,
            "{defect}"
        );
    }
}
