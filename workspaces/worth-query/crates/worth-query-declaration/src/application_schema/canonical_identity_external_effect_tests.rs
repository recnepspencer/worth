use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

use super::{identity, ApplicationSchemaMember};
use crate::application_schema::{
    ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily,
};

#[test]
fn every_external_effect_contract_dimension_changes_identity() {
    let base = ApplicationSchemaMember::OperationExternalEffect {
        operation: "Operation".to_string(),
        effect: "ExternalEffect".to_string(),
        rust_payload_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "Payload",
        ),
        protocol: external_protocol(1),
        maximum_payload_bytes: 64,
        correlation_family: WorthQueryExternalEffectCorrelationFamily::new("external-family")
            .unwrap(),
    };
    let base_identity = identity(std::slice::from_ref(&base));

    macro_rules! changed {
        ($field:ident, $value:expr) => {{
            let mut member = base.clone();
            let ApplicationSchemaMember::OperationExternalEffect { $field, .. } = &mut member
            else {
                unreachable!("external-effect fixture changed member family")
            };
            *$field = $value;
            member
        }};
    }

    for member in [
        changed!(operation, "OtherOperation".to_string()),
        changed!(effect, "OtherEffect".to_string()),
        changed!(
            rust_payload_type,
            crate::portable_identity::WorthQueryPortableTypeIdentity::declared("OtherPayload")
        ),
        changed!(protocol, external_protocol(2)),
        changed!(maximum_payload_bytes, 65),
        changed!(
            correlation_family,
            WorthQueryExternalEffectCorrelationFamily::new("other-family").unwrap()
        ),
    ] {
        assert_ne!(identity(&[member]), base_identity);
    }
}

fn external_protocol(version: u32) -> ApplicationExternalEffectProtocol {
    ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("test.external-payload"),
        BoundaryProtocolVersion::new(version),
    )
}
