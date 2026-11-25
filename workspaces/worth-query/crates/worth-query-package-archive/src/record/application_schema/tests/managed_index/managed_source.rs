//! A managed source must be publicly disclosed and cannot close a real cycle.
use super::*;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

fn source_query() -> ErasedApplicationQueryDefinition {
    let binding = definition().parts().source.binding.parts().clone();
    ErasedApplicationQueryDefinition::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryParts {
            name: binding.query_name,
            query_type: binding.query.clone(),
            parameter_type: binding.parameters,
            result_type: binding.result.clone(),
            scope_type: binding.scope,
            root_entity: "Document".into(),
            scope_entity: "Document".into(),
            parameters: vec![],
            result_shape: ApplicationQueryResultShape::from_untrusted_parts(
                ARCHIVE_DECODE,
                WorthQueryPortableApplicationQueryResultShapeParts {
                    query_type: binding.query,
                    result_type: binding.result,
                    root_entity: "Document".into(),
                    fields: vec![],
                    relations: vec![],
                },
            ),
            root_paths: vec![],
            orbit: None,
            component_selection: None,
            managed_index: None,
            cardinality: ApplicationQueryCardinality::Many,
            predicates: vec![],
            where_filter: None,
            related_predicates: vec![],
            ordering: vec![],
            continuation: None,
            live_cause: None,
            live_whole_result: false,
            disclosure: ApplicationQueryDisclosureContract::public(),
            authorization: ApplicationQueryAuthorizationRequirement::Public,
            basis_support: ApplicationQueryBasisSupport::current_and_pinned(),
            lanes: ApplicationQueryLaneEligibility::one_shot(),
        },
    )
}

fn schema(source: ErasedApplicationQueryDefinition) -> WorthQueryPortableApplicationSchemaRecord {
    let mut members = vec![
        ApplicationSchemaMember::Entity {
            entity: "Document".into(),
        },
        ApplicationSchemaMember::ApplicationQuery { definition: source },
        ApplicationSchemaMember::ManagedIndex {
            definition: definition(),
        },
    ];
    members.sort();
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationSchemaParts {
            owner: "worth.tests.index".into(),
            name: "Index".into(),
            major: 1,
            minor: 0,
            members,
            contributions: vec![],
        },
    )
}

#[test]
fn managed_source_requires_public_disclosure_and_a_real_cycle_is_refused() {
    let source = source_query();
    assert!(validate_portable_application_schema_freshly(schema(source.clone())).is_ok());
    let mut parts = source.clone().into_parts();
    worth_query_declaration::worth_query_disclosure_classification!(PrivateDisclosure);
    parts.disclosure = ApplicationQueryDisclosureContract::installed_policy(PrivateDisclosure);
    assert_eq!(
        validate_portable_application_schema_freshly(schema(
            ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, parts)
        ))
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidManagedIndex {
            index: "Prefix".into(),
            cause: ApplicationManagedIndexDefinitionDenial::UnsupportedSourcePosture,
        }
    );
    let mut parts = source.into_parts();
    parts.parameters = vec![ApplicationQueryParameterDefinition::from_untrusted_fields(
        ARCHIVE_DECODE,
        "probe".into(),
        ScalarAspectType::String,
        ProbeBinding::IDENTITY,
        None,
        None,
    )];
    parts.managed_index = Some(ApplicationQueryManagedIndexMeaning::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableQueryManagedIndexParts {
            semantic_version: 1,
            index: "Prefix".into(),
            probe_parameter: "probe".into(),
        },
    ));
    assert_eq!(
        validate_portable_application_schema_freshly(schema(
            ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, parts)
        ))
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidManagedIndex {
            index: "Prefix".into(),
            cause: ApplicationManagedIndexDefinitionDenial::SourceDependencyCycle,
        }
    );
}
