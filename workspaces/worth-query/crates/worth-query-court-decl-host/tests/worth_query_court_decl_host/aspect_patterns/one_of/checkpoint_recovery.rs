//! Both Native checkpoint classes are matchable through the Query audience.
use super::*;
use sha2::{Digest, Sha256};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

const QUERY_BODY_START: usize = 8 + 32;
const NATIVE_LEN_OFFSET: usize = QUERY_BODY_START + 6 + 8;
const NATIVE_START: usize = QUERY_BODY_START + 6 + 8 + 8 + 8 + 8 + 8;
const CURRENT_NATIVE_MAGIC: &[u8] = b"WORTH-REL-CHK23\n";

fn reseal_query_frame(frame: &mut [u8]) {
    let checksum = Sha256::digest(&frame[QUERY_BODY_START..]);
    frame[8..QUERY_BODY_START].copy_from_slice(&checksum);
}

fn replace_native_magic(original: &[u8], native_len: u64, replacement: &[u8]) -> Vec<u8> {
    let mut frame = original.to_vec();
    frame.splice(
        NATIVE_START..NATIVE_START + CURRENT_NATIVE_MAGIC.len(),
        replacement.iter().copied(),
    );
    let updated_len = (native_len as isize
        + (replacement.len() as isize - CURRENT_NATIVE_MAGIC.len() as isize))
        as u64;
    frame[NATIVE_LEN_OFFSET..NATIVE_LEN_OFFSET + 8].copy_from_slice(&updated_len.to_be_bytes());
    reseal_query_frame(&mut frame);
    frame
}

fn assert_native_inspection_class(
    host: &world::Host,
    label: &str,
    bytes: &[u8],
    expected: RecoveryFailureClass,
) {
    host.with_host_advancement(|phase| {
        let denial = host
            .runtime()
            .inspect_application_checkpoint(&phase, bytes)
            .expect_err("damaged or retired Native checkpoint must fail inspection");
        let WorthQueryCheckpointInspectionDenial::Native(stop) = denial else {
            panic!("{label}: wrong checkpoint inspection phase: {denial:?}");
        };
        let RelationalRecoveryStop::Durability(cause) = stop.as_ref() else {
            panic!("{label}: missing Native durability cause: {stop:?}");
        };
        assert_eq!(cause.class, expected, "{label}");
    })
    .unwrap();
}

fn assert_untrusted_restore_refused(label: &str, bytes: Vec<u8>) {
    let denial = match world::try_restore(WorthQueryApplicationCheckpoint::from_untrusted_bytes(
        ARCHIVE_DECODE,
        bytes.into_boxed_slice(),
    )) {
        Err(denial) => denial,
        Ok(_) => panic!("{label}: damaged or retired checkpoint installed"),
    };
    let WorthQueryApplicationPreparationDenial::Graph(denial) = denial else {
        panic!("{label}: wrong checkpoint reconstruction phase: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::InterimCheckpointCaptureOriginMissing,
        "{label}"
    );
}

#[test]
fn checkpoint_recovery_retains_corruption_and_unsupported_semantics_by_type() {
    let host = world::install();
    let original = world::checkpoint(&host);
    let bytes = original.bytes();
    // Query's authenticated envelope wraps an opaque current-format Native frame.
    // Mutating only the Native frame and repairing outer framing lets Native judge it.
    assert!(bytes[NATIVE_START..].starts_with(CURRENT_NATIVE_MAGIC));
    let native_len = u64::from_be_bytes(
        bytes[NATIVE_LEN_OFFSET..NATIVE_LEN_OFFSET + 8]
            .try_into()
            .unwrap(),
    );

    let mut corrupt_payload = bytes.to_vec();
    corrupt_payload[NATIVE_START + CURRENT_NATIVE_MAGIC.len()] = 0xc1;
    reseal_query_frame(&mut corrupt_payload);

    let cases: [(&str, Vec<u8>, RecoveryFailureClass); 4] = [
        (
            "corrupt-native-payload",
            corrupt_payload,
            RecoveryFailureClass::CorruptCheckpoint,
        ),
        (
            "retired-chk17",
            replace_native_magic(bytes, native_len, b"WORTH-REL-CHK17\n"),
            RecoveryFailureClass::UnsupportedLegacySemantics,
        ),
        // Current-format-only cutover: retired CHK20 and CHK20.1 frames must be
        // refused with UnsupportedLegacySemantics even after outer length/checksum repair.
        (
            "retired-chk20",
            replace_native_magic(bytes, native_len, b"WORTH-REL-CHK20\n"),
            RecoveryFailureClass::UnsupportedLegacySemantics,
        ),
        (
            "retired-chk20.1",
            replace_native_magic(bytes, native_len, b"WORTH-REL-CHK20.1\n"),
            RecoveryFailureClass::UnsupportedLegacySemantics,
        ),
    ];

    for (label, frame, expected) in cases {
        assert_native_inspection_class(&host, label, &frame, expected);
        assert_untrusted_restore_refused(label, frame);
    }

    host.with_host_advancement(|phase| {
        let current = host
            .runtime()
            .inspect_application_checkpoint(&phase, original.bytes())
            .unwrap();
        assert!(!current.live_records.is_empty());
    })
    .unwrap();

    let restored = world::try_restore(original).unwrap();
    assert_eq!(world::read(&restored), world::read(&host));
}
