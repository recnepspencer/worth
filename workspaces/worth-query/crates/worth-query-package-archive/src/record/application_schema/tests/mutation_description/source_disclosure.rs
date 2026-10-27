//! Source inspection is an explicit portable association, independent of aftermath.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBindingReference, ApplicationQueryBindingReferenceParts,
    ApplicationQueryScopeResolutionMode,
};

fn member() -> ApplicationSchemaMember {
    let ApplicationSchemaMember::ApplicationMutation { description } = mutation_member(
        ApplicationMutationScopeResolutionMode::InputField,
        ApplicationMutationOutputRoleCardinality::ExactlyOne,
    ) else {
        unreachable!()
    };
    let mut parts = description.into_parts();
    parts.inspection_disclosure = Some(DeclaredApplicationSourceDisclosure::from_untrusted_parts(
        DeclaredApplicationSourceDisclosureParts {
            identity: identity("inspect-original-source.v1"),
            source_binding: parts.binding_identity.clone(),
            query: ApplicationQueryBindingReference::from_untrusted_parts(
                ApplicationQueryBindingReferenceParts {
                    binding: identity("source-inspection-read.v1"),
                    input: identity("inspection-input.v1"),
                    query_name: "source-inspection".to_owned(),
                    query: identity("source-inspection-query.v1"),
                    parameters: identity("inspection-parameters.v1"),
                    result: identity("inspection-result.v1"),
                    scope: identity("Body"),
                    scope_entity: "Body".to_owned(),
                    scope_aspect: "Identity".to_owned(),
                    scope_field: "Key".to_owned(),
                    scope_value: identity("body-key.v1"),
                    scope_resolution: ApplicationQueryScopeResolutionMode::InputField,
                    scope_writable: false,
                    principal_binding: "InspectorPrincipal".to_owned(),
                },
            ),
        },
    ));
    ApplicationSchemaMember::ApplicationMutation {
        description: ApplicationMutationDescription::from_untrusted_parts(parts),
    }
}

#[test]
fn independent_source_disclosure_roundtrips_every_query_binding_field() {
    let member = member();
    let encoded = encode_member(&member);
    let decoded = decode_member(&encoded);
    assert_eq!(decoded, member);
    assert_eq!(encode_member(&decoded), encoded);
    for expected in [
        "inspect-original-source.v1",
        "source-inspection-read.v1",
        "InspectorPrincipal",
    ] {
        assert!(encoded
            .windows(expected.len())
            .any(|window| window == expected.as_bytes()));
    }
    for length in 0..encoded.len() {
        assert_eq!(
            try_decode_member(&encoded[..length]).unwrap_err().kind(),
            WorthQueryPackageArchiveDenialKind::Truncated
        );
    }
}

#[test]
fn current_source_disclosure_requires_a_known_explicit_option_tag() {
    let plain = mutation_member(
        ApplicationMutationScopeResolutionMode::InputField,
        ApplicationMutationOutputRoleCardinality::ExactlyOne,
    );
    let encoded = encode_member(&plain);
    let option = encoded.len() - 2;
    assert_eq!(&encoded[option..], &0u16.to_be_bytes());
    assert_eq!(
        try_decode_member(&encoded[..option]).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::Truncated
    );
    let mut unknown = encode_member(&member());
    unknown[option..option + 2].copy_from_slice(&2u16.to_be_bytes());
    assert_eq!(
        try_decode_member(&unknown).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );
}
