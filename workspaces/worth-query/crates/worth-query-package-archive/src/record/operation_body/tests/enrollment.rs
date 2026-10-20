//! v17 cannot silently downgrade the created-subject enrollment attachment.
use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputPosture, ApplicationMutationOutputRoleCardinality,
    ApplicationMutationOutputRoleDescription,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyEnrollment, ApplicationOperationBodyOutput,
    ApplicationOperationBodyOutputTarget,
};

fn body(enrolled: bool) -> ApplicationOperationBodyProgram {
    ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Container".into(),
        vec![],
        Some(
            ApplicationOperationBodyCreation::from_untrusted(
                "Subject".into(),
                "subject".into(),
                vec![],
            )
            .with_untrusted_enrollment(enrolled.then(|| {
                ApplicationOperationBodyEnrollment::from_untrusted(
                    "workflow".into(),
                    "binding".into(),
                    "subject".into(),
                )
            })),
        ),
        vec![],
    )
    .with_untrusted_outputs(vec![ApplicationOperationBodyOutput {
        binding: "binding".into(),
        role: ApplicationMutationOutputRoleDescription {
            name: "subject".into(),
            entity: "Subject".into(),
            posture: ApplicationMutationOutputPosture::Create,
            cardinality: ApplicationMutationOutputRoleCardinality::ExactlyOne,
        },
        target: ApplicationOperationBodyOutputTarget::Creation {
            key: "subject".into(),
        },
    }])
}
#[test]
fn birth_body_v17_roundtrips_and_v16_leaves_the_new_meaning_unconsumed() {
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body(true)).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &17_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body(true));
    assert!(input.is_finished());
    bytes[..2].copy_from_slice(&16_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert!(decode(&mut input, &mut budget)
        .unwrap()
        .creation()
        .unwrap()
        .enrollment()
        .is_none());
    assert!(!input.is_finished());
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body(false)).unwrap();
    assert_eq!(
        &output.into_bytes()[..2],
        &10_u16.to_be_bytes(),
        "an un-enrolled output retains its old format"
    );
}
#[test]
fn enrollment_codec_refuses_invalid_flag_and_truncated_target() {
    for bytes in [vec![2], vec![1], vec![1, 0, 0]] {
        assert!(super::super::enrollment::decode(&mut BinaryInput::new(&bytes)).is_err());
    }
}
