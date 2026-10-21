//! Related correspondence has explicit transport meaning; decoding grants no native target.
use super::*;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
        ApplicationMutationOutputRoleDescription,
    },
    application_schema::{
        ApplicationOperationBodyOutput, ApplicationOperationBodyOutputTarget as Target,
    },
};

#[test]
fn related_output_v22_retains_endpoints_accounts_and_rejects_downgrade() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Root".into(),
        vec![],
        None,
        vec![],
    )
    .with_untrusted_outputs(vec![ApplicationOperationBodyOutput {
        binding: "binding".into(),
        role: ApplicationMutationOutputRoleDescription {
            name: "source".into(),
            entity: "Child".into(),
            posture: Posture::Preserve,
            cardinality: Cardinality::ExactlyOne,
        },
        target: Target::Related {
            relation: "Selected".into(),
            from: "Root".into(),
            to: "Child".into(),
        },
    }]);
    let mut output = BinaryOutput::with_capacity(512);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &22_u16.to_be_bytes());
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
    for version in [10_u16, 21] {
        bytes[..2].copy_from_slice(&version.to_be_bytes());
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
