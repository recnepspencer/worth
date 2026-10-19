//! Real issued native evidence, bounded wire/parse ownership, and lawful retry.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(bytes), Some(1_000_000)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap()
}

#[test]
fn native_dependency_wire_and_deep_parse_keep_the_original_payer_through_clones() {
    let host = crate::test_execution_policy::serial_request();
    let system = worth_execution::ExecutionRequest::serial(&host);
    let fixture = authorization_fixture(&system);
    let payer = lease(64 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&payer);
    let stamp = stamp_now(&execution, &fixture);
    let deep_charge = payer.charged_memory_bytes();
    assert!(deep_charge > 0);
    // Existing codec is the canonical-byte oracle; no second native encoder.
    let expected = rmp_serde::to_vec_named(&stamp).unwrap();
    let expected_hex = expected
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let wire = stamp.to_wire(&execution).unwrap();
    assert_eq!(wire.as_str(), expected_hex);
    assert_eq!(wire.charged_payload_bytes(), Some(wire.len() as u64));
    let retained_stamp = stamp.clone();
    drop(stamp);
    assert_eq!(
        payer.charged_memory_bytes(),
        deep_charge + wire.len() as u64
    );
    let parsed = RelationalAuthorizationDurableDependencies::from_wire(&execution, &wire).unwrap();
    assert!(parsed.matches(&retained_stamp));
    drop(retained_stamp);
    drop(wire);
    let parsed_charge = payer.charged_memory_bytes();
    assert!(parsed_charge > 0);
    let retained = parsed.clone();
    drop(parsed);
    assert_eq!(payer.charged_memory_bytes(), parsed_charge);
    assert!(retained
        .source_revisions_match(
            &fixture.runtime,
            &execution,
            &fixture.runtime.visibility_authority().snapshot()
        )
        .unwrap());
    assert_eq!(payer.charged_memory_bytes(), parsed_charge);
    drop(retained);
    assert_eq!(payer.charged_memory_bytes(), 0);
}

#[test]
fn native_dependency_deep_parse_refusal_preserves_original_wire_and_head_for_retry() {
    let host = crate::test_execution_policy::serial_request();
    let system = worth_execution::ExecutionRequest::serial(&host);
    let fixture = authorization_fixture(&system);
    let source = stamp_now(&system, &fixture);
    let wire = source.to_wire(&system).unwrap();
    assert_eq!(wire.charged_payload_bytes(), None); // Selected System, never a fake lease.
    let original = wire.clone();
    let before = fixture.runtime.visibility_authority().snapshot();
    // Exact hex buffer fits. Refusal MUST be deep storage, before owned serde DTO.
    let tight = lease((wire.len() / 2) as u64);
    let execution = worth_execution::ExecutionRequest::leased(&tight);
    let denied =
        RelationalAuthorizationDurableDependencies::from_wire(&execution, &wire).unwrap_err();
    assert!(
        matches!(denied, RelationalAuthorizationDependencyDenial::Execution(
        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested, admitted: 0, ancestor: 0,
        }) if requested > 0)
    );
    assert_eq!(tight.charged_memory_bytes(), 0);
    assert_eq!(wire, original);
    assert_eq!(
        fixture.runtime.visibility_authority().snapshot().version_id,
        before.version_id
    );
    let retry = lease(64 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&retry);
    let decoded = RelationalAuthorizationDurableDependencies::from_wire(&execution, &wire).unwrap();
    assert!(source.matches(&decoded));
    drop(decoded);
    assert_eq!(retry.charged_memory_bytes(), 0);
    let cancelled = retry.controlled_child(CancellationToken::new(), None);
    // Existing original request control, not a codec-only observer or reset.
    let cancellation = worth_execution::CancellationSource::new();
    let stopped = cancelled.controlled_child(cancellation.token(), None);
    cancellation.cancel();
    let denied = RelationalAuthorizationDurableDependencies::from_wire(
        &worth_execution::ExecutionRequest::leased(&stopped),
        &wire,
    )
    .unwrap_err();
    assert_eq!(
        denied,
        RelationalAuthorizationDependencyDenial::Execution(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(retry.charged_memory_bytes(), 0);
    assert_eq!(wire, original);
}
