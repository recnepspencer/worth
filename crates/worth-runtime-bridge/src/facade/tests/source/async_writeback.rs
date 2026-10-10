use super::support::{
    admitted_authoritative_request_response_completion,
    admitted_preview_subscription_backed_completion, aspect_reconciliation_intent,
    authoritative_writeback_request, integer_projected_state_diff_intent,
    newer_authoritative_request_identity, runtime_with_authority, runtime_with_rejecting_authority,
};
use crate::facade::{
    BridgeAsyncRequestTruthViewBasis, BridgeAsyncWritebackAdmissionRequest,
    BridgeAsyncWritebackNoopClass, BridgeAsyncWritebackRejectedClass,
    BridgeAsyncWritebackRejectionKind,
};
use worth_signal::facade::NodeId;

#[test]
fn authoritative_completion_writeback_commits_and_retains_causality_transfer() {
    let runtime = runtime_with_authority();
    let completion =
        admitted_authoritative_request_response_completion(&runtime, NodeId::new(510, 0), "a");
    let admitted = runtime
        .admit_async_writeback(authoritative_writeback_request(&completion, "commit-a"))
        .expect("authoritative completion should admit async writeback");
    let staged = runtime
        .stage_async_writeback_effect(&admitted)
        .expect("authoritative completion should stage async writeback");
    let report = runtime.commit_async_writeback(&staged);
    let committed = report
        .committed()
        .expect("authoritative writeback should commit");

    assert_eq!(
        committed.authority_receipt().outcome_class(),
        crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit
    );
    assert_eq!(
        committed.causality_transfer().completion_identity(),
        completion.completion_identity()
    );
    assert_eq!(
        committed.causality_transfer().request_identity(),
        completion.request_identity().request_identity().as_str()
    );
    assert_eq!(committed.counters().committed_count(), 1);
}

#[test]
fn duplicate_completion_becomes_explicit_noop() {
    let runtime = runtime_with_authority();
    let completion =
        admitted_authoritative_request_response_completion(&runtime, NodeId::new(511, 0), "b");
    let admitted = runtime
        .admit_async_writeback(authoritative_writeback_request(&completion, "commit-b"))
        .expect("authoritative completion should admit async writeback");
    let staged = runtime
        .stage_async_writeback_effect(&admitted)
        .expect("authoritative completion should stage async writeback");
    let first = runtime.commit_async_writeback(&staged);
    assert!(first.committed().is_some());
    let second = runtime.commit_async_writeback(&staged);
    let noop = second
        .noop()
        .expect("duplicate completion should become explicit noop");

    assert_eq!(
        noop.noop_class(),
        BridgeAsyncWritebackNoopClass::DuplicateCompletion
    );
    assert_eq!(noop.counters().duplicate_noop_count(), 1);
}

#[test]
fn preview_origin_completion_is_rejected() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_authority();
    let completion = admitted_preview_subscription_backed_completion(
        &runtime,
        NodeId::new(512, 0),
        "preview",
        resource_request,
    );
    let rejection = runtime
        .admit_async_writeback(authoritative_writeback_request(
            &completion,
            "commit-preview",
        ))
        .expect_err("preview completion should not admit authoritative writeback");

    assert_eq!(
        rejection.kind(),
        BridgeAsyncWritebackRejectionKind::PreviewCompletionForbidden
    );
}

#[test]
fn truth_changed_completion_rejects_before_staging() {
    let runtime = runtime_with_authority();
    let completion =
        admitted_authoritative_request_response_completion(&runtime, NodeId::new(513, 0), "c");
    let rejection = runtime
        .admit_async_writeback(BridgeAsyncWritebackAdmissionRequest::authoritative_commit(
            &completion,
            super::support::projected_state_diff_intent("commit-c"),
            BridgeAsyncRequestTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch_fixture("truth-main:c-newer"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:c-newer"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:c-newer"),
            ),
        ))
        .expect_err("displaced authoritative completion should reject writeback");

    assert_eq!(
        rejection.kind(),
        BridgeAsyncWritebackRejectionKind::CurrentAuthorityDrifted
    );
}

