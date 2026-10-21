//! Flat prior-parent topology, exact field closure and context rules survive fresh readmission.
use super::program;
use worth_query_decl::facade::application_schema::*;
#[test]
fn transported_children_cannot_change_parent_topology_or_drop_contracts() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert!(validate_portable_application_schema_freshly(record.clone()).is_ok());
    for defect in [
        "cycle",
        "duplicate-key",
        "wrong-endpoint",
        "missing-field",
        "missing-link",
        "shared-owner",
        "wrong-secondary-parent",
        "duplicate-secondary-key",
        "missing-secondary-link",
        "shared-secondary-owner",
    ] {
        let mut parts = record.clone().into_parts();
        let mut changed = false;
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "Create"
                        && matches!(
                            defect,
                            "cycle"
                                | "duplicate-key"
                                | "wrong-endpoint"
                                | "missing-field"
                                | "wrong-secondary-parent"
                                | "duplicate-secondary-key"
                        ) =>
                {
                    let root = body.creation().unwrap();
                    let mut children = root.children().to_vec();
                    match defect {
                        "cycle" => children[0].parent_key = "detail".into(),
                        "duplicate-key" => children[2].creation = children[0].creation.clone(),
                        "wrong-endpoint" => {
                            let mut links = children[0].creation.links().to_vec();
                            links[0].from = "Parent".into();
                            children[0].creation =
                                children[0].creation.clone().with_untrusted_links(links);
                        }
                        "wrong-secondary-parent" | "duplicate-secondary-key" => {
                            let mut links = children[0].creation.links().to_vec();
                            if defect == "wrong-secondary-parent" {
                                links[1].from = "Parent".into();
                            } else {
                                links[1].key = links[0].key.clone();
                            }
                            children[0].creation =
                                children[0].creation.clone().with_untrusted_links(links);
                        }
                        "missing-field" => {
                            let c = &children[2].creation;
                            let fields = c
                                .fields()
                                .iter()
                                .filter(|f| f.field().field() != "CellKind")
                                .cloned()
                                .collect();
                            children[2].creation =
                                ApplicationOperationBodyCreation::from_untrusted(
                                    c.entity().into(),
                                    c.key().into(),
                                    fields,
                                )
                                .with_untrusted_links(c.links().to_vec());
                        }
                        _ => unreachable!(),
                    }
                    *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                        "Parent".into(),
                        body.scope_fields().to_vec(),
                        Some(root.clone().with_untrusted_children(children)),
                        vec![],
                    )
                    .with_untrusted_values(body.values().to_vec())
                    .with_untrusted_path_input_expectations(
                        body.path_input_expectations().to_vec(),
                    );
                    changed = true;
                }
                ApplicationSchemaMember::OperationProgram { operation, target }
                    if operation == "Create"
                        && matches!(defect, "missing-link" | "missing-secondary-link")
                        && matches!(&*target,ApplicationOperationProgramTarget::Link{relation,..} if relation==if defect=="missing-link" {"CellDetails"}else{"RowCellArchive"}) =>
                {
                    *target = ApplicationOperationProgramTarget::Create {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::Relation {
                    relation,
                    integrity,
                    ..
                } if (relation == "RowCells" && defect == "shared-owner")
                    || (relation == "RowCellArchive" && defect == "shared-secondary-owner") =>
                {
                    *integrity =
                        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
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
            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
            "{defect}"
        );
    }
}
