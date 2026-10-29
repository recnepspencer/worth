pub(crate) use {worth_store_offline_integrity_observer::OfflineIntegrityObservationCounters};

pub(crate) use {crate::root_protocol_corruption_matrix::Operator};
pub(crate) use {crate::store_offline_integrity_observer_support::Target};

pub(crate) struct ExpectedCounters {
    pub(crate) entries: u64,
    pub(crate) bytes: u64,
    pub(crate) files: u64,
    pub(crate) open_high_water: u64,
    pub(crate) checksums: u64,
    pub(crate) namespace_decoders: u64,
    pub(crate) frames: u64,
    pub(crate) selectors: u64,
    pub(crate) roots: u64,
    pub(crate) duplicates: u64,
    pub(crate) missing: u64,
    pub(crate) depth: u64,
}

pub(crate) fn expected_counters(target: Target, operator: Operator) -> ExpectedCounters {
    let selector = target != crate::store_offline_integrity_observer_support::Target::Root;
    let mut expected = match operator {
        crate::root_protocol_corruption_matrix::Operator::B | crate::root_protocol_corruption_matrix::Operator::K => ExpectedCounters {
            entries: 5,
            bytes: 670,
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
        crate::root_protocol_corruption_matrix::Operator::L => ExpectedCounters {
            entries: 5,
            bytes: 670,
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
        crate::root_protocol_corruption_matrix::Operator::S | crate::root_protocol_corruption_matrix::Operator::P => ExpectedCounters {
            entries: 5,
            bytes: 670,
            files: 16,
            open_high_water: 5,
            checksums: 4,
            namespace_decoders: 1,
            frames: 3,
            selectors: 2,
            roots: 1,
            duplicates: 0,
            missing: u64::from(selector && matches!(operator, crate::root_protocol_corruption_matrix::Operator::P)),
            depth: 4,
        },
        crate::root_protocol_corruption_matrix::Operator::T => ExpectedCounters {
            entries: 5,
            bytes: 669,
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
        crate::root_protocol_corruption_matrix::Operator::R if selector => ExpectedCounters {
            entries: 4,
            bytes: 563,
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
        crate::root_protocol_corruption_matrix::Operator::R => ExpectedCounters {
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
        crate::root_protocol_corruption_matrix::Operator::D if selector => ExpectedCounters {
            entries: 6,
            bytes: 670,
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
        crate::root_protocol_corruption_matrix::Operator::D => ExpectedCounters {
            entries: 6,
            bytes: 670,
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
        crate::root_protocol_corruption_matrix::Operator::USchema | crate::root_protocol_corruption_matrix::Operator::UFormat => ExpectedCounters {
            entries: 5,
            bytes: 670,
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
    expected.missing += if selector || matches!(operator, crate::root_protocol_corruption_matrix::Operator::D) {
        3
    } else {
        1
    };
    if cfg!(windows) {
        // Each fully read file makes three high-resolution path queries;
        // a cached hard-link alias returns after the first two.
        expected.files += match operator {
            crate::root_protocol_corruption_matrix::Operator::R => 9,
            crate::root_protocol_corruption_matrix::Operator::D => 14,
            _ => 12,
        };
        expected.open_high_water += 1;
    }
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

pub(crate) const fn lane(selector: bool, selector_value: u64, root_value: u64) -> u64 {
    if selector {
        selector_value
    } else {
        root_value
    }
}