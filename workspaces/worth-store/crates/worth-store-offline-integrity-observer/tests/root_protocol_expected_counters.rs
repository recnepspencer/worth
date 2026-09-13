use worth_store_offline_integrity_observer::OfflineIntegrityObservationCounters;

use crate::root_protocol_corruption_matrix::Operator;
use crate::support::Target;

pub(crate) struct ExpectedCounters {
    entries: u64,
    bytes: u64,
    files: u64,
    open_high_water: u64,
    checksums: u64,
    namespace_decoders: u64,
    frames: u64,
    selectors: u64,
    roots: u64,
    duplicates: u64,
    missing: u64,
    depth: u64,
}

pub(crate) fn expected_counters(target: Target, operator: Operator) -> ExpectedCounters {
    let selector = target != Target::Root;
    let mut expected = match operator {
        Operator::B | Operator::K => ExpectedCounters {
            entries: 5,
            bytes: 654,
            files: 16,
            open_high_water: 5,
            checksums: 4,
            namespace_decoders: 1,
            frames: 2,
            selectors: lane(selector, 1, 2),
            roots: lane(selector, 1, 0),
            duplicates: 0,
            missing: 0,
            depth: 4,
        },
        Operator::L => ExpectedCounters {
            entries: 5,
            bytes: 654,
            files: 16,
            open_high_water: 5,
            checksums: 3,
            namespace_decoders: 1,
            frames: 2,
            selectors: lane(selector, 1, 2),
            roots: lane(selector, 1, 0),
            duplicates: 0,
            missing: 0,
            depth: 4,
        },
        Operator::S | Operator::P => ExpectedCounters {
            entries: 5,
            bytes: 654,
            files: 16,
            open_high_water: 5,
            checksums: 4,
            namespace_decoders: 1,
            frames: 3,
            selectors: 2,
            roots: 1,
            duplicates: 0,
            missing: u64::from(selector && matches!(operator, Operator::P)),
            depth: 4,
        },
        Operator::T => ExpectedCounters {
            entries: 5,
            bytes: 653,
            files: 16,
            open_high_water: 5,
            checksums: 3,
            namespace_decoders: 1,
            frames: 2,
            selectors: lane(selector, 1, 2),
            roots: lane(selector, 1, 0),
            duplicates: 0,
            missing: 0,
            depth: 4,
        },
        Operator::R if selector => ExpectedCounters {
            entries: 4,
            bytes: 547,
            files: 12,
            open_high_water: 5,
            checksums: 3,
            namespace_decoders: 1,
            frames: 2,
            selectors: 1,
            roots: 1,
            duplicates: 0,
            missing: 1,
            depth: 4,
        },
        Operator::R => ExpectedCounters {
            entries: 4,
            bytes: 286,
            files: 11,
            open_high_water: 4,
            checksums: 3,
            namespace_decoders: 1,
            frames: 2,
            selectors: 2,
            roots: 0,
            duplicates: 0,
            missing: 1,
            depth: 3,
        },
        Operator::D if selector => ExpectedCounters {
            entries: 6,
            bytes: 654,
            files: 20,
            open_high_water: 5,
            checksums: 4,
            namespace_decoders: 1,
            frames: 3,
            selectors: 2,
            roots: 1,
            duplicates: 1,
            missing: 0,
            depth: 4,
        },
        Operator::D => ExpectedCounters {
            entries: 6,
            bytes: 654,
            files: 21,
            open_high_water: 5,
            checksums: 4,
            namespace_decoders: 1,
            frames: 3,
            selectors: 2,
            roots: 1,
            duplicates: 1,
            missing: 0,
            depth: 4,
        },
        Operator::USchema | Operator::UFormat => ExpectedCounters {
            entries: 5,
            bytes: 654,
            files: 16,
            open_high_water: 5,
            checksums: 3,
            namespace_decoders: 1,
            frames: 2,
            selectors: lane(selector, 1, 2),
            roots: lane(selector, 1, 0),
            duplicates: 0,
            missing: 0,
            depth: 4,
        },
    };
    // The literal root world deliberately has no bootstrap or free-space files.
    // Full traversal also visits the root's namespace/families entries.
    expected.entries += 3;
    expected.missing += if selector || matches!(operator, Operator::D) {
        3
    } else {
        1
    };
    expected
}

pub(crate) fn assert_counters(
    counters: &OfflineIntegrityObservationCounters,
    expected: ExpectedCounters,
) {
    assert_eq!(counters.entries_visited(), expected.entries);
    assert_eq!(counters.bytes_read(), expected.bytes);
    assert_eq!(counters.files_opened(), expected.files);
    assert_eq!(counters.open_file_high_water(), expected.open_high_water);
    assert_eq!(counters.maximum_depth_reached(), expected.depth);
    assert_eq!(counters.checksum_calculations(), expected.checksums);
    assert_eq!(
        counters.namespace_identity_payload_decoder_entries(),
        expected.namespace_decoders
    );
    assert_eq!(
        counters.checksum_validated_durable_frames(),
        expected.frames
    );
    assert_eq!(
        counters.selector_payload_decoder_entries(),
        expected.selectors
    );
    assert_eq!(
        counters.root_manifest_payload_decoder_entries(),
        expected.roots
    );
    assert_eq!(counters.duplicate_identities(), expected.duplicates);
    assert_eq!(counters.missing_artifacts(), expected.missing);
}

const fn lane(selector: bool, selector_value: u64, root_value: u64) -> u64 {
    if selector {
        selector_value
    } else {
        root_value
    }
}