#[test]
fn mismatched_current_authority_request_rejects_typed() {
    let runtime = runtime_with_authority();
    let completion =
        admitted_authoritative_request_response_completion(&runtime, NodeId::new(513, 1), "c2");
    let newer = newer_authoritative_request_identity(&runtime, NodeId::new(513, 1), "c2-newer");
    let rejection = runtime
        .admit_async_writeback(
            authoritative_writeback_request(&completion, "commit-c2")
                .with_current_authoritative_request(newer),
        )
        .expect_err("mismatched current authority request should reject writeback");

    assert_eq!(
        rejection.kind(),
        BridgeAsyncWritebackRejectionKind::CurrentAuthorityDrifted
    );
}

#[test]
fn mapper_effect_class_mismatch_rejects_typed() {
    let runtime = runtime_with_authority();
    let completion =
        admitted_authoritative_request_response_completion(&runtime, NodeId::new(514, 0), "d");
    let rejection = runtime
        .admit_async_writeback(
            crate::facade::BridgeAsyncWritebackAdmissionRequest::authoritative_commit(
                &completion,
                aspect_reconciliation_intent("wrong-family"),
                completion
                    .request_identity()
                    .basis_binding()
                    .truth_view_basis()
                    .clone(),
            ),
        )
        .expect_err("phase 10 should reject non projected-state-diff writeback effects");

    assert_eq!(
        rejection.kind(),
        BridgeAsyncWritebackRejectionKind::MapperEffectClassUnsupported
    );
}

#[test]
fn authority_rejection_is_retained_as_explicit_outcome() {
    let runtime = runtime_with_rejecting_authority();
    let completion =
        admitted_authoritative_request_response_completion(&runtime, NodeId::new(515, 0), "e");
    let admitted = runtime
        .admit_async_writeback(authoritative_writeback_request(&completion, "commit-e"))
        .expect("authoritative completion should admit async writeback");
    let staged = runtime
        .stage_async_writeback_effect(&admitted)
        .expect("authoritative completion should stage async writeback");
    let report = runtime.commit_async_writeback(&staged);
    let rejected = report
        .rejected()
        .expect("rejecting authority should retain an explicit rejected writeback");

    assert_eq!(
        rejected.rejected_class(),
        BridgeAsyncWritebackRejectedClass::AuthorityRejected
    );
    assert_eq!(rejected.counters().authority_rejection_count(), 1);
}

#[test]
fn mapper_failure_is_retained_as_typed_rejection() {
    let runtime = runtime_with_authority();
    let completion =
        admitted_authoritative_request_response_completion(&runtime, NodeId::new(515, 1), "e2");
    let admitted = runtime
        .admit_async_writeback(BridgeAsyncWritebackAdmissionRequest::authoritative_commit(
            &completion,
            integer_projected_state_diff_intent(7),
            completion
                .request_identity()
                .basis_binding()
                .truth_view_basis()
                .clone(),
        ))
        .expect("authoritative completion should admit async writeback");
    let rejection = runtime
        .stage_async_writeback_effect(&admitted)
        .expect_err("unsupported mapper payload should reject during staging");

    assert_eq!(
        rejection.kind(),
        BridgeAsyncWritebackRejectionKind::MapperFailed
    );
}

#[test]
fn loop_prevention_evidence_is_stable_across_equivalent_runtimes() {
    let runtime_a = runtime_with_authority();
    let runtime_b = runtime_with_authority();
    let completion_a =
        admitted_authoritative_request_response_completion(&runtime_a, NodeId::new(516, 0), "f");
    let completion_b =
        admitted_authoritative_request_response_completion(&runtime_b, NodeId::new(516, 0), "f");
    let admitted_a = runtime_a
        .admit_async_writeback(authoritative_writeback_request(&completion_a, "commit-f"))
        .expect("runtime a should admit async writeback");
    let admitted_b = runtime_b
        .admit_async_writeback(authoritative_writeback_request(&completion_b, "commit-f"))
        .expect("runtime b should admit async writeback");
    let staged_a = runtime_a
        .stage_async_writeback_effect(&admitted_a)
        .expect("runtime a should stage async writeback");
    let staged_b = runtime_b
        .stage_async_writeback_effect(&admitted_b)
        .expect("runtime b should stage async writeback");

    assert_eq!(
        staged_a.loop_prevention().digest(),
        staged_b.loop_prevention().digest()
    );
    assert_eq!(
        staged_a.loop_prevention().disposition(),
        staged_b.loop_prevention().disposition()
    );
}
