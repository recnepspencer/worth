//! Pure declaration evidence, not signing or mutation authority.
use super::*;
use crate::application_schema::{
    ApplicationOperationBodyCreation as Creation, ApplicationOperationBodySigningSpec as Signing,
};
use worth_foundational::facade::ScalarAspectType;
#[test]
fn related_capture_is_not_a_signed_subject_footprint_and_presence_changes_meaning() {
    let source = ApplicationOperationBodyRelatedFieldSource {
        relation: "Selected".into(),
        from: "Root".into(),
        to: "Child".into(),
        field: ApplicationFieldBindingLocus::new("Child", "Facts", "Text"),
        presence: ApplicationFieldPresence::Required,
    };
    let mut optional = source.clone();
    optional.presence = ApplicationFieldPresence::Optional;
    assert_ne!(source.canonical_record(), optional.canonical_record());
    let mut different_selection = source.clone();
    different_selection.relation = "History".into();
    assert_ne!(
        source.canonical_record(),
        different_selection.canonical_record()
    );
    let creation = Creation::from_untrusted(
        "Child".into(),
        "child".into(),
        vec![Step::RelatedField {
            field: source.field.clone(),
            binding: crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.rust.string".into(),
            ),
            scalar_family: ScalarAspectType::String,
            source,
        }],
    );
    assert_eq!(
        super::super::classified_capture::validate_creation(&creation),
        Ok(())
    );
    let signed = creation.with_untrusted_signing(Some(Signing::from_untrusted(
        "approval".into(),
        ApplicationFieldBindingLocus::new("Child", "Facts", "Actor"),
        ApplicationFieldBindingLocus::new("Child", "Facts", "Provenance"),
    )));
    assert_eq!(
        super::super::classified_capture::validate_creation(&signed),
        Err(Denial::UnclassifiedSignedCapture)
    );
}
