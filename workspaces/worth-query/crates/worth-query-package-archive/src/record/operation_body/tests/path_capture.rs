//! Codec-only V29 evidence: decoded path descriptions must still pass native readmission.
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryRootPathMeaning as Path, ApplicationQueryRootPathStep as Step,
        RelationDirection as Direction, WorthQueryPortableApplicationQueryRootPathParts as Parts,
    },
    application_schema::{
        ApplicationFieldPresence as Presence, ApplicationOperationBodyCaptureRole as Role,
        ApplicationOperationBodyPathFieldSource as Source,
    },
};
fn capture(identity_path: bool) -> ApplicationOperationBodyCreatedField {
    let entity = if identity_path { "Row" } else { "Lot" };
    ApplicationOperationBodyCreatedField::CapturedPathField {
        field: ApplicationFieldBindingLocus::new("Evidence", "Facts", "Text"),
        binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into()),
        scalar_family: ScalarAspectType::String,
        source: Source {
            path: Path::from_untrusted_parts(Parts {
                start_entity: "Row".into(),
                terminal_entity: entity.into(),
                guards: Vec::new(),
                steps: if identity_path {
                    Vec::new()
                } else {
                    vec![Step::from_untrusted_fields(
                        "LotRows".into(),
                        "Lot".into(),
                        "Row".into(),
                        Direction::Reverse,
                     worth_query_declaration::facade::application_query::ApplicationQueryRootPathTraversal::OneHop,)]
                },
            }),
            field: ApplicationFieldBindingLocus::new(entity, "Facts", "Comment"),
            presence: Presence::Optional,
        },
        role: Role::Subject,
        target_presence: Presence::Optional,
    }
}
fn budget(bytes: &[u8]) -> RecordDecodeAttempt {
    RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap()
}
#[test]
fn certified_path_v29_roundtrips_identity_and_reverse_path_with_exact_decode_measure() {
    for identity in [true, false] {
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Row".into(),
            Vec::new(),
            Some(
                ApplicationOperationBodyCreation::from_untrusted(
                    "Evidence".into(),
                    "signed".into(),
                    vec![capture(identity)],
                )
                .with_untrusted_subject_impact_bound(std::num::NonZeroUsize::new(4)),
            ),
            Vec::new(),
        );
        let mut output = BinaryOutput::with_capacity(4096);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &29_u16.to_be_bytes());
        let mut input = BinaryInput::new(&bytes);
        let mut decoded_work = budget(&bytes);
        assert_eq!(decode(&mut input, &mut decoded_work).unwrap(), body);
        assert!(input.is_finished());
        let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
        write(&mut measure, &body).unwrap();
        assert_eq!(
            measure.nested_entries(),
            decoded_work.finish().nested_entries()
        );
        bytes[..2].copy_from_slice(&28_u16.to_be_bytes());
        assert_eq!(
            decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes))
                .unwrap_err()
                .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
#[test]
fn path_capture_refuses_unknown_presence_and_role_tags() {
    let field = capture(false);
    let mut output = BinaryOutput::with_capacity(2048);
    super::super::path_capture::write(&mut output, &field).unwrap();
    let original = output.into_bytes();
    for from_end in [2, 4, 6] {
        let mut bytes = original.clone();
        let position = bytes.len() - from_end;
        bytes[position..position + 2].copy_from_slice(&99_u16.to_be_bytes());
        assert_eq!(
            super::super::path_capture::decode(
                &mut BinaryInput::new(&bytes),
                &mut budget(&bytes),
                field.field().clone(),
                field.binding().clone()
            )
            .unwrap_err()
            .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
#[test]
fn authored_subject_bound_alone_selects_v29_and_cannot_be_silently_dropped() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        Vec::new(),
        Some(
            ApplicationOperationBodyCreation::from_untrusted(
                "Evidence".into(),
                "signed".into(),
                Vec::new(),
            )
            .with_untrusted_subject_impact_bound(std::num::NonZeroUsize::new(1)),
        ),
        Vec::new(),
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &29_u16.to_be_bytes());
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes)).unwrap(),
        body
    );
    bytes[..2].copy_from_slice(&28_u16.to_be_bytes());
    let mut input = BinaryInput::new(&bytes);
    let _ = decode(&mut input, &mut budget(&bytes)).unwrap();
    assert!(
        !input.is_finished(),
        "the enclosing record framing must refuse the unconsumed new footer"
    );
}
