//! The actual native output stays reserved through the custom stage and projection.
use super::*;
use crate::validation::{
    engine::InvariantRuntimeView, execution::InvariantStage, reduction::ReducedInvariantPrefix,
};

#[test]
fn native_backing_stays_on_the_original_lease_until_final_projection_is_dropped() {
    let (fixture, hooks) = fixture(2);
    let plan = fixture.merged_plan(fixture.chain_batch(1));
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &fixture.runtime,
        InvariantObservation::committed(fixture.runtime.storage_access().current_edition()),
        fixture.runtime.current_version_id(),
        Some(&plan),
        None,
    );
    let lease =
        super::super::fixture::request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let execution = ExecutionRequest::leased(&lease);
    let view =
        InvariantRuntimeView::from_runtime(&fixture.runtime).with_candidate_input_plan(&request);
    let stage = crate::validation::engine::stage_execution::execute_stage(
        &view,
        execution,
        &request,
        Some(&lease),
        InvariantStage::Native,
    )
    .unwrap()
    .unwrap();
    // This pure backing projection also exercises the envelope caller: treating
    // Native rule payloads as zero must fail even when vector backing is large.
    let mut measured = stage.envelopes[0].clone();
    let before_rule = measured.owned_allocation_capacity_bytes();
    let InvariantReportedRule::Native(rule) = &measured.results[0].result.rule else {
        panic!("the genuine stage must retain its native declaration");
    };
    let rule_backing = rule.owned_allocation_capacity_bytes();
    assert!(rule_backing > 0, "the authored contract name owns backing");
    measured.results[0].result.rule =
        InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(0));
    assert_eq!(
        before_rule - measured.owned_allocation_capacity_bytes(),
        rule_backing
    );
    drop(measured);
    let native_extent = stage.owned_allocation_capacity_bytes();
    assert!(native_extent > 0);
    let prefix = ReducedInvariantPrefix::new()
        .absorb_in_request(stage, execution, Some(&lease), None)
        .unwrap();
    assert_eq!(
        lease.charged_memory_bytes(),
        native_extent,
        "successful native evidence has no failure clones"
    );
    let whole_allowance = lease.policy().charged_memory_bytes().unwrap();
    assert!(
        lease.reserve_memory(whole_allowance).is_err(),
        "native output competes on the original ledger"
    );
    let blocker = lease
        .reserve_memory(whole_allowance - native_extent)
        .unwrap();
    let blocked = crate::validation::engine::stage_execution::execute_stage(
        &view,
        execution,
        &request,
        Some(&lease),
        InvariantStage::Custom {
            packet_offset: prefix.packet_count(),
        },
    );
    let Err(TransactionCommitError::Execution { denial, .. }) = blocked else {
        panic!("the real custom stage must refuse its framework admission");
    };
    assert!(
        matches!(
            denial.kind,
            crate::transactions::data::CommitExecutionDenialKind::Cause(
                crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
            )
        ),
        "{denial:?}"
    );
    assert_eq!(hooks.observed(), (0, 0));
    assert_eq!(lease.charged_memory_bytes(), whole_allowance);
    drop(blocker);
    assert_eq!(
        lease.charged_memory_bytes(),
        native_extent,
        "only the external blocker is released"
    );
    let custom = crate::validation::engine::stage_execution::execute_stage(
        &view,
        execution,
        &request,
        Some(&lease),
        InvariantStage::Custom {
            packet_offset: prefix.packet_count(),
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(hooks.observed(), (1, 1));
    assert_eq!(
        lease.charged_memory_bytes(),
        native_extent,
        "custom stage cannot release the native prefix"
    );
    let prefix = prefix
        .absorb_in_request(custom, execution, Some(&lease), None)
        .unwrap();
    let completed = prefix
        .finish_in_request(&request, execution, Some(&lease), None)
        .unwrap();
    assert_eq!(completed.result().results().len(), 2);
    assert_eq!(completed.result().decision_log().len(), 2);
    assert_eq!(
        completed
            .result()
            .metadata()
            .proof_boundary()
            .unwrap()
            .packet_count(),
        2
    );
    assert!(lease.charged_memory_bytes() > native_extent);
    assert!(lease.reserve_memory(whole_allowance).is_err());
    drop(completed);
    assert_eq!(lease.charged_memory_bytes(), 0);
    let funded = lease
        .reserve_memory(whole_allowance)
        .expect("same reservation fits after actual evidence drops");
    drop(funded);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn native_serial_omission_stays_usable_and_an_existing_accounted_ancestor_remains_authoritative() {
    let (fixture, hooks) = fixture(2);
    let plan = fixture.merged_plan(fixture.chain_batch(1));
    let request = || {
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &fixture.runtime,
            InvariantObservation::committed(fixture.runtime.storage_access().current_edition()),
            fixture.runtime.current_version_id(),
            Some(&plan),
            None,
        )
    };
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let serial = Arc::new(worth_execution::SerialRequest::from_policy(
        &policy,
        CancellationToken::new(),
        None,
    ));
    *hooks.native_probe.lock().unwrap() = Some(Arc::clone(&serial));
    assert!(serial.memory().is_none());
    let native = ExecutionRequest::serial(&serial);
    let result = InvariantEngine::new(&fixture.runtime)
        .execute(&native, request())
        .unwrap();
    assert_eq!(result.results().len(), 2);
    assert_eq!(hooks.observed(), (1, 1));
    assert_eq!(
        *hooks.native_accounting.lock().unwrap(),
        [false],
        "the original native callback does not acquire a meter"
    );
    let ancestor = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(8 * 1024 * 1024)),
        CancellationToken::new(),
        None,
    );
    let (result, report) = ExecutionRequest::serial(&ancestor)
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(4_000_000)),
            |_| InvariantEngine::new(&fixture.runtime).execute(&native, request()),
        )
        .unwrap();
    assert_eq!(result.unwrap().results().len(), 2);
    assert_eq!(hooks.observed(), (2, 2));
    assert_eq!(
        *hooks.native_accounting.lock().unwrap(),
        [false, true],
        "the same callback inherits the existing accounted ancestor"
    );
    assert!(report.has_accounting());
    assert!(
        report.charged_work() > 0,
        "the inherited reduction is actually metered"
    );
    assert_eq!(ancestor.memory().unwrap().charged_memory_bytes(), 0);
}
