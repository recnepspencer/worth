#[allow(unused_imports)]
use crate::c9_scope_identity_contracts::*;

#[test]
fn physical_work_decode_keeps_store_bytes_outside_the_filename_identity() {
    let identity = worth_store_physical_format::PhysicalWorkObligationIdentity::new(
        core::num::NonZeroU64::new(1).unwrap(),
        core::num::NonZeroU64::new(2).unwrap(),
        core::num::NonZeroU64::new(3).unwrap(),
    );
    let obligation = worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6::from_identity(
        store(),
        identity,
        worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::DurabilityBarrier,
        worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization,
        None,
    )
    .unwrap();
    let encoded = encode_physical_work_obligation_v6(obligation);
    let decoded = decode_physical_work_obligation_v6(&encoded).unwrap();
    assert_eq!(decoded.identity(), identity);
    assert_eq!(decoded.store_identity(), store().bytes());
}

#[test]
fn wal_codec_exposes_container_identity_without_promoting_staged_lsn_fields() {
    let identity = worth_store_physical_format::WalSegmentIdentity::new(4, 5).unwrap();
    let request =
        worth_store_physical_format::wal_frame::WalFrameV1EncodeRequest::from_segment_identity(
            identity,
            10,
            11,
            b"wal-identity",
            b"payload",
        )
        .unwrap();
    let encoded = encode_wal_frame_v1(request);
    let header: &[u8; WAL_FRAME_V1_HEADER_BYTES] =
        encoded[..WAL_FRAME_V1_HEADER_BYTES].try_into().unwrap();
    let decoded = decode_wal_frame_v1_header(header).unwrap();
    assert_eq!(decoded.identity(), identity);
    assert_eq!(decoded.lsn_start(), 10);
    assert_eq!(decoded.lsn_end(), 11);
}
