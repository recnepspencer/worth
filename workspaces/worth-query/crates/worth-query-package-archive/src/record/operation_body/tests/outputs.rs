//! V10 extends descriptions; old body bytes are still selected when unused.
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
fn fixed_output_targets_roundtrip_and_require_their_version() {
    for target in [
        Target::Scope,
        Target::Creation {
            key: "record".to_owned(),
        },
    ] {
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Item".to_owned(),
            vec![],
            None,
            vec![],
        )
        .with_untrusted_outputs(vec![ApplicationOperationBodyOutput {
            binding: "worth.example.binding.v1".to_owned(),
            role: ApplicationMutationOutputRoleDescription {
                name: "output".to_owned(),
                entity: "Item".to_owned(),
                posture: if target == Target::Scope {
                    Posture::Preserve
                } else {
                    Posture::Create
                },
                cardinality: Cardinality::ExactlyOne,
            },
            target,
        }]);
        let mut output = BinaryOutput::with_capacity(256);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &10_u16.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        let mut input = BinaryInput::new(&bytes);
        assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
        assert!(input.is_finished());
        bytes[..2].copy_from_slice(&9_u16.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        let mut input = BinaryInput::new(&bytes);
        let decoded = decode(&mut input, &mut budget).unwrap();
        assert!(decoded.outputs().is_empty());
        assert!(!input.is_finished(), "v9 cannot consume v10 meaning");
    }
}
