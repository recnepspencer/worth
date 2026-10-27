use super::support::*;
use crate::adapter::BridgeHistoricalLineageRequest;
use crate::continuity::BridgeContinuityAuthorityBasis;
use crate::error::BridgeLineageSourceErrorKind;
use crate::facade::RuntimeBridgeBuilder;
use crate::truth_identity_fixtures::{truth_branch, truth_snapshot};

#[test]
fn build_accepts_optional_continuity_lineage_source() {
    let serial_request = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .with_continuity_lineage_source(TestLineageSource)
        .register_mapping(exact_registration("user-profile-name"))
        .build()
        .expect("builder should accept continuity lineage source");

    let authority_basis =
        BridgeContinuityAuthorityBasis::new(truth_branch("main"), truth_snapshot(1, 1));
    let source = runtime
        .continuity_lineage_source()
        .expect("continuity lineage source should be present");
    let authority = source
        .historical_lineage(
            BridgeHistoricalLineageRequest::new(
                authority_basis,
                native_prior_field_slice("entity:test", "aspect.test", "field_test"),
            ),
            execution,
        )
        .expect("test lineage source should answer");

    assert!(authority
        .lineage_digest()
        .starts_with("historical-lineage-authority:sha256:"));
}

#[test]
fn build_accepts_optional_writeback_authority() {
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .with_writeback_authority(TestWritebackAuthority)
        .register_mapping(exact_registration("user-profile-name"))
        .build()
        .expect("builder should accept optional writeback authority");

    assert!(runtime.writeback_authority().is_some());
}

#[test]
fn continuity_lineage_source_can_return_typed_unsupported_class_failure() {
    let serial_request = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .with_continuity_lineage_source(TestUnsupportedLineageSource)
        .register_mapping(exact_registration("user-profile-name"))
        .build()
        .expect("builder should accept continuity lineage source");

    let source = runtime
        .continuity_lineage_source()
        .expect("continuity lineage source should be present");
    let error = source
        .historical_lineage(
            BridgeHistoricalLineageRequest::new(
                BridgeContinuityAuthorityBasis::new(truth_branch("main"), truth_snapshot(1, 1)),
                native_prior_field_slice("relation:test", "aspect.test", "field_test"),
            ),
            execution,
        )
        .expect_err("unsupported continuity class should be typed");

    assert_eq!(
        error.kind(),
        BridgeLineageSourceErrorKind::UnsupportedContinuityClass
    );
}
