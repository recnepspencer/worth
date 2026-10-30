//! Versioned provider meaning never serializes an executable provider or root.
use super::{decode_member, encode_member, try_decode_member};
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity as Identity;
use worth_query_declaration::facade::{application_query::*, application_schema::*};

#[derive(worth_foundational::facade::WorthValue)]
pub struct Probe {
    text: String,
}
worth_query_declaration::worth_query_value_binding!(pub ProbeBinding for Probe {
    identity: "worth.query.tests.archive.index-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});

fn definition() -> ErasedApplicationManagedIndexDefinition {
    ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
        WorthQueryPortableManagedIndexParts {
            semantic_version: 1,
            name: "Prefix".into(),
            provider: "worth.tests.prefix.v1".into(),
            provider_version: 1,
            source: ApplicationManagedIndexSourceMeaning::same_scope_without_parameters(
                ApplicationQueryBindingReference::from_untrusted_parts(
                    ApplicationQueryBindingReferenceParts {
                        binding: Identity::declared("worth.tests.source-binding.v1"),
                        input: Identity::declared("worth.tests.source-input.v1"),
                        query_name: "Source".into(),
                        query: Identity::declared("worth.tests.source-query.v1"),
                        parameters: Identity::declared("worth.rust.unit"),
                        result: Identity::declared("worth.tests.source-row.v1"),
                        scope: Identity::declared("worth.tests.scope.v1"),
                        scope_entity: "Document".into(),
                        scope_aspect: "Identity".into(),
                        scope_field: "Id".into(),
                        scope_value: Identity::declared("worth.rust.u64"),
                        scope_resolution: ApplicationQueryScopeResolutionMode::PrincipalIdentity,
                        scope_writable: false,
                        principal_binding: "Principal".into(),
                    },
                ),
            ),
            entity: "Document".into(),
            probe: ApplicationManagedIndexProbeMeaning::of::<ProbeBinding>(),
        },
    )
}

#[test]
fn managed_index_v1_round_trips_full_probe_and_rejects_future_version_before_payload() {
    let member = ApplicationSchemaMember::ManagedIndex {
        definition: definition(),
    };
    let bytes = encode_member(&member);
    assert_eq!(&bytes[..4], &[0, 48, 0, 1]);
    assert_eq!(decode_member(&bytes), member);
    // No payload follows: version refusal must precede any source/profile read.
    assert_eq!(
        try_decode_member(&[0, 48, 0, 2]).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
    let mut parts = definition().parts().clone();
    parts.probe = ApplicationManagedIndexProbeMeaning::of::<StringApplicationValueBinding>();
    assert_eq!(
        ErasedApplicationManagedIndexDefinition::from_untrusted_parts(parts).validate(),
        Err(ApplicationManagedIndexDefinitionDenial::UnsupportedProbeProfile)
    );
    // The old physical entity grammar remains an independent literal byte oracle.
    let entity = ApplicationSchemaMember::Entity {
        entity: "Document".into(),
    };
    assert_eq!(
        encode_member(&entity),
        [0, 1, 0, 0, 0, 8, b'D', b'o', b'c', b'u', b'm', b'e', b'n', b't']
    );
}

fn source_query() -> ErasedApplicationQueryDefinition {
    let binding = definition().parts().source.binding.parts().clone();
    ErasedApplicationQueryDefinition::from_untrusted_parts(
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
            ordering: vec![],
            continuation: None,
            live_cause: None,
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
    parts.disclosure = ApplicationQueryDisclosureContract::installed_policy("private");
    assert_eq!(
        validate_portable_application_schema_freshly(schema(
            ErasedApplicationQueryDefinition::from_untrusted_parts(parts)
        ))
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidManagedIndex {
            index: "Prefix".into(),
            cause: ApplicationManagedIndexDefinitionDenial::UnsupportedSourcePosture,
        }
    );
    let mut parts = source.into_parts();
    parts.parameters = vec![ApplicationQueryParameterDefinition::from_untrusted_fields(
        "probe".into(),
        ScalarAspectType::String,
        ProbeBinding::IDENTITY,
        None,
        None,
    )];
    parts.managed_index = Some(ApplicationQueryManagedIndexMeaning::from_untrusted_parts(
        WorthQueryPortableQueryManagedIndexParts {
            semantic_version: 1,
            index: "Prefix".into(),
            probe_parameter: "probe".into(),
        },
    ));
    assert_eq!(
        validate_portable_application_schema_freshly(schema(
            ErasedApplicationQueryDefinition::from_untrusted_parts(parts)
        ))
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidManagedIndex {
            index: "Prefix".into(),
            cause: ApplicationManagedIndexDefinitionDenial::SourceDependencyCycle,
        }
    );
}
