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
    for (legacy, expected) in [
        (false, RecoveryFailureClass::CorruptCheckpoint),
        (true, RecoveryFailureClass::UnsupportedLegacySemantics),
    ] {
        let mut damaged = bytes.to_vec();
        if legacy {
            damaged[native_start..native_start + b"WORTH-REL-CHK17\n".len()]
                .copy_from_slice(b"WORTH-REL-CHK17\n");
        } else {
            damaged[native_start + b"WORTH-REL-CHK18\n".len()] = 0xc1;
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
    let restored = world::try_restore(original).unwrap();
    assert_eq!(world::read(&restored), world::read(&host));
}
