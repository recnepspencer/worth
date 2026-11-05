//! Delete the authoritative root membership while leaving the field and frame intact.
use super::{original, recovery::publish, transaction::fixture};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::{content::RelationalContentDenial, durability::*};

#[test]
fn content_decoded_root_omission_refuses_without_using_envelope_bytes_as_a_cache() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    publish(&runtime, &execution, &contract);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let mut bytes = checkpoint.bytes().to_vec();
    // The manifest precedes members/envelopes. Exact key framing distinguishes
    // root `content` from schema `content_formats` and region `content_digest`.
    let key = b"\xa7content";
    let start = bytes
        .windows(key.len())
        .position(|word| word == key)
        .unwrap()
        + key.len();
    assert_eq!(
        bytes[start], 0x91,
        "fixture root retains one distinct frame"
    );
    let end = value_end(&bytes, start);
    bytes.splice(start..end, [0x90]);
    assert!(
        bytes
            .windows(super::expected::FRAME.len())
            .any(|word| word == super::expected::FRAME),
        "transport still contains the correct bytes"
    );
    let corrupt = RelationalNativeCheckpoint::from_untrusted_bytes(bytes);
    let (mut target, _) = fixture();
    let before = target
        .observe_branch(&target.main_branch_identity())
        .unwrap()
        .0;
    let refusal = target
        .durability_recovery()
        .restore_native_checkpoint(&execution, &corrupt)
        .unwrap_err();
    assert!(
        matches!(
            refusal,
            RelationalRecoveryStop::Content(RelationalContentDenial::MissingContent)
        ),
        "{refusal:?}"
    );
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .0,
        before
    );
}

// Test-side MessagePack framing only. It neither interprets fields nor copies
// Native's closure checker, content writer, digest or recovery algorithm.
pub(super) fn value_end(bytes: &[u8], start: usize) -> usize {
    fn size(bytes: &[u8], at: usize, width: usize) -> usize {
        bytes[at..at + width].iter().fold(0_usize, |value, byte| {
            value
                .checked_mul(256)
                .unwrap()
                .checked_add(usize::from(*byte))
                .unwrap()
        })
    }
    let marker = bytes[start];
    let (mut at, children) = match marker {
        0x00..=0x7f | 0xc0 | 0xc2 | 0xc3 | 0xe0..=0xff => return start + 1,
        0x80..=0x8f => (start + 1, usize::from(marker & 15) * 2),
        0x90..=0x9f => (start + 1, usize::from(marker & 15)),
        0xa0..=0xbf => return start + 1 + usize::from(marker & 31),
        0xc4 | 0xd9 => return start + 2 + size(bytes, start + 1, 1),
        0xc5 | 0xda => return start + 3 + size(bytes, start + 1, 2),
        0xc6 | 0xdb => return start + 5 + size(bytes, start + 1, 4),
        0xcc | 0xd0 => return start + 2,
        0xcd | 0xd1 => return start + 3,
        0xce | 0xd2 | 0xca => return start + 5,
        0xcf | 0xd3 | 0xcb => return start + 9,
        0xdc => (start + 3, size(bytes, start + 1, 2)),
        0xdd => (start + 5, size(bytes, start + 1, 4)),
        0xde => (start + 3, size(bytes, start + 1, 2) * 2),
        0xdf => (start + 5, size(bytes, start + 1, 4) * 2),
        marker => panic!("fixture unexpectedly uses MessagePack marker {marker:x}"),
    };
    for _ in 0..children {
        at = value_end(bytes, at);
    }
    at
}
