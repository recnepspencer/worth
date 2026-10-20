use super::*;
use crate::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationStructuredValueBinding,
    ApplicationValueValidationDenial,
};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

struct ValidatedPayload;
impl ApplicationStructuredValueBinding for ValidatedPayload {
    type Value = String;
    const IDENTITY_NAME: &'static str = "worth.query.test.validated-external-payload.v1";
    fn validate(value: &String) -> Result<(), ApplicationValueValidationDenial> {
        if value == "allowed" {
            Ok(())
        } else {
            Err(ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "not allowed",
            ))
        }
    }
}
impl ApplicationExternalEffectBinding for ValidatedPayload {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("worth.query.test.validated-external"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 32;
    fn external_effect_bytes(value: &String) -> Vec<u8> {
        assert_eq!(value, "allowed", "validation must precede encoding");
        value.as_bytes().to_vec()
    }
}

#[test]
fn external_projection_preserves_payload_validation_without_a_resource_quote() {
    assert!(
        ApplicationExternalEffectPayloadProjection::encode::<ValidatedPayload>(
            "Notice",
            "denied".into(),
        )
        .is_err()
    );
    let valid = ApplicationExternalEffectPayloadProjection::encode::<ValidatedPayload>(
        "Notice",
        "allowed".into(),
    )
    .unwrap();
    assert_eq!(valid.bytes().as_ref(), b"allowed");
}
