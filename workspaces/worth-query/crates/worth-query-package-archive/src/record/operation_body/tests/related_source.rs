//! Codec-only proof: related descriptions survive transport; decoding grants no selection authority.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence as Presence, ApplicationOperationBodyRelatedFieldSource as Source,
};

#[test]
fn related_source_v20_preserves_presence_endpoints_and_budget_and_refuses_downgrade() {
    for presence in [Presence::Required, Presence::Optional] {
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Root".into(),
            vec![],
            Some(ApplicationOperationBodyCreation::from_untrusted(
                "Child".into(),
                "child".into(),
                vec![ApplicationOperationBodyCreatedField::RelatedField {
                    field: ApplicationFieldBindingLocus::new("Child", "Facts", "Text"),
                    binding: WorthQueryPortableTypeIdentity::from_untrusted(
                        "worth.rust.string".into(),
                    ),
                    scalar_family: ScalarAspectType::String,
                    source: Source {
                        relation: "Selected".into(),
                        from: "Root".into(),
                        to: "Child".into(),
                        field: ApplicationFieldBindingLocus::new("Child", "Facts", "Text"),
                        presence,
                    },
                }],
            )),
            vec![],
        );
        let mut output = BinaryOutput::with_capacity(512);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &20_u16.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        let mut input = BinaryInput::new(&bytes);
        assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
        assert!(input.is_finished());
        let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
        write(&mut measure, &body).unwrap();
        assert_eq!(measure.nested_entries(), budget.finish().nested_entries());
        bytes[..2].copy_from_slice(&19_u16.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            decode(&mut BinaryInput::new(&bytes), &mut budget)
                .unwrap_err()
                .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
#[test]
fn ordinary_scope_source_keeps_its_existing_v4_encoding() {
    let body = ApplicationOperationBodyProgram::from_untrusted_with_creation(
        vec![ApplicationOperationBodyScopeField::Assign {
            field: ApplicationFieldBindingLocus::new("Root", "Facts", "Ready"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.bool".into()),
            value: AspectValue::Bool(true),
        }],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            "Child".into(),
            "child".into(),
            vec![ApplicationOperationBodyCreatedField::ScopeField {
                field: ApplicationFieldBindingLocus::new("Child", "Facts", "Text"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into()),
                scalar_family: ScalarAspectType::String,
                source: ApplicationFieldBindingLocus::new("Root", "Facts", "Text"),
            }],
        )),
    );
    let mut output = BinaryOutput::with_capacity(512);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &4_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget).unwrap(),
        body
    );
}
