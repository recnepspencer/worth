pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
pub(crate) use worth_store_physical_format::WalSegmentIdentity;
pub(crate) use worth_store_physical_integrity::{
    validate_wal_frame, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalDamageLocalization, PhysicalFormatField,
    PhysicalIntegrityObservationCounters, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact, WalFrameIntegrityValidation,
};

pub(crate) use crate::sha256::independent_sha256;

pub(crate) const FRAME_OFFSET: u64 = 8_192;
pub(crate) const CLEAN_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
pub(crate) const SEGMENT_HEX: &str = "574f52544857414c0100740002000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020308678175202523d47e928bae246a14bb4ac1daf638620b47aad28f81d770cc30b";
pub(crate) const GENERATION_HEX: &str = "574f52544857414c0100740001000000000000000300000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020303938564097b63d061e658bdbc8be3e920d3b384e4caf9e411112d17159f6988b";
pub(crate) const LSN_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000040000000000000003000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c00f77cb1d5b1bd504ace84fc0ee7f04b00a8f35e94b4d998111a32f35cfb28c";
pub(crate) const LENGTH_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000400000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c7617fee7f6b6b3eafa436e8acce7518dd9378720b74ec141e6a2c741dd24cdf";
pub(crate) const VERSION_HEX: &str = "574f52544857414c0200740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030f3b1a6b324b3ee017668c2fd33f1e5198da8228575b0c8e048a3c5e5c7bf387b";
pub(crate) const HEADER_LENGTH_HEX: &str = "574f52544857414c0100750001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020303769bf4c44ac01dbbe8a36815201340bf540b7580db7658a261aca8565fe603c";
pub(crate) const PAYLOAD_CHECKSUM_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768f1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b91020303d6d138251061e3f65f95ae975b5e68904562eb5ef4f2c5ddea8b0e0b31a5317";
pub(crate) const FOOTER_CHECKSUM_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa680";
pub(crate) const PAYLOAD_BYTE_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9112030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
pub(crate) const MAGIC_HEX: &str = "564f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
pub(crate) const ZERO_SEGMENT_HEX: &str = "574f52544857414c0100740000000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c12f4b99b324fd9949efc5d22402659d8f1780a21c277cd359e06409e8894088";
pub(crate) const ZERO_GENERATION_HEX: &str = "574f52544857414c0100740001000000000000000000000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b910203068430e7328eff20ce22db64f977e99ff9e7d6b7033828332a98a86bc00a6ac08";

pub(crate) fn literal(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16)
                .expect("frozen vector contains a hex byte")
        })
        .collect()
}

pub(crate) fn assert_independent_frame_checksums(bytes: &[u8]) {
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

pub(crate) fn store(byte: u8) -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [byte; 16],
        )
        .unwrap(),
    )
    .published_identity()
}

pub(crate) fn scope(
    store_byte: u8,
    segment: u64,
    generation: u64,
    length: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::wal_frame(
        store(store_byte),
        worth_store_physical_format::WalSegmentIdentity::new(segment, generation).unwrap(),
        worth_store_physical_integrity::PhysicalByteRange::new(FRAME_OFFSET, length).unwrap(),
    )
}

pub(crate) fn rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_wal_frame(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    ) {
        (
            worth_store_physical_integrity::WalFrameIntegrityValidation::Rejected(rejection),
            counters,
        ) => (rejection, counters),
        (worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(_), _) => {
            panic!("WAL vector unexpectedly validated")
        }
    }
}

pub(crate) fn assert_damage(
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
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
            worth_store_physical_integrity::PhysicalDamageLocalization::new(
                scope,
                cause,
                worth_store_physical_integrity::PhysicalByteRange::new(
                    FRAME_OFFSET + offset,
                    length
                )
                .unwrap(),
                field,
                blast_radius,
            )
        )
    );
}

pub(crate) fn assert_rejected_counters(
    counters: PhysicalIntegrityObservationCounters,
    bytes: u64,
    cause: PhysicalDamageCause,
) {
    assert_eq!(counters.family(), worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::WalFrame);
    assert_eq!(
        (counters.inspected_frames(), counters.inspected_bytes()),
        (1, bytes)
    );
    assert_eq!(
        (counters.intact_frames(), counters.rejected_frames()),
        (0, 1)
    );
    assert_eq!(
        counters.rejected_for(
            worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause)
        ),
        1
    );
}
