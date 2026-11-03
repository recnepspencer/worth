//! Pure declaration carriage: no runtime, binding recipe or live head is made.
use super::*;
mod navigation;
mod retention;
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryModelDomain as Domain, ApplicationHistoryOperation,
        ErasedApplicationHistoryDefinition as Definition,
        WorthQueryPortableApplicationHistoryParts as Parts,
    },
    application_query::{ApplicationQueryBindingReference, ApplicationQueryDisclosureContract},
};

fn definition(domain: Domain, governed: bool) -> Definition {
    let history = super::history::history();
    Definition::from_untrusted_parts(Parts {
        identity: "DocumentHistory".into(),
        history_type: history.parts().history_type.clone(),
        access: history.parts().subject_read.clone(),
        metadata_disclosure: if governed {
            history.parts().metadata_disclosure.clone()
        } else {
            ApplicationQueryDisclosureContract::public()
        },
        model_domain: domain,
        ownership: Vec::new(),
        navigation: None,
        window: None,
        bookmarks: None,
        operations: vec![
            ApplicationHistoryOperation {
                binding: "ZEdit".into(),
                label: "Move Z".into(),
            },
            ApplicationHistoryOperation {
                binding: "AEdit".into(),
                label: "Move A".into(),
            },
        ],
    })
}

#[test]
fn document_history_roundtrips_full_access_metadata_domain_and_authored_order() {
    for domain in [Domain::WholeComposite, Domain::ScopeEntityRecord] {
        for governed in [false, true] {
            let original = definition(domain, governed);
            original.validate().unwrap();
            let member = ApplicationSchemaMember::ApplicationHistory {
                definition: original.clone(),
            };
            let bytes = encode_member(&member);
            assert_eq!(
                &bytes[..6],
                &[
                    0,
                    51,
                    0,
                    1,
                    0,
                    if domain == Domain::WholeComposite {
                        1
                    } else {
                        2
                    }
                ]
            );
            let decoded = decode_member(&bytes);
            assert_eq!(decoded, member);
            let ApplicationSchemaMember::ApplicationHistory {
                definition: decoded,
            } = &decoded
            else {
                panic!("the document binding cannot become an observation-history query")
            };
            assert_eq!(decoded.parts(), original.parts());
            assert_eq!(decoded.parts().operations[0].binding, "ZEdit");
            assert_eq!(
                encode_member(&member),
                encode_member(&ApplicationSchemaMember::ApplicationHistory {
                    definition: decoded.clone()
                })
            );
        }
    }
}

#[test]
fn malformed_history_definitions_refuse_before_the_member_tag_is_emitted() {
    let valid = definition(Domain::ScopeEntityRecord, true);
    assert!(
        !encode_member(&ApplicationSchemaMember::ApplicationHistory {
            definition: valid.clone()
        })
        .is_empty()
    );
    for change in 0..6 {
        let mut parts = valid.parts().clone();
        match change {
            0 => parts.identity = "bad identity".into(),
            1 => parts.operations.clear(),
            2 => parts.operations.push(parts.operations[0].clone()),
            3 => parts.operations[0].label = " ".into(),
            4 => parts.history_type = worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::declared("foreign.history.v1"),
            5 => {
                let mut access = parts.access.parts().clone();
                access.query_name.clear();
                parts.access = ApplicationQueryBindingReference::from_untrusted_parts(access);
            }
            _ => unreachable!(),
        }
        let member = ApplicationSchemaMember::ApplicationHistory {
            definition: Definition::from_untrusted_parts(parts),
        };
        let mut output = BinaryOutput::with_capacity(0);
        assert_eq!(
            super::super::member::write(&mut output, &member)
                .unwrap_err()
                .kind(),
            Kind::InvalidRecordShape
        );
        assert!(
            output.into_bytes().is_empty(),
            "invalid definition {change} wrote bytes"
        );
    }
}

#[test]
fn unknown_history_headers_refuse_before_reading_body_fields() {
    for bytes in [&[0, 51, 0, 99][..], &[0, 51, 0, 1, 0, 99][..]] {
        assert_eq!(
            try_decode_member(bytes).unwrap_err().kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}

#[test]
fn declared_ownership_has_distinct_wire_meaning_and_preserves_exact_endpoints() {
    use worth_query_declaration::facade::application_history::ApplicationHistoryOwnershipRelation;
    let original = definition(Domain::WholeComposite, false);
    let unowned = encode_member(&ApplicationSchemaMember::ApplicationHistory {
        definition: original.clone(),
    });
    let mut parts = original.parts().clone();
    parts.ownership = vec![
        ApplicationHistoryOwnershipRelation {
            relation: "OwnsMember".into(),
            from: "Model".into(),
            to: "Member".into(),
        },
        ApplicationHistoryOwnershipRelation {
            relation: "OwnsChild".into(),
            from: "Member".into(),
            to: "Member".into(),
        },
    ];
    let owned = ApplicationSchemaMember::ApplicationHistory {
        definition: Definition::from_untrusted_parts(parts),
    };
    let encoded = encode_member(&owned);
    assert_eq!(&unowned[..4], &[0, 51, 0, 1]);
    assert_eq!(&encoded[..4], &[0, 51, 0, 4]);
    assert_eq!(decode_member(&encoded), owned);
    assert_eq!(encode_member(&decode_member(&unowned)), unowned);
}

#[test]
fn decoded_duplicate_operations_cannot_bypass_declaration_validation() {
    let member = ApplicationSchemaMember::ApplicationHistory {
        definition: definition(Domain::WholeComposite, false),
    };
    let mut bytes = encode_member(&member);
    let operation = bytes
        .windows(5)
        .position(|window| window == b"ZEdit")
        .unwrap();
    bytes[operation] = b'A';
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );
}

#[test]
fn history_truncation_and_operation_count_pressure_refuse_without_partial_meaning() {
    let original = definition(Domain::WholeComposite, false);
    let member = ApplicationSchemaMember::ApplicationHistory {
        definition: original.clone(),
    };
    let mut bytes = encode_member(&member);
    for cut in 0..bytes.len() {
        assert_eq!(
            try_decode_member(&bytes[..cut]).unwrap_err().kind(),
            Kind::Truncated,
            "prefix {cut}"
        );
    }
    let operation_bytes: usize = original
        .parts()
        .operations
        .iter()
        .map(|operation| 8 + operation.binding.len() + operation.label.len())
        .sum();
    let count_offset = bytes.len() - operation_bytes - 4;
    bytes[count_offset..count_offset + 4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        Kind::NestedEntryBudgetExceeded
    );
}

#[test]
fn history_depth_refusal_matches_encode_preflight_and_bounded_decode() {
    let member = ApplicationSchemaMember::ApplicationHistory {
        definition: definition(Domain::WholeComposite, false),
    };
    assert_eq!(
        super::super::member::require_encoding_contract(&member, 1)
            .unwrap_err()
            .kind(),
        Kind::NestingDepthBudgetExceeded
    );
    let bytes = encode_member(&member);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_nesting_depth(1);
    let mut budget =
        RecordDecodeAttempt::begin(Default::default(), bytes.len() as u64, limits).unwrap();
    let mut input = crate::binary_input::BinaryInput::new(&bytes);
    assert_eq!(
        super::super::member::decode(&mut input, &mut budget)
            .unwrap_err()
            .kind(),
        Kind::NestingDepthBudgetExceeded
    );
}
