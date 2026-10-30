pub use bank_external_rail::{
    dispatch, inquire_admission_count, inquire_notice, inquire_status, LedgerStatus,
    RailCorrelation, RailDispatch, RailEffectPayload, RailExchangeOutcome,
    RailProtocolSupportProfile, RailRejection,
};
pub use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolUnsupportedVersionPosture, BoundaryProtocolVersion,
};

pub use crate::facade::support::{
    spawn_rail, spawn_rail_with_protocol_support, EXPECTED_EFFECT, EXPECTED_PROTOCOL_IDENTITY,
    FRAME_TIMEOUT,
};

pub const V1_CORPUS: &str =
    include_str!("../../../../protocol-corpus/estate-death-notification/v1.hex");
pub const V2_CORPUS: &str =
    include_str!("../../../../protocol-corpus/estate-death-notification/v2.hex");

pub async fn send_invalid_raw_frame(address: std::net::SocketAddr, request: &serde_json::Value) {
    let payload = serde_json::to_vec(request).unwrap();
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    stream
        .write_all(&u32::try_from(payload.len()).unwrap().to_le_bytes())
        .await
        .unwrap();
    stream.write_all(&payload).await.unwrap();
    stream.shutdown().await.unwrap();
    let mut response = [0u8; 1];
    assert_eq!(stream.read(&mut response).await.unwrap(), 0);
}

pub async fn assert_unsupported_v1(
    address: std::net::SocketAddr,
    scenario: &str,
    expected: BoundaryProtocolUnsupportedVersionPosture,
) {
    let correlation = correlation(scenario);
    let outcome = dispatch_payload(
        address,
        correlation.clone(),
        EXPECTED_PROTOCOL_IDENTITY,
        1,
        24,
        V1_CORPUS,
    )
    .await;
    let RailExchangeOutcome::Rejected(RailRejection::UnsupportedProtocolVersion(unsupported)) =
        outcome
    else {
        panic!("external owner must preserve typed unsupported posture: {outcome:?}");
    };
    assert_eq!(unsupported.produced(), BoundaryProtocolVersion::new(1));
    assert_eq!(unsupported.posture(), expected);
    assert_unadmitted(address, correlation, 0).await;
}

pub async fn dispatch_payload(
    address: std::net::SocketAddr,
    correlation: RailCorrelation,
    identity: BoundaryProtocolIdentity,
    version: u32,
    maximum: u64,
    corpus: &str,
) -> RailExchangeOutcome {
    dispatch_bytes(
        address,
        correlation,
        identity,
        version,
        maximum,
        decode_hex(corpus),
    )
    .await
}

pub async fn dispatch_bytes(
    address: std::net::SocketAddr,
    correlation: RailCorrelation,
    identity: BoundaryProtocolIdentity,
    version: u32,
    maximum: u64,
    bytes: Vec<u8>,
) -> RailExchangeOutcome {
    dispatch(
        address,
        RailDispatch {
            correlation,
            payload: RailEffectPayload::new(
                EXPECTED_EFFECT,
                identity,
                BoundaryProtocolVersion::new(version),
                maximum,
                bytes,
            ),
        },
        FRAME_TIMEOUT,
    )
    .await
}

pub async fn assert_unadmitted(
    address: std::net::SocketAddr,
    correlation: RailCorrelation,
    expected_admissions: u64,
) {
    assert_eq!(
        inquire_status(address, correlation.clone(), FRAME_TIMEOUT)
            .await
            .unwrap(),
        LedgerStatus::NoRecord
    );
    assert_eq!(
        inquire_notice(address, correlation, FRAME_TIMEOUT)
            .await
            .unwrap(),
        None
    );
    assert_eq!(admission_count(address).await, expected_admissions);
}

pub fn correlation(scenario: &str) -> RailCorrelation {
    RailCorrelation::new("phase-8-c3-protocol", scenario.as_bytes().to_vec())
}

pub async fn admission_count(address: std::net::SocketAddr) -> u64 {
    inquire_admission_count(address, FRAME_TIMEOUT)
        .await
        .unwrap()
}

pub fn decode_hex(corpus: &str) -> Vec<u8> {
    let corpus = corpus.trim();
    assert!(corpus.len().is_multiple_of(2), "hex corpus has a remainder");
    let bytes = corpus
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(bytes.len() * 2, corpus.len());
    bytes
}
