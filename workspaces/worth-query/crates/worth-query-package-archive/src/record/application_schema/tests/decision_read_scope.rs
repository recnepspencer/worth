//! Canonical wire order precedes SDK authoring normalization; descriptions grant nothing.
use super::{decode_member, encode_member, try_decode_member};
use crate::binary_output::BinaryOutput;
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use worth_query_declaration::facade::application_query::ApplicationQueryBindingReference;
use worth_query_declaration::facade::application_schema::{
    ApplicationDecisionReadScopeMeaning as Meaning,
    ApplicationOperationDecisionReadTarget as Target, ApplicationSchemaMember as Member,
};

fn targets() -> Vec<Target> {
    vec![
        Target::Entity {
            entity: "Document".into(),
        },
        Target::Field {
            entity: "Document".into(),
            aspect: "Facts".into(),
            field: "value".into(),
        },
        Target::Relation {
            relation: "Children".into(),
            from: "Document".into(),
            to: "Document".into(),
        },
        Target::ManagedIndex {
            parent: super::managed_index::definition()
                .parts()
                .source
                .binding
                .clone(),
        },
    ]
}
fn wire(targets: &[Target], derived: &[String]) -> Vec<u8> {
    let mut output = BinaryOutput::with_capacity(1024);
    output.u16(55);
    output.u16(1);
    crate::record::sequence::write_sequence(&mut output, targets, |output, target| {
        // Reuse the existing ordinary member's target codec; skip tag + empty operation.
        let bytes = encode_member(&Member::OperationDecisionRead {
            operation: String::new(),
            target: target.clone(),
        });
        output.raw_bytes(&bytes[6..])
    })
    .unwrap();
    crate::record::sequence::write_sequence(&mut output, derived, |output, name| output.text(name))
        .unwrap();
    output.into_bytes()
}
#[test]
fn shared_scope_roundtrip_preserves_all_targets_parent_axes_and_content_identity() {
    let definition =
        Meaning::from_untrusted_parts(targets(), vec!["Zulu".into(), "Alpha".into()]).unwrap();
    let expected = definition.identity().to_owned();
    let mut reversed = definition.targets().to_vec();
    reversed.reverse();
    assert_eq!(
        definition,
        Meaning::from_untrusted_parts(reversed, vec!["Zulu".into(), "Alpha".into()]).unwrap()
    );
    let member = Member::DecisionReadScope {
        definition: definition.clone(),
    };
    let encoded = encode_member(&member);
    assert_eq!(&encoded[..4], &[0, 55, 0, 1]);
    assert_eq!(decode_member(&encoded), member);
    let reference = Member::OperationDecisionReadScope {
        operation: "Read".into(),
        scope: expected.clone(),
    };
    assert_eq!(decode_member(&encode_member(&reference)), reference);
    let mut changed = definition.targets().to_vec();
    changed.push(Target::Entity {
        entity: "Other".into(),
    });
    assert_ne!(
        Meaning::from_untrusted_parts(changed, definition.derived_relations().to_vec())
            .unwrap()
            .identity(),
        expected
    );
    assert_ne!(
        Meaning::from_untrusted_parts(definition.targets().to_vec(), vec!["OtherDerived".into()])
            .unwrap()
            .identity(),
        expected
    );
    let mut changed = definition.targets().to_vec();
    let Target::ManagedIndex { parent } = &mut changed[0] else {
        panic!("semantic order starts with Parent");
    };
    let mut parts = parent.parts().clone();
    parts.principal_binding = "OtherPrincipal".into();
    *parent = ApplicationQueryBindingReference::from_untrusted_parts(parts);
    assert_ne!(
        Meaning::from_untrusted_parts(changed, definition.derived_relations().to_vec())
            .unwrap()
            .identity(),
        expected
    );
}
#[test]
fn shared_scope_vectors_refuse_descending_or_duplicate_wire_before_normalization() {
    let definition =
        Meaning::from_untrusted_parts(targets(), vec!["Alpha".into(), "Zulu".into()]).unwrap();
    assert_eq!(
        decode_member(&wire(definition.targets(), definition.derived_relations())),
        Member::DecisionReadScope {
            definition: definition.clone()
        }
    );
    let mut descending = definition.targets().to_vec();
    descending.reverse();
    let duplicate = vec![
        definition.targets()[0].clone(),
        definition.targets()[0].clone(),
    ];
    for values in [descending, duplicate] {
        assert_eq!(
            try_decode_member(&wire(&values, definition.derived_relations()))
                .unwrap_err()
                .kind(),
            Kind::NonCanonicalRecordSequence
        );
    }
    for names in [
        vec!["Zulu".into(), "Alpha".into()],
        vec!["Alpha".into(), "Alpha".into()],
    ] {
        assert_eq!(
            try_decode_member(&wire(definition.targets(), &names))
                .unwrap_err()
                .kind(),
            Kind::NonCanonicalRecordSequence
        );
    }
}
#[test]
fn shared_scope_unknown_local_versions_refuse_before_variable_content() {
    for tag in [55, 56] {
        assert_eq!(
            try_decode_member(&[0, tag, 0, 2]).unwrap_err().kind(),
            Kind::UnsupportedRecordVersion
        );
    }
}

#[test]
fn scalar_scope_reference_does_not_require_definition_sequence_depth() {
    use crate::record::decode_budget::RecordDecodeAttempt;
    use crate::{binary_input::BinaryInput, limits::WorthQueryPackageArchiveLimits};
    let member = Member::OperationDecisionReadScope {
        operation: "Read".into(),
        scope: "exact-content".into(),
    };
    let bytes = encode_member(&member);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_nesting_depth(1),
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(
        super::super::member::decode(&mut input, &mut budget).unwrap(),
        member
    );
    assert!(input.is_finished());
}
