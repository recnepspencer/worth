//! Absence is a serialized Option, never a missing record member or null scalar.
use super::super::{canonical_identity_with_capture, encoder};
use super::*;
use serde::{ser::SerializeStruct, Serialize, Serializer};

fn capture(value: &impl Serialize) -> ApplicationCanonicalInput {
    canonical_identity_with_capture(
        "optional-input",
        "record",
        value,
        encoder::HashingSink::new().capturing(true),
    )
    .unwrap()
    .capture
    .unwrap()
}

#[test]
fn optional_members_preserve_absence_empty_and_text_without_poisoning_other_selectors() {
    #[derive(Serialize)]
    struct Input<'a> {
        identity: &'a str,
        note: Option<&'a str>,
    }
    for note in [None, Some(""), Some("details")] {
        let input = capture(&Input {
            identity: "item",
            note,
        });
        assert_eq!(
            input
                .scalar_member("identity", ScalarAspectType::String)
                .unwrap()
                .into_value(),
            AspectValue::String(InternedString::Raw("item".to_owned()))
        );
        let selected = input
            .optional_scalar_member("note", ScalarAspectType::String)
            .unwrap()
            .map(ApplicationCanonicalInputScalar::into_value);
        assert_eq!(
            selected,
            note.map(|value| AspectValue::String(InternedString::Raw(value.to_owned())))
        );
        assert!(matches!(
            input.optional_scalar_member("missing", ScalarAspectType::String),
            Err(ApplicationCanonicalInputDenial::MissingMember)
        ));
        assert!(matches!(
            input.scalar_member("note", ScalarAspectType::String),
            Err(ApplicationCanonicalInputDenial::InvalidScalar)
        ));
        assert!(matches!(
            input.optional_scalar_member("identity", ScalarAspectType::String),
            Err(ApplicationCanonicalInputDenial::InvalidScalar)
        ));
    }
}

#[test]
fn nested_options_and_wrong_scalar_families_are_refused() {
    #[derive(Serialize)]
    struct Nested {
        note: Option<Option<u64>>,
    }
    for note in [Some(None), Some(Some(7))] {
        assert!(matches!(
            capture(&Nested { note }).optional_scalar_member("note", ScalarAspectType::UInt64),
            Err(ApplicationCanonicalInputDenial::UnsupportedShape)
        ));
    }
    #[derive(Serialize)]
    struct Wrong {
        note: Option<u64>,
    }
    assert!(matches!(
        capture(&Wrong { note: Some(7) }).optional_scalar_member("note", ScalarAspectType::String),
        Err(ApplicationCanonicalInputDenial::InvalidScalar)
    ));
}

struct Duplicate;
impl Serialize for Duplicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut record = serializer.serialize_struct("Duplicate", 2)?;
        record.serialize_field("note", &None::<String>)?;
        record.serialize_field("note", &Some("later"))?;
        record.end()
    }
}
#[test]
fn duplicate_after_none_is_not_mistaken_for_an_unselected_member() {
    assert!(matches!(
        capture(&Duplicate).optional_scalar_member("note", ScalarAspectType::String),
        Err(ApplicationCanonicalInputDenial::DuplicateMember)
    ));
}
