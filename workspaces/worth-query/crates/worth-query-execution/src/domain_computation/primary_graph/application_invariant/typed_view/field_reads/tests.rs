//! Literal absence laws and hostile present content exercise the actual owner decision.
use super::*;
use worth_foundational::facade::StructAspectValue;
#[test]
fn optional_parent_absence_preserves_required_and_optional_child_meanings() {
    let key = FieldKey::new("value").unwrap();
    // Independent truth-table oracle: parent absence is legal only for Optional.
    for child in [
        ApplicationFieldPresence::Required,
        ApplicationFieldPresence::Optional,
    ] {
        assert_eq!(
            field_material(None, AbsenceLaw::Optional, child, Some(&key)).unwrap(),
            None
        );
        for parent in [AbsenceLaw::Required, AbsenceLaw::Defaulted] {
            assert_eq!(
                field_material(None, parent, child, Some(&key))
                    .unwrap_err()
                    .kind(),
                WorthQueryInvariantAccessDenialKind::MissingRequiredField
            );
        }
    }
}
#[test]
fn corrupt_present_required_child_is_refused_while_optional_child_may_be_absent() {
    let key = FieldKey::new("value").unwrap();
    let empty = StructAspectValue::new([]).unwrap();
    // Inject corrupted material at the private read-owner boundary. Native's
    // contract validator cannot lawfully issue this present Required-child image.
    let corrupt = Some(ContractValidatedAspectValueView::Struct(&empty));
    for parent in [AbsenceLaw::Optional, AbsenceLaw::Required] {
        assert_eq!(
            field_material(
                corrupt,
                parent,
                ApplicationFieldPresence::Required,
                Some(&key)
            )
            .unwrap_err()
            .kind(),
            WorthQueryInvariantAccessDenialKind::MissingRequiredField
        );
        assert_eq!(
            field_material(
                corrupt,
                parent,
                ApplicationFieldPresence::Optional,
                Some(&key)
            )
            .unwrap(),
            None
        );
    }
    let complete = StructAspectValue::new([(key.clone(), AspectValue::UInt64(17))]).unwrap();
    assert_eq!(
        field_material(
            Some(ContractValidatedAspectValueView::Struct(&complete)),
            AbsenceLaw::Optional,
            ApplicationFieldPresence::Required,
            Some(&key)
        )
        .unwrap(),
        Some(&AspectValue::UInt64(17))
    );
    assert_eq!(
        field_material(
            Some(ContractValidatedAspectValueView::Scalar(
                &AspectValue::UInt64(17)
            )),
            AbsenceLaw::Optional,
            ApplicationFieldPresence::Required,
            Some(&key)
        )
        .unwrap_err()
        .kind(),
        WorthQueryInvariantAccessDenialKind::InvalidValue
    );
}
