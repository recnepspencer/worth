use super::*;
use crate::application_schema::{
    ApplicationCanonicalValueContract, ApplicationFieldPresence, ApplicationSchemaMember,
};
use std::collections::BTreeMap;
use worth_foundational::facade::{
    encode_canonical_value, AspectContractRevision, AspectIdentity, CanonicalBasisLocus,
    CanonicalBasisValue as V, CanonicalIntegerWidth, CanonicalValue, InternedString,
    ScalarAspectType, ValueDecodeBounds, WorthValue,
};

const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(1024, 64, 16, 8);

#[derive(WorthValue)]
struct Original {
    choices: Vec<Choice>,
}
#[derive(WorthValue)]
struct RustRenamed {
    choices: Vec<Choice>,
}
#[derive(WorthValue)]
enum Choice {
    First,
    Second(String),
}
#[derive(WorthValue)]
struct AddedVariant {
    choices: Vec<ExpandedChoice>,
}
#[derive(WorthValue)]
enum ExpandedChoice {
    First,
    Second(String),
    Third(u64),
}
#[derive(WorthValue)]
struct RenamedField {
    #[serde(rename = "selection")]
    choices: Vec<Choice>,
}

fn field<Value: CanonicalValue>(bounds: ValueDecodeBounds) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Field {
        entity: "Document".into(),
        aspect: "Facts".into(),
        field: "Saved".into(),
        presence: ApplicationFieldPresence::Required,
        scalar_family: ScalarAspectType::String,
        value_type: "example.saved.v1".into(),
        canonical: Some(ApplicationCanonicalValueContract::of::<Value>(bounds)),
        unit: None,
        frame: None,
        writable: true,
        equality_queryable: false,
        unique: false,
    }
}
fn aspect(field: ApplicationSchemaMember, revision: u64) -> ApplicationAspectValueMeaning {
    ApplicationAspectValueMeaning::from_members(&[
        ApplicationSchemaMember::Aspect {
            absence: worth_foundational::facade::AbsenceLaw::Required,
            entity: "Document".into(),
            aspect: "Facts".into(),
            identity: AspectIdentity(71),
            revision: AspectContractRevision(revision),
        },
        field,
    ])
    .unwrap()
    .remove(0)
}

#[test]
fn value_meaning_golden_is_the_complete_shared_field_emitter() {
    let meaning = ApplicationFieldValueMeaning::from_member(&field::<Vec<String>>(BOUNDS))
        .unwrap()
        .unwrap();
    let payload = meaning.canonical_basis().payload();
    assert_eq!(payload.version().as_str(), "worth-query-field-meaning-v1");
    let actual = payload
        .entries()
        .iter()
        .map(|entry| {
            let CanonicalBasisLocus::Named(InternedString::Raw(name)) = entry.locus() else {
                panic!("golden loci are named text")
            };
            (name.clone(), entry.value().clone())
        })
        .collect::<BTreeMap<_, _>>();
    let text = |value: &str| V::ExactText(value.to_owned().into());
    let count = |value| V::UnsignedInteger {
        width: CanonicalIntegerWidth::Bits64,
        value,
    };
    // Typed literal golden, not re-encoding a second copy of the source shape.
    let expected = [
        ("kind", text("field")),
        ("entity", text("Document")),
        ("aspect", text("Facts")),
        ("field", text("Saved")),
        ("presence", text("required")),
        ("scalar-family", text("string")),
        ("value-type", text("example.saved.v1")),
        ("codec.protocol", text("worth.canonical-json.v1")),
        ("codec.maximum-wire-bytes", count(1024)),
        ("codec.maximum-string-bytes", count(64)),
        ("codec.maximum-sequence-length", count(16)),
        ("codec.maximum-depth", count(8)),
        ("codec.shape.kind", text("sequence")),
        ("codec.shape.child.kind", text("string")),
        ("unit", V::Null),
        ("frame", V::Null),
        ("writable", V::Bool(true)),
        ("equality-queryable", V::Bool(false)),
        ("unique", V::Bool(false)),
    ]
    .into_iter()
    .map(|(key, value)| (format!("member[0].{key}"), value))
    .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected);
    assert_eq!(
        encode_canonical_value(
            &vec!["nondefault".to_owned(), "escaped\n".to_owned()],
            BOUNDS
        )
        .unwrap(),
        "[\"nondefault\",\"escaped\\n\"]"
    );
    assert_eq!(meaning.codec_digest(), Some(meaning.semantic_digest()));
}

