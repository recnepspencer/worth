use super::support::*;
use crate::facade::{
    BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId,
    BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict,
};

#[test]
fn phase_18_closeout_seals_all_required_suite_rows() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let artifact = runtime
        .seal_subscription_temporal_async_certification_closeout(temporal_async_closeout_request(
            &runtime,
            resource_request,
        ))
        .expect("phase 18 closeout should seal");
    let matrix =
        runtime.inspect_subscription_temporal_async_certification_support_matrix(&artifact);

    assert_eq!(
        matrix.rows().len(),
        BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::all().len()
    );
    assert!(matrix.rows().iter().any(|row| {
        row.suite_id()
            == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite44UnsupportedBasis
            && row.verdict()
                == BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict::TypedRejectionProven
            && row.primary_failure_boundary()
                == Some(BridgeSubscriptionCertificationFailureBoundary::BasisDrift)
    }));
    assert!(matrix.rows().iter().any(|row| {
        row.suite_id()
            == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite46UnsupportedNeighbor
            && row.verdict()
                == BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict::TypedRejectionProven
            && row.primary_failure_boundary()
                == Some(BridgeSubscriptionCertificationFailureBoundary::IllegalSharingReuse)
    }));
    assert!(matrix.rows().iter().any(|row| {
        row.suite_id()
            == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite48TemporalAsyncBundleParity
            && row.verdict()
                == BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict::ParityBandProven
    }));
    assert!(matrix.rows().iter().any(|row| {
        row.suite_id()
            == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite50MergedCloseout
            && row.verdict()
                == BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict::CloseoutProven
            && row.evidence_digest() == artifact.digest()
    }));
    assert_eq!(artifact.counters().phase_18_closeout_artifact_count(), 1);
    assert_eq!(
        artifact.counters().phase_18_support_matrix_count(),
        BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::all().len()
    );
}

#[test]
fn equivalent_closeout_inputs_produce_equal_digests() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let left_runtime = runtime(BridgeRuntimePolicy::development());
    let right_runtime = runtime(BridgeRuntimePolicy::development());
    let left = left_runtime
        .seal_subscription_temporal_async_certification_closeout(temporal_async_closeout_request(
            &left_runtime,
            resource_request,
        ))
        .expect("left closeout should seal");
    let right = right_runtime
        .seal_subscription_temporal_async_certification_closeout(temporal_async_closeout_request(
            &right_runtime,
            resource_request,
        ))
        .expect("right closeout should seal");

    assert_eq!(left.digest(), right.digest());
    assert_eq!(
        left.support_matrix().digest(),
        right.support_matrix().digest()
    );
}

#[test]
fn different_temporal_async_parity_bands_produce_unequal_closeout_digests() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let left_runtime = runtime(BridgeRuntimePolicy::development());
    let right_runtime = runtime(BridgeRuntimePolicy::development());
    let left = left_runtime
        .seal_subscription_temporal_async_certification_closeout(
            temporal_async_closeout_request_with_seed(&left_runtime, "left", resource_request),
        )
        .expect("left closeout should seal");
    let right = right_runtime
        .seal_subscription_temporal_async_certification_closeout(
            temporal_async_closeout_request_with_seed(&right_runtime, "right", resource_request),
        )
        .expect("right closeout should seal");

    assert_ne!(left.digest(), right.digest());
    assert_ne!(
        left.support_matrix().digest(),
        right.support_matrix().digest()
    );
}

#[test]
fn incomplete_temporal_async_parity_band_rejects_closeout() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let rejection = runtime
        .seal_subscription_temporal_async_certification_closeout(divergent_closeout_request(
            &runtime,
            resource_request,
        ))
        .expect_err("phase 18 closeout must reject incomplete parity band");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionTemporalAsyncCertificationCloseoutRejectionKind::TemporalAsyncParityBandIncomplete
    );
}

#[test]
fn suite_48_support_matrix_row_binds_full_temporal_async_parity_band() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let request = temporal_async_closeout_request(&runtime, resource_request);
    let expected_band_digest = request.temporal_async_parity_band_digest();
    let artifact = runtime
        .seal_subscription_temporal_async_certification_closeout(request)
        .expect("phase 18 closeout should seal");
    let suite_48 = artifact
        .support_matrix()
        .rows()
        .iter()
        .find(|row| {
            row.suite_id()
                == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite48TemporalAsyncBundleParity
        })
        .expect("suite 48 row must exist");

    assert_eq!(
        suite_48.verdict(),
        BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict::ParityBandProven
    );
    assert_eq!(suite_48.evidence_digest(), expected_band_digest);
}
