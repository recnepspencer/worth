//! Both Native checkpoint classes are matchable through the Query audience.
use super::*;
use sha2::{Digest, Sha256};

#[test]
fn checkpoint_recovery_retains_corruption_and_unsupported_semantics_by_type() {
    let host = world::install();
    let original = world::checkpoint(&host);
    let bytes = original.bytes();
    // Query's authenticated envelope wraps an opaque Native frame. Corrupt only
    // that frame and authenticate the outer transport so Native judges it.
    let native_start = 8 + 32 + 6 + 8 + 8 + 8;
    let native_magic = b"WORTH-REL-CHK20.1\n";
    assert!(bytes[native_start..].starts_with(native_magic));
    for (legacy, expected) in [
        (false, RecoveryFailureClass::CorruptCheckpoint),
        (true, RecoveryFailureClass::UnsupportedLegacySemantics),
    ] {
        let mut damaged = bytes.to_vec();
        if legacy {
            damaged[native_start..native_start + b"WORTH-REL-CHK17\n".len()]
                .copy_from_slice(b"WORTH-REL-CHK17\n");
        } else {
            damaged[native_start + native_magic.len()] = 0xc1;
        }
        let checksum = Sha256::digest(&damaged[40..]);
        damaged[8..40].copy_from_slice(&checksum);
        let denial = match world::try_restore(
            WorthQueryApplicationCheckpoint::from_untrusted_bytes(damaged.into_boxed_slice()),
        ) {
            Err(denial) => denial,
            Ok(_) => panic!("damaged checkpoint installed"),
        };
        let WorthQueryInMemoryApplicationDenial::Graph(denial) = denial else {
            panic!("wrong checkpoint reconstruction phase: {denial:?}")
        };
        let Some(RelationalRecoveryStop::Durability(cause)) = denial.native_recovery_cause() else {
            panic!("missing Native cause: {denial:?}")
        };
        assert_eq!(cause.class, expected);
    }
    // Reframe only the Native capability marker and outer lengths/checksum.
    // The real consumer must read the identical model through legacy admission.
    let native_len = u64::from_be_bytes(
        bytes[native_start - 16..native_start - 8]
            .try_into()
            .unwrap(),
    );
    let mut legacy = bytes.to_vec();
    let legacy_magic = b"WORTH-REL-CHK20\n";
    legacy.splice(
        native_start..native_start + native_magic.len(),
        legacy_magic.iter().copied(),
    );
    legacy[native_start - 16..native_start - 8].copy_from_slice(
        &(native_len - (native_magic.len() - legacy_magic.len()) as u64).to_be_bytes(),
    );
    let checksum = Sha256::digest(&legacy[40..]);
    legacy[8..40].copy_from_slice(&checksum);
    host.with_host_advancement(|phase| {
        let current = host
            .runtime()
            .inspect_application_checkpoint(&phase, original.bytes())
            .unwrap();
        let prior = host
            .runtime()
            .inspect_application_checkpoint(&phase, &legacy)
            .unwrap();
        assert_eq!(current.model, prior.model);
        assert_eq!(current.live_records, prior.live_records);
    })
    .unwrap();
    let restored_legacy = world::try_restore(
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(legacy.into_boxed_slice()),
    )
    .unwrap();
    assert_eq!(world::read(&restored_legacy), world::read(&host));
    let restored = world::try_restore(original).unwrap();
    assert_eq!(world::read(&restored), world::read(&host));
}
