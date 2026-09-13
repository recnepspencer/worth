use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::wal_frame::{
    decode_wal_frame_v1_header, WalSegmentIdentity, WAL_FRAME_V1_FOOTER_BYTES,
    WAL_FRAME_V1_HEADER_BYTES,
};

use crate::{
    PhysicalArtifactScope, PhysicalByteRange, PhysicalIntegrityObservationCounters,
    UntrustedPhysicalArtifact,
};

use super::{validate_wal_frame, WalFrameIntegrityValidation};

/// Admits the first complete WAL frame in one bounded segment suffix.
///
/// The length probe remains inside the integrity owner. It reveals no WAL
/// fields to recovery before the exact frame has passed full validation.
pub fn validate_wal_frame_prefix<'media>(
    suffix: UntrustedPhysicalArtifact<'media>,
    store: StableStoreIdentity,
    identity: WalSegmentIdentity,
    artifact_offset: u64,
) -> (
    WalFrameIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let bytes = suffix.bytes();
    let inspected_length = inspected_frame_length(bytes);
    let input_length = inspected_length.min(bytes.len());
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes[..input_length]);
    let range =
        PhysicalByteRange::new(artifact_offset, inspected_length as u64).unwrap_or_else(|_| {
            PhysicalByteRange::new(artifact_offset, input_length as u64).expect(
                "a hostile declared length still has one bounded header at its artifact offset",
            )
        });
    validate_wal_frame(
        input,
        PhysicalArtifactScope::wal_frame(store, identity, range),
    )
}

fn inspected_frame_length(bytes: &[u8]) -> usize {
    if bytes.len() < WAL_FRAME_V1_HEADER_BYTES {
        return WAL_FRAME_V1_HEADER_BYTES;
    }
    let header: &[u8; WAL_FRAME_V1_HEADER_BYTES] = bytes[..WAL_FRAME_V1_HEADER_BYTES]
        .try_into()
        .expect("the guarded WAL prefix contains one complete header");
    let Ok(header) = decode_wal_frame_v1_header(header) else {
        return WAL_FRAME_V1_HEADER_BYTES;
    };
    usize::try_from(header.payload_bytes())
        .ok()
        .and_then(|payload| WAL_FRAME_V1_HEADER_BYTES.checked_add(payload))
        .and_then(|framed| framed.checked_add(WAL_FRAME_V1_FOOTER_BYTES))
        .unwrap_or(bytes.len())
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    use super::*;
    use crate::{PhysicalDamageCause, PhysicalIntegrityRejection};

    #[test]
    fn short_header_localizes_only_the_known_missing_header_range() {
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([9; 16]).unwrap(),
        )
        .published_identity();
        let identity = WalSegmentIdentity::new(3, 4).unwrap();
        let bytes = [0_u8; 17];
        let (validation, _) = validate_wal_frame_prefix(
            UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
            store,
            identity,
            23,
        );
        let WalFrameIntegrityValidation::Rejected(PhysicalIntegrityRejection::Damaged(damage)) =
            validation
        else {
            panic!("short WAL header must be a typed truncation")
        };
        assert_eq!(damage.cause(), PhysicalDamageCause::Truncated);
        assert_eq!(
            damage.scope().byte_range(),
            PhysicalByteRange::new(23, 116).unwrap()
        );
        assert_eq!(
            damage.damaged_range(),
            PhysicalByteRange::new(40, 99).unwrap()
        );
    }

    #[test]
    fn later_frame_with_overflowing_absolute_length_is_a_typed_rejection() {
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
        )
        .published_identity();
        let identity = WalSegmentIdentity::new(3, 4).unwrap();
        let mut header = [0_u8; WAL_FRAME_V1_HEADER_BYTES];
        header[..8].copy_from_slice(b"WORTHWAL");
        header[8..10].copy_from_slice(&1_u16.to_le_bytes());
        header[10..12].copy_from_slice(&(WAL_FRAME_V1_HEADER_BYTES as u16).to_le_bytes());
        header[12..20].copy_from_slice(&3_u64.to_le_bytes());
        header[20..28].copy_from_slice(&4_u64.to_le_bytes());
        header[28..36].copy_from_slice(&5_u64.to_le_bytes());
        header[36..44].copy_from_slice(&6_u64.to_le_bytes());
        header[44..52].copy_from_slice(&(u64::MAX - 200).to_le_bytes());

        let (validation, _) = validate_wal_frame_prefix(
            UntrustedPhysicalArtifact::from_bounded_bytes(&header),
            store,
            identity,
            256,
        );
        let WalFrameIntegrityValidation::Rejected(PhysicalIntegrityRejection::Damaged(damage)) =
            validation
        else {
            panic!("overflowing absolute frame scope must be rejected")
        };
        assert_eq!(damage.cause(), PhysicalDamageCause::FramingLengthMismatch);
    }
}
