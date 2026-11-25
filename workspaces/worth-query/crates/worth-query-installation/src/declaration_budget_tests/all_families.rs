//! A freshly validated portable schema, assembled from real declaration fixtures.
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
mod graph;
mod history;
mod operation;

pub(crate) fn declaration() -> ErasedApplicationSchemaDeclaration {
    let mut sources = crate::application_schema_tests::budget_declarations();
    sources.push(crate::application_principal_binding_tests::budget_declaration());
    sources.push(crate::application_capability::tests::warm_lookup::budget_declaration());
    sources.push(history::index_declaration());
    let mut members: Vec<_> = sources
        .iter()
        .flat_map(|s| s.members().iter().cloned())
        .collect();
    graph::extend(&mut members);
    history::extend(&mut members);
    operation::extend(&mut members);
    let entity = members
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Entity { entity } => Some(entity.clone()),
            _ => None,
        })
        .expect("the complete fixture declares entities");
    let definition = ApplicationDecisionReadScopeMeaning::from_untrusted_parts(
        ARCHIVE_DECODE,
        vec![ApplicationOperationDecisionReadTarget::Entity {
            entity: entity.clone(),
        }],
        Vec::new(),
    )
    .unwrap();
    members.push(ApplicationSchemaMember::ApplicationNameScheme {
        definition: worth_query_declaration::facade::application_naming::ErasedApplicationNameScheme::from_untrusted_parts(ARCHIVE_DECODE,
            "worth.budget.entity.v1".into(),
            worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind::Entity,
            entity,
            vec!["authored".into()],
            Vec::new(),
            false,
            None,
            Vec::new(),
            true,
        ),
    });
    members.push(ApplicationSchemaMember::ApplicationGroupNameScheme {
        definition: worth_query_declaration::facade::application_naming::ErasedApplicationGroupNameScheme::from_untrusted_parts(ARCHIVE_DECODE,
            worth_query_declaration::facade::application_naming::ErasedApplicationNameScheme::from_untrusted_parts(ARCHIVE_DECODE,
                "worth.budget.group.v1".into(),
                worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind::Group,
                "BudgetLoop".into(),
                vec!["anchor_face".into(), "edge".into()],
                Vec::new(),
                false,
                None,
                Vec::new(),
                true,
            ),
            worth_query_declaration::facade::application_naming::ApplicationGroupContinuityRule::Anchor,
            Some("anchor_face".into()),
            None,
        ),
    });
    members.push(ApplicationSchemaMember::Operation {
        operation: "SharedBudgetRead".into(),
        input_type: portable("worth.rust.unit"),
    });
    members.push(ApplicationSchemaMember::OperationDecisionReadScope {
        operation: "SharedBudgetRead".into(),
        scope: definition.identity().into(),
    });
    members.push(ApplicationSchemaMember::DecisionReadScope { definition });
    members.sort();
    members.dedup();
    validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableApplicationSchemaParts {
                owner: "worth.budget-fixture".into(),
                name: "AllDeclarationFamilies".into(),
                major: 1,
                minor: 0,
                members,
                contributions: Vec::new(),
            },
        ),
    )
    .expect("the complete schema must pass real identifier, closure and contract validation")
}

fn portable(name: &'static str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::declared(name)
}
