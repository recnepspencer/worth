use super::support::*;
use crate::facade::RuntimeBridgeBuilder;

#[test]
fn build_freezes_mapping_registry_before_runtime_construction() {
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(exact_registration("user-profile-name"))
        .register_aspect_mapping(exact_aspect_registration("user-profile-name-field"))
        .build()
        .expect("builder should freeze mapping registrations");

    assert_eq!(
        runtime.policy(),
        &crate::policy::BridgeRuntimePolicy::default()
    );
}

#[test]
fn delivery_reports_use_the_installed_sink_and_tier() {
    use std::sync::Arc;
    use worth_execution::report::{DenialClass, RecordingDiagnosticSink, ReportDetail};

    use crate::diagnostics::BridgeFailureSource;
    use crate::error::{BridgeDeliveryError, BridgeDeliveryErrorKind, BridgeExecutionDenial};
    use crate::policy::BridgeRuntimePolicy;
    use crate::snapshot::BridgeSnapshotReadError;
    use crate::truth_identity_fixtures::{truth_commit_fixture, truth_snapshot_fixture};

    for (policy, source_detail, summary) in [
        (
            BridgeRuntimePolicy::operational(),
            ReportDetail::Withheld,
            "Operation denied.",
        ),
        (
            BridgeRuntimePolicy::development(),
            ReportDetail::Withheld,
            "Bridge could not deliver the request.",
        ),
        (
            BridgeRuntimePolicy::forensic(),
            ReportDetail::Disclosed("execution-cancelled".into()),
            "Bridge could not deliver the request.",
        ),
    ] {
        let sink = Arc::new(RecordingDiagnosticSink::default());
        let runtime = RuntimeBridgeBuilder::new()
            .with_diagnostic_sink(sink.clone())
            .with_policy(policy)
            .with_relational_source(TestSource)
            .with_signal_sink(TestSink)
            .register_mapping(exact_registration("user-profile-name"))
            .build()
            .unwrap();
        for identity in ["private-first", "private-second"] {
            let error = BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SnapshotReadFailure(
                    BridgeSnapshotReadError::execution_denied(BridgeExecutionDenial::Cancelled),
                ),
                identity,
            );
            let source = BridgeFailureSource::new(
                truth_commit_fixture(identity),
                crate::truth_identity_fixtures::truth_patch_fixture(identity),
                truth_snapshot_fixture(identity),
                Default::default(),
            );
            runtime.diagnostics.record_delivery_failure(source, &error);
        }
        let reports = sink.take_reports();
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0], reports[1]);
        assert_eq!(
            reports[0].report_code().as_str(),
            "bridge.delivery.snapshot_read_failure"
        );
        assert_eq!(reports[0].denial_class(), DenialClass::ExternalEffect);
        assert_eq!(reports[0].summary(), summary);
        assert_eq!(reports[0].chain().len(), 2);
        assert_eq!(reports[0].chain()[1].detail(), &source_detail);
        assert!(!reports[0].to_canonical_json().contains("private-"));
    }
}
