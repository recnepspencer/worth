//! Current resource wire meaning includes complete codec profiles, never recipes.
use super::*;
use worth_foundational::facade::{AspectValue, ValueDecodeBounds};
use worth_query_declaration::facade::{
    application_query::*, application_resource::*, application_schema::*,
    portable_identity::WorthQueryPortableTypeIdentity,
};

fn identity(name: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(format!("worth.archive.resource.{name}.v1"))
}

fn definition() -> ErasedApplicationResourceDefinition {
    let resource = identity("assessment");
    let mut result =
        ApplicationResourceValueBindingReference::of::<StringApplicationValueBinding>()
            .parts()
            .clone();
    result.canonical = Some(
        ApplicationCanonicalValueContract::of::<String>(ValueDecodeBounds::new(2048, 1024, 8, 4))
            .portable_reference(),
    );
    result.supports_admitted_decode = true;
    let disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Governed,
            classification: "resource".to_owned(),
            capability_name: Some("InspectResource".to_owned()),
            capability_type: Some(identity("capability")),
            rules: ApplicationResourceDisclosureFacet::ALL
                .into_iter()
                .map(|facet| {
                    ApplicationQueryDisclosureRule::from_untrusted_fields(
                        ApplicationQueryDisclosureSelector::Resource(
                            ApplicationResourceDisclosureMeaning::from_untrusted_fields(
                                resource.clone(),
                                facet,
                            ),
                        ),
                        AspectValue::Bool(true),
                        ApplicationQueryInfluenceContract::permit(
                            facet.required_influences().iter().copied(),
                        ),
                    )
                })
                .collect(),
        },
    );
    ErasedApplicationResourceDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationResourceParts {
            name: "Assessment".to_owned(),
            resource_type: resource,
            provider_role: identity("provider"),
            access: ApplicationQueryBindingReference::from_untrusted_parts(
                ApplicationQueryBindingReferenceParts {
                    binding: identity("binding"),
                    input: identity("input"),
                    query_name: "SubjectRead".to_owned(),
                    query: identity("query"),
                    parameters: identity("parameters"),
                    result: identity("result"),
                    scope: identity("scope"),
                    scope_entity: "Node".to_owned(),
                    scope_aspect: "Identity".to_owned(),
                    scope_field: "Key".to_owned(),
                    scope_value: identity("key"),
                    scope_resolution: ApplicationQueryScopeResolutionMode::InputField,
                    scope_writable: false,
                    principal_binding: "ResourcePrincipal".to_owned(),
                },
            ),
            input: ApplicationResourceValueBindingReference::of::<U64ApplicationValueBinding>(),
            result: ApplicationResourceValueBindingReference::from_untrusted_parts(result),
            failure: ApplicationResourceValueBindingReference::of::<StringApplicationValueBinding>(
            ),
            profile: ApplicationResourceRequestResponseProfile {
                retry_max_attempts: 3,
                retry_delay_milliseconds: 2,
                timeout_milliseconds: 5,
            },
            limits: ApplicationResourceLimits {
                maximum_live: 2,
                maximum_retained_results: 2,
                ingress_slots_per_request: 2,
                maximum_input_bytes: 32,
                maximum_frame_bytes: 2048,
            },
            authorization: ApplicationResourceAuthorization::from_untrusted_requirements([
                ApplicationQueryAuthorizationRequirement::Ability {
                    ability: "StartAssessment".to_owned(),
                    scope_entity: "Node".to_owned(),
                },
                ApplicationQueryAuthorizationRequirement::Ability {
                    ability: "InspectAssessment".to_owned(),
                    scope_entity: "Node".to_owned(),
                },
                ApplicationQueryAuthorizationRequirement::Ability {
                    ability: "AdvanceAssessment".to_owned(),
                    scope_entity: "Node".to_owned(),
                },
                ApplicationQueryAuthorizationRequirement::Ability {
                    ability: "CloseAssessment".to_owned(),
                    scope_entity: "Node".to_owned(),
                },
            ]),
            disclosure,
        },
    )
}

#[test]
fn resource_member_preserves_canonical_and_native_profiles_and_all_disclosure_slots() {
    let definition = definition();
    definition.validate().unwrap();
    let member = ApplicationSchemaMember::ApplicationResource { definition };
    let bytes = encode_member(&member);
    assert_eq!(&bytes[..2], &39u16.to_be_bytes());
    assert_eq!(decode_member(&bytes), member);
    assert_eq!(encode_member(&decode_member(&bytes)), bytes);
    for prefix in 0..bytes.len() {
        assert!(
            try_decode_member(&bytes[..prefix]).is_err(),
            "truncated prefix {prefix}"
        );
    }
}

#[test]
fn resource_direct_payload_disclosure_does_not_authorize_cleanup_counts_or_foreign_slots() {
    let source = definition();
    for foreign in [false, true] {
        let mut parts = source.parts().clone();
        let contract = &parts.disclosure;
        let mut rules = contract.rules().to_vec();
        let position = rules
            .iter()
            .position(|rule| {
                matches!(rule.selector(),
            ApplicationQueryDisclosureSelector::Resource(meaning)
            if meaning.facet() == ApplicationResourceDisclosureFacet::CleanupDetail)
            })
            .unwrap();
        let rule = &rules[position];
        let selector = if foreign {
            ApplicationQueryDisclosureSelector::Resource(
                ApplicationResourceDisclosureMeaning::from_untrusted_fields(
                    identity("other-resource"),
                    ApplicationResourceDisclosureFacet::CleanupDetail,
                ),
            )
        } else {
            rule.selector().clone()
        };
        let influence = if foreign {
            rule.influence().clone()
        } else {
            ApplicationQueryInfluenceContract::permit([
                ApplicationQueryObservableInfluence::Explanation,
            ])
        };
        rules[position] = ApplicationQueryDisclosureRule::from_untrusted_fields(
            selector,
            rule.disclosure_value().clone(),
            influence,
        );
        parts.disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(
            WorthQueryPortableApplicationQueryDisclosureParts {
                posture: contract.posture(),
                classification: contract.classification().to_owned(),
                capability_name: contract.capability_name().map(str::to_owned),
                capability_type: contract.capability_identity(),
                rules,
            },
        );
        let invalid = ErasedApplicationResourceDefinition::from_untrusted_parts(parts);
        assert_eq!(
            invalid.validate(),
            Err(ApplicationResourceDefinitionDenial::InvalidDisclosure)
        );
        let mut output = BinaryOutput::with_capacity(0);
        assert_eq!(
            super::super::member::write(
                &mut output,
                &ApplicationSchemaMember::ApplicationResource {
                    definition: invalid
                }
            )
            .unwrap_err()
            .kind(),
            WorthQueryPackageArchiveDenialKind::InvalidRecordShape
        );
        assert!(output.into_bytes().is_empty());
    }
}
