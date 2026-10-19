//! Program/evolution meaning retains every reference correspondence; empty profiles stay byte-stable.
use super::*;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationOperationBodyReferenceInput as Reference,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
#[test]
fn reference_correspondence_changes_program_meaning_without_rewriting_old_profiles() {
    let base = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        None,
        vec![],
    );
    assert_eq!(
        operation_body_record(&base),
        related_output::operation_body_record(&base)
    );
    let reference = Reference {
        relation: "Selected".into(),
        from: "Row".into(),
        to: "Principal".into(),
        field: ApplicationFieldBindingLocus::new("Principal", "Facts", "Identity"),
        binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".into()),
        scalar_family: worth_foundational::facade::ScalarAspectType::UInt64,
        member: "selected".into(),
        key: "selected".into(),
    };
    let original = operation_body_record(
        &base
            .clone()
            .with_untrusted_reference_inputs(vec![reference.clone()]),
    );
    for variant in 0..10 {
        let mut changed = reference.clone();
        match variant {
            0 => changed.relation = "Other".into(),
            1 => changed.from = "Other".into(),
            2 => changed.to = "Other".into(),
            3 => changed.field = ApplicationFieldBindingLocus::new("Other", "Facts", "Identity"),
            4 => {
                changed.field = ApplicationFieldBindingLocus::new("Principal", "Other", "Identity")
            }
            5 => changed.field = ApplicationFieldBindingLocus::new("Principal", "Facts", "Other"),
            6 => changed.binding = WorthQueryPortableTypeIdentity::from_untrusted("other".into()),
            7 => changed.scalar_family = worth_foundational::facade::ScalarAspectType::String,
            8 => changed.member = "other".into(),
            9 => changed.key = "other".into(),
            _ => unreachable!(),
        }
        assert_ne!(
            operation_body_record(&base.clone().with_untrusted_reference_inputs(vec![changed])),
            original,
            "member {variant}"
        );
    }
}
