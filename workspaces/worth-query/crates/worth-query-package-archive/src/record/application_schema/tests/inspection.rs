//! These are portable descriptions; literals intentionally confer no authority.
use super::{decode_member, encode_member, try_decode_member};
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_query::*, application_schema::ApplicationSchemaMember,
    portable_identity::WorthQueryPortableTypeIdentity as Identity,
};

fn binding() -> ApplicationQueryBindingReference {
    let id = |name| Identity::from_untrusted(format!("worth.archive.inspection.{name}.v1"));
    ApplicationQueryBindingReference::from_untrusted_parts(ApplicationQueryBindingReferenceParts {
        binding: id("binding"),
        input: id("input"),
        query_name: "Read".into(),
        query: id("query"),
        parameters: id("parameters"),
        result: id("result"),
        scope: id("scope"),
        scope_entity: "Node".into(),
        scope_aspect: "Identity".into(),
        scope_field: "Key".into(),
        scope_value: id("key"),
        scope_resolution: ApplicationQueryScopeResolutionMode::InputField,
        scope_writable: false,
        principal_binding: "PrincipalBinding".into(),
    })
}

fn governed_parts() -> WorthQueryPortableApplicationQueryInspectionParts {
    let binding = binding();
    let rules = ApplicationQueryInspectionMetadataFacet::ALL
        .into_iter()
        .map(|facet| {
            ApplicationQueryDisclosureRule::from_untrusted_fields(
                ApplicationQueryDisclosureSelector::InspectionMetadata(
                    ApplicationQueryInspectionMetadataMeaning::from_untrusted_fields(
                        binding.clone(),
                        facet,
                    ),
                ),
                AspectValue::Bool(true),
                ApplicationQueryInfluenceContract::permit([facet.influence()]),
            )
        })
        .collect();
    WorthQueryPortableApplicationQueryInspectionParts {
        semantic_version: 1,
        binding,
        metadata_disclosure: ApplicationQueryDisclosureContract::from_untrusted_parts(
            WorthQueryPortableApplicationQueryDisclosureParts {
                posture: ApplicationQueryDisclosurePosture::Governed,
                classification: "inspection".into(),
                capability_name: Some("Inspect".into()),
                capability_type: Some(Identity::declared("worth.inspect.v1")),
                rules,
            },
        ),
    }
}

fn definition(
    parts: WorthQueryPortableApplicationQueryInspectionParts,
) -> ErasedApplicationQueryInspectionDefinition {
    ErasedApplicationQueryInspectionDefinition::from_untrusted_parts(parts)
}

fn change_contract(
    parts: &mut WorthQueryPortableApplicationQueryInspectionParts,
    change: impl FnOnce(&mut WorthQueryPortableApplicationQueryDisclosureParts),
) {
    let source = &parts.metadata_disclosure;
    let mut contract = WorthQueryPortableApplicationQueryDisclosureParts {
        posture: source.posture(),
        classification: source.classification().into(),
        capability_name: source.capability_name().map(str::to_owned),
        capability_type: source.capability_identity(),
        rules: source.rules().to_vec(),
    };
    change(&mut contract);
    parts.metadata_disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(contract);
}

#[test]
fn inspection_members_and_full_binding_selectors_roundtrip_under_additive_tags() {
    for parts in [
        governed_parts(),
        WorthQueryPortableApplicationQueryInspectionParts {
            semantic_version: 1,
            binding: binding(),
            metadata_disclosure: ApplicationQueryDisclosureContract::public(),
        },
    ] {
        let source = definition(parts);
        source.validate().unwrap();
        let member = ApplicationSchemaMember::ApplicationQueryInspection { definition: source };
        let bytes = encode_member(&member);
        assert_eq!(&bytes[..4], &[0, 45, 0, 1]);
        assert_eq!(decode_member(&bytes), member);
        assert_eq!(encode_member(&decode_member(&bytes)), bytes);
    }
}

