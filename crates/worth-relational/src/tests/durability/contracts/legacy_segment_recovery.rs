use super::*;

#[test]
fn native_current_recovery_refuses_old_segments_without_rewriting_evidence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for fixture in [
        include_str!("../../../../tests/fixtures/worth_query_9_16_1_1_native_segment.hex"),
        include_str!("../../../../tests/fixtures/worth_query_9_16_1_1_non_main_segment.hex"),
        include_str!(
            "../../../../tests/fixtures/worth_query_9_16_1_1_metadata_lineage_segment.hex"
        ),
        include_str!(
            "../../../../tests/fixtures/worth_query_9_16_1_1_rejected_lineage_segment.hex"
        ),
    ] {
        let source = persisted_runtime_with_test_schema();
        install_legacy_segment(&source, fixture);
        let path = source
            .config()
            .durability
            .policy
            .store_layout
            .unwrap()
            .root_path
            .join("segments/segment-1.worth-segment");
        let before = std::fs::read(&path).unwrap();
        let plan = source
            .durability()
            .recovery_plan(
                execution,
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let mut target = persisted_runtime_with_test_schema();
        let error = expect_durability_failure(
            target
                .durability_recovery()
                .recover(execution, plan)
                .unwrap_err(),
        );
        assert_eq!(
            error.class,
            crate::durability::data::RecoveryFailureClass::UnsupportedLegacySemantics
        );
        assert_eq!(target.history().immutable_commit_count(), 0);
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}

fn install_legacy_segment(runtime: &crate::runtime::RelationalRuntime, fixture: &str) {
    let layout = runtime
        .config
        .durability
        .policy
        .store_layout
        .clone()
        .expect("persisted test runtime has a store layout");
    crate::durability::log::local_store::load_store_from_disk(runtime)
        .expect("legacy store manifest initializes");
    let segment_path = crate::durability::log::local_store::segment_file_path(
        &layout,
        crate::durability::data::DurableSegmentId(1),
    );
    std::fs::write(&segment_path, decode_hex(fixture))
        .expect("real 9.16.1.1 segment is installed in the native store");
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    let digits = encoded
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert_eq!(digits.len() % 2, 0, "fixture hex has complete bytes");
    digits
        .chunks_exact(2)
        .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
        .collect()
}

fn hex_digit(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        b'a'..=b'f' => digit - b'a' + 10,
        _ => panic!("fixture contains a non-hex digit"),
    }
}