#[test]
fn value_meaning_nested_changes_require_an_explicit_revision_advance() {
    let previous = aspect(field::<Original>(BOUNDS), 3);
    assert_eq!(
        previous,
        aspect(field::<RustRenamed>(BOUNDS), 3),
        "Rust names do not change portable meaning"
    );
    let mut binding = field::<Original>(BOUNDS);
    let ApplicationSchemaMember::Field { value_type, .. } = &mut binding else {
        unreachable!()
    };
    *value_type = "example.saved.v2".into();
    let mut presence = field::<Original>(BOUNDS);
    let ApplicationSchemaMember::Field {
        presence: value, ..
    } = &mut presence
    else {
        unreachable!()
    };
    *value = ApplicationFieldPresence::Optional;
    let mut unit = field::<Original>(BOUNDS);
    let ApplicationSchemaMember::Field { unit: value, .. } = &mut unit else {
        unreachable!()
    };
    *value = Some("metre".into());
    let mut frame = field::<Original>(BOUNDS);
    let ApplicationSchemaMember::Field { frame: value, .. } = &mut frame else {
        unreachable!()
    };
    *value = Some("model".into());
    for changed in [
        field::<AddedVariant>(BOUNDS),
        field::<RenamedField>(BOUNDS),
        field::<Original>(ValueDecodeBounds::new(2048, 64, 16, 8)),
        binding,
        presence,
        unit,
        frame,
    ] {
        assert_eq!(
            aspect(changed.clone(), 3).compare_predecessor(&previous),
            Err(ApplicationAspectMeaningPredecessorDenial::MeaningChangedWithoutRevision)
        );
        assert_eq!(
            aspect(changed, 4).compare_predecessor(&previous),
            Ok(ApplicationAspectMeaningChange::MeaningChanged)
        );
    }
    assert_eq!(
        aspect(field::<Original>(BOUNDS), 2).compare_predecessor(&previous),
        Err(ApplicationAspectMeaningPredecessorDenial::RevisionRegressed)
    );
    let revision_only = aspect(field::<Original>(BOUNDS), 4);
    assert_eq!(
        revision_only.compare_predecessor(&previous),
        Ok(ApplicationAspectMeaningChange::RevisionOnly)
    );
    assert_eq!(
        revision_only.fields()[0].codec_digest(),
        previous.fields()[0].codec_digest(),
        "revision is separate from field codec meaning"
    );
}

#[test]
fn value_meaning_refuses_ambiguous_rosters() {
    let original = field::<Original>(BOUNDS);
    assert_eq!(
        ApplicationAspectValueMeaning::from_members(&[original.clone()]),
        Err(ApplicationValueMeaningDenial::FieldWithoutAspect)
    );
    let mut members = vec![
        ApplicationSchemaMember::Aspect {
            absence: worth_foundational::facade::AbsenceLaw::Required,
            entity: "Document".into(),
            aspect: "Facts".into(),
            identity: AspectIdentity(71),
            revision: AspectContractRevision(1),
        },
        original.clone(),
        original,
    ];
    assert_eq!(
        ApplicationAspectValueMeaning::from_members(&members),
        Err(ApplicationValueMeaningDenial::DuplicateField)
    );
    members.pop();
    members.push(members[0].clone());
    assert!(ApplicationAspectValueMeaning::from_members(&members).is_err());
}

#[test]
fn whole_aspect_absence_is_revisioned_independently_of_fields() {
    let required = aspect(field::<Original>(BOUNDS), 3);
    let members = |revision| {
        [
            ApplicationSchemaMember::Aspect {
                entity: "Document".into(),
                aspect: "Facts".into(),
                identity: AspectIdentity(71),
                revision: AspectContractRevision(revision),
                absence: worth_foundational::facade::AbsenceLaw::Optional,
            },
            field::<Original>(BOUNDS),
        ]
    };
    let optional = ApplicationAspectValueMeaning::from_members(&members(3))
        .unwrap()
        .remove(0);
    assert_eq!(
        optional.compare_predecessor(&required),
        Err(ApplicationAspectMeaningPredecessorDenial::MeaningChangedWithoutRevision)
    );
    assert_eq!(
        ApplicationAspectValueMeaning::from_members(&members(4)).unwrap()[0]
            .compare_predecessor(&required),
        Ok(ApplicationAspectMeaningChange::MeaningChanged)
    );
}