#[test]
fn inspection_metadata_rejects_foreign_full_bindings_partial_mixed_and_influence_contracts() {
    let source = governed_parts();
    let original = definition(source.clone());
    for change in 0..14 {
        let mut parts = source.clone();
        let mut reference = parts.binding.parts().clone();
        let other = Identity::declared("worth.archive.other.v1");
        match change {
            0 => reference.binding = other,
            1 => reference.input = other,
            2 => reference.query_name = "Other".into(),
            3 => reference.query = other,
            4 => reference.parameters = other,
            5 => reference.result = other,
            6 => reference.scope = other,
            7 => reference.scope_entity = "Other".into(),
            8 => reference.scope_aspect = "Other".into(),
            9 => reference.scope_field = "Other".into(),
            10 => reference.scope_value = other,
            11 => {
                reference.scope_resolution = ApplicationQueryScopeResolutionMode::PrincipalIdentity
            }
            12 => reference.scope_writable = true,
            13 => reference.principal_binding = "Other".into(),
            _ => unreachable!(),
        }
        parts.binding = ApplicationQueryBindingReference::from_untrusted_parts(reference);
        let facet = ApplicationQueryInspectionMetadataFacet::EvidenceLinks;
        let old = ApplicationQueryInspectionMetadataMeaning::from_untrusted_fields(
            source.binding.clone(),
            facet,
        );
        let new = ApplicationQueryInspectionMetadataMeaning::from_untrusted_fields(
            parts.binding.clone(),
            facet,
        );
        assert_ne!(
            old.slot_key(),
            new.slot_key(),
            "slot equality must include field {change}"
        );
        let changed = definition(parts);
        assert_eq!(
            changed.validate(),
            Err(ApplicationQueryInspectionDefinitionDenial::InvalidMetadataDisclosure)
        );
        assert_ne!(original.canonical_basis(), changed.canonical_basis());
    }
    for change in 0..7 {
        let mut parts = source.clone();
        change_contract(&mut parts, |contract| match change {
            0 => {
                contract.rules.pop();
            }
            1 => {
                contract.rules.push(contract.rules[0].clone());
                contract.rules.sort();
            }
            2 => {
                contract.rules[0] = ApplicationQueryDisclosureRule::from_untrusted_fields(
                    ApplicationQueryDisclosureSelector::HistoryMetadata(
                        ApplicationHistoryMetadataMeaning::from_untrusted_fields(
                            Identity::declared("worth.history.v1"),
                            ApplicationHistoryMetadataFacet::Occurrence,
                        ),
                    ),
                    AspectValue::Bool(true),
                    ApplicationQueryInfluenceContract::permit_all(),
                );
            }
            3 => {
                let rule = contract.rules[2].clone();
                contract.rules[2] = ApplicationQueryDisclosureRule::from_untrusted_fields(
                    rule.selector().clone(),
                    rule.disclosure_value().clone(),
                    ApplicationQueryInfluenceContract::permit([
                        ApplicationQueryObservableInfluence::Explanation,
                    ]),
                );
            }
            4 => contract.posture = ApplicationQueryDisclosurePosture::Public,
            5 => contract.posture = ApplicationQueryDisclosurePosture::InstalledPolicyRequired,
            6 => contract.rules.reverse(),
            _ => unreachable!(),
        });
        assert_eq!(
            definition(parts).validate(),
            Err(ApplicationQueryInspectionDefinitionDenial::InvalidMetadataDisclosure),
            "contract mutation {change}"
        );
    }
}

#[test]
fn inspection_subgrammars_refuse_unknown_versions_before_reading_fields() {
    assert_eq!(
        try_decode_member(&[0, 45, 0, 2]).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
    let mut invalid = governed_parts();
    invalid.semantic_version = 2;
    assert_eq!(
        definition(invalid).validate(),
        Err(ApplicationQueryInspectionDefinitionDenial::UnsupportedSemanticVersion)
    );
}

#[test]
fn inspection_metadata_uses_existing_decode_depth_and_sequence_budgets() {
    use crate::facade::WorthQueryPackageArchiveLimits as Limits;
    use crate::record::decode_budget::RecordDecodeAttempt;
    let bytes = encode_member(&ApplicationSchemaMember::ApplicationQueryInspection {
        definition: definition(governed_parts()),
    });
    for (limits, expected) in [
        (
            Limits::DEFAULT.with_maximum_nesting_depth(1),
            Kind::NestingDepthBudgetExceeded,
        ),
        (
            Limits::DEFAULT.with_maximum_nested_entries(1),
            Kind::NestedEntryBudgetExceeded,
        ),
    ] {
        let mut input = crate::binary_input::BinaryInput::new(&bytes);
        let mut budget =
            RecordDecodeAttempt::begin(Default::default(), bytes.len() as u64, limits).unwrap();
        assert_eq!(
            super::super::member::decode(&mut input, &mut budget)
                .unwrap_err()
                .kind(),
            expected
        );
    }
}
