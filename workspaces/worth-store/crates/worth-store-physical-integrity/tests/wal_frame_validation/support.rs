use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::WalSegmentIdentity;
use worth_store_physical_integrity::{
    validate_wal_frame, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalDamageLocalization, PhysicalFormatField,
    PhysicalIntegrityObservationCounters, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact, WalFrameIntegrityValidation,
};

use super::sha256::independent_sha256;

pub const FRAME_OFFSET: u64 = 8_192;
pub const CLEAN_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
pub const SEGMENT_HEX: &str = "574f52544857414c0100740002000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020308678175202523d47e928bae246a14bb4ac1daf638620b47aad28f81d770cc30b";
pub const GENERATION_HEX: &str = "574f52544857414c0100740001000000000000000300000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020303938564097b63d061e658bdbc8be3e920d3b384e4caf9e411112d17159f6988b";
pub const LSN_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000040000000000000003000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c00f77cb1d5b1bd504ace84fc0ee7f04b00a8f35e94b4d998111a32f35cfb28c";
pub const LENGTH_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000400000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c7617fee7f6b6b3eafa436e8acce7518dd9378720b74ec141e6a2c741dd24cdf";
pub const VERSION_HEX: &str = "574f52544857414c0200740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030f3b1a6b324b3ee017668c2fd33f1e5198da8228575b0c8e048a3c5e5c7bf387b";
pub const HEADER_LENGTH_HEX: &str = "574f52544857414c0100750001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020303769bf4c44ac01dbbe8a36815201340bf540b7580db7658a261aca8565fe603c";
pub const PAYLOAD_CHECKSUM_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768f1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020303d6d138251061e3f65f95ae975b5e68904562eb5ef4f2c5ddea8b0e0b31a5317";
pub const FOOTER_CHECKSUM_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa680";
pub const PAYLOAD_BYTE_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9112030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
pub const MAGIC_HEX: &str = "564f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
pub const ZERO_SEGMENT_HEX: &str = "574f52544857414c0100740000000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c12f4b99b324fd9949efc5d22402659d8f1780a21c277cd359e06409e8894088";
pub const ZERO_GENERATION_HEX: &str = "574f52544857414c0100740001000000000000000000000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b910203068430e7328eff20ce22db64f977e99ff9e7d6b7033828332a98a86bc00a6ac08";

pub fn literal(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16)
                .expect("frozen vector contains a hex byte")
        })
        .collect()
}

pub fn assert_independent_frame_checksums(bytes: &[u8]) {
    let footer_start = bytes.len() - 32;
    assert_eq!(
        independent_sha256(&bytes[116..footer_start]),
        bytes[84..116]
    );
    assert_eq!(
        independent_sha256(&bytes[..footer_start]),
        bytes[footer_start..]
    );
}

pub fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub fn scope(store_byte: u8, segment: u64, generation: u64, length: u64) -> PhysicalArtifactScope {
    PhysicalArtifactScope::wal_frame(
        store(store_byte),
        WalSegmentIdentity::new(segment, generation).unwrap(),
        PhysicalByteRange::new(FRAME_OFFSET, length).unwrap(),
    )
}

pub fn rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_wal_frame(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope) {
        (WalFrameIntegrityValidation::Rejected(rejection), counters) => (rejection, counters),
        (WalFrameIntegrityValidation::Intact(_), _) => panic!("WAL vector unexpectedly validated"),
    }
}

pub fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    offset: u64,
    length: u64,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
            scope,
            cause,
            PhysicalByteRange::new(FRAME_OFFSET + offset, length).unwrap(),
            field,
            blast_radius,
        ))
    );
}

pub fn assert_rejected_counters(
    counters: PhysicalIntegrityObservationCounters,
    bytes: u64,
    cause: PhysicalDamageCause,
) {
    assert_eq!(counters.family(), PhysicalIntegrityArtifactFamily::WalFrame);
    assert_eq!(
        (counters.inspected_frames(), counters.inspected_bytes()),
        (1, bytes)
    );
    assert_eq!(
        (counters.intact_frames(), counters.rejected_frames()),
        (0, 1)
    );
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(cause)),
        1
    );
}
