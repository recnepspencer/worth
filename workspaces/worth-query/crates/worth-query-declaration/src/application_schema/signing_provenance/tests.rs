//! Codec boundaries only. Descriptive parts never mint a native signing permit.
use super::{
    ApplicationSigningEntityCoordinates as Coordinates, ApplicationSigningProvenance as Value,
    ApplicationSigningProvenanceBinding as Binding,
    ApplicationSigningProvenanceMetadata as Metadata, ApplicationSigningProvenanceParts as Parts,
};
use crate::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
};

fn parts() -> Parts {
    let coordinate = Coordinates {
        partition: 0,
        slot: 1,
        generation: 0,
    };
    Parts {
        metadata: Metadata {
            program_revision: "program".into(),
            operation: "sign".into(),
            binding: "binding".into(),
            signer_issuer: "i".repeat(2048),
            signer_subject: "s".repeat(1024),
            purpose: "verification".into(),
            verification_configuration: "v".repeat(512),
            clock_source: "c".repeat(8192),
            clock_timeline: "t".repeat(8192),
        },
        operation_meaning: [1; 32],
        logical_request: [2; 32],
        input: [3; 32],
        subject: coordinate,
        signer: coordinate,
        signing_intent: [4; 32],
        subject_coverage: [5; 32],
        event_reference: [6; 32],
        issued_at_nanoseconds: 1,
        observed_at_nanoseconds: 2,
    }
}
#[test]
fn original_authentication_limits_and_long_native_clock_names_roundtrip() {
    let value = Value::from_parts(parts()).unwrap();
    let encoded = Binding::encode(&value).unwrap();
    assert_eq!(Binding::decode(&encoded).unwrap(), value);
    assert!(value.parts().metadata.maximum_encoded_bytes().unwrap() > 16 * 1024);
}
#[test]
fn codec_reuses_authentication_validation_and_refuses_reversed_named_time() {
    let mut bad = parts();
    bad.metadata.signer_issuer.push('i');
    assert!(Value::from_parts(bad).is_err());
    let mut bad = parts();
    bad.metadata.verification_configuration.push('v');
    assert!(Value::from_parts(bad).is_err());
    let mut bad = parts();
    bad.metadata.purpose = " padded".into();
    assert!(Value::from_parts(bad).is_err());
    let mut bad = parts();
    bad.observed_at_nanoseconds = 0;
    assert!(Value::from_parts(bad).is_err());
}
