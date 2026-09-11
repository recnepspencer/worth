use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationRetainedEffectBinding,
};
use worth_query_installation::facade::InstalledExternalEffectContract;

use super::model::{WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission};

const EFFECT: &str = "ExternalEffect";
const EXTERNAL_PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
    BoundaryProtocolIdentity::new("test.external-payload"),
    BoundaryProtocolVersion::new(1),
);

#[derive(Clone)]
struct ExternalPayload(Vec<u8>);
worth_query_declaration::worth_query_portable_type!(
    ExternalPayload => "worth.query.test.external-payload.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(ExternalPayloadBinding for ExternalPayload { identity: "worth.query.test.external-payload.v1" });
impl ApplicationRetainedEffectBinding for ExternalPayloadBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<Self::Value>() + value.0.capacity()).unwrap_or(u64::MAX)
    }
}

impl ApplicationExternalEffectBinding for ExternalPayloadBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = EXTERNAL_PROTOCOL;
    const MAX_EXTERNAL_BYTES: u64 = 8;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.clone()
    }
}

#[test]
fn exact_installed_external_emission_yields_its_projected_bytes() {
    let batch = admitted(vec![external(EFFECT, b"notice")]);

    assert_eq!(
        batch.external_payload(&contract(EFFECT)),
        Ok(Some(b"notice".to_vec()))
    );
}

#[test]
fn ordinary_emission_cannot_satisfy_an_external_contract() {
    let batch = admitted(vec![WorthQueryApplicationEmission::new::<
        ExternalPayloadBinding,
    >(EFFECT, ExternalPayload(b"notice".to_vec()))]);

    assert_eq!(
        batch.external_payload(&contract(EFFECT)),
        Err("declared external effect used an ordinary non-external emission")
    );
}

#[test]
fn missing_wrong_and_duplicate_external_emissions_are_rejected() {
    assert_eq!(
        admitted(Vec::new()).external_payload(&contract(EFFECT)),
        Err("declared external effect did not emit its installed typed payload")
    );
    assert_eq!(
        admitted(vec![external("WrongEffect", b"notice")]).external_payload(&contract(EFFECT)),
        Err("declared external effect did not emit its installed typed payload")
    );
    assert_eq!(
        admitted(vec![external(EFFECT, b"one"), external(EFFECT, b"two")])
            .external_payload(&contract(EFFECT)),
        Err("declared external effect emitted its installed payload more than once")
    );
}

#[test]
fn installed_bound_drift_is_rejected() {
    let mut contract = contract(EFFECT);
    let InstalledExternalEffectContract::Declared {
        maximum_payload_bytes,
        ..
    } = &mut contract
    else {
        unreachable!()
    };
    *maximum_payload_bytes = 7;

    assert_eq!(
        admitted(vec![external(EFFECT, b"notice")]).external_payload(&contract),
        Err("external payload projection drifted from its installed bound")
    );
}

fn contract(effect: &str) -> InstalledExternalEffectContract {
    InstalledExternalEffectContract::Declared {
        correlation_family:
            worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                "external-family",
            )
            .unwrap(),
        effect: effect.to_owned(),
        rust_payload_type: <ExternalPayload as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY,
        protocol: EXTERNAL_PROTOCOL,
        maximum_payload_bytes: ExternalPayloadBinding::MAX_EXTERNAL_BYTES,
    }
}

fn external(effect: &'static str, bytes: &[u8]) -> WorthQueryApplicationEmission {
    WorthQueryApplicationEmission::new_external::<ExternalPayloadBinding>(
        effect,
        ExternalPayload(bytes.to_vec()),
    )
    .expect("fixture projection stays within its declared bound")
}

fn admitted(
    emissions: Vec<WorthQueryApplicationEmission>,
) -> WorthQueryAdmittedApplicationEmissionBatch {
    WorthQueryAdmittedApplicationEmissionBatch::admit(emissions, 1_024)
        .expect("fixture batch stays within its retained-byte ceiling")
}
