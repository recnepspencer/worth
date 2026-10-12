use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui_query_binding::{
    UiProjectionConsumptionBudget, UiProjectionConsumptionLimits, UiProjectionFactStopKind,
    UiProjectionObservation, UiProjectionUnavailableKind, UiQueryObservationReportingProjection,
    UiScalarProjectionBindingAdmission,
};

use super::support::{
    assert_current, assert_stop, assert_unavailable, assert_zero_native_work, completion_batch,
    consume_batch, initial_fact, ScalarLifecycleWorld,
};

const SHARED_VALUE: &str = "Authority Ready";

#[test]
fn exact_operational_world_admits_one_indexed_native_value() {
    let (mut world, completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31350, 0), SHARED_VALUE);
    let pending = initial_fact(&mut world);
    let current = world.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );

    assert_current(&current, SHARED_VALUE);
    assert_eq!(current.work().native_key_declaration_checks(), 1);
    assert_eq!(current.work().native_key_indexed_slot_lookups(), 2);
    assert_eq!(current.work().native_indexed_accesses(), 1);
    assert_eq!(current.work().native_key_scan_work(), 0);
    assert_eq!(current.work().native_access_scan_work(), 0);
    assert!(current.retained_predecessor().is_none());
}

#[test]
fn equal_printable_native_keys_require_the_exact_operational_world() {
    let report =
        worth_ui_query_binding::certification::certify_scalar_native_authority_attack(SHARED_VALUE);

    assert_eq!(report.owner_key(), report.foreign_key());
    assert_eq!(report.owner_value(), SHARED_VALUE);
    assert_eq!(report.owner_counters().indexed_accesses, 1);
    assert_eq!(report.owner_counters().refinement_checks, 1);
    assert_eq!(
        report.foreign_denial(),
        worth_query::facade::installed::operation::WorthQueryNativeAccessDenialKind::RuntimeMismatch
    );
    assert_eq!(report.foreign_counters().authority_checks, 2);
    assert_eq!(report.foreign_counters().indexed_accesses, 0);
    assert_eq!(report.foreign_counters().refinement_checks, 0);
    assert_eq!(report.foreign_counters().fact_scans, 0);
    assert_eq!(report.foreign_counters().row_scans, 0);
}

#[test]
fn equal_printable_foreign_world_stops_before_operational_work() {
    let (mut local, local_completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31351, 0), SHARED_VALUE);
    let local_pending = initial_fact(&mut local);
    let (mut foreign, foreign_completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31351, 0), SHARED_VALUE);
    let _foreign_pending = initial_fact(&mut foreign);

    assert_equal_printable_controls(&local, &foreign);
    let foreign_batch = completion_batch(&mut foreign, foreign_completion);
    let stopped = consume_batch(
        &mut local,
        foreign_batch,
        local_pending,
        UiProjectionConsumptionBudget::platform_pulse(),
    );

    assert_stop(&stopped, UiProjectionFactStopKind::WrongWorld);
    assert_zero_native_work(stopped.work());
    assert!(stopped.retained_predecessor().is_some());
    let (_, predecessor) = stopped.into_fact_and_predecessor();
    let local_batch = completion_batch(&mut local, local_completion);
    let current = consume_batch(
        &mut local,
        local_batch,
        predecessor.expect("foreign denial returns the local predecessor"),
        UiProjectionConsumptionBudget::platform_pulse(),
    );
    assert_current(&current, SHARED_VALUE);
    assert_eq!(current.work().native_indexed_accesses(), 1);
}

#[test]
fn stale_installation_generation_stops_before_native_access() {
    let (mut world, completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31352, 0), SHARED_VALUE);
    let pending = initial_fact(&mut world);
    let batch = completion_batch(&mut world, completion);
    worth_query::facade::consumer_kit::advance_test_workspace_domain_installation_generation(
        &mut world.workspace,
    );
    let stopped = consume_batch(
        &mut world,
        batch,
        pending,
        UiProjectionConsumptionBudget::platform_pulse(),
    );

    assert_stop(&stopped, UiProjectionFactStopKind::StaleBindingGeneration);
    assert_zero_native_work(stopped.work());
    assert!(stopped.retained_predecessor().is_some());
}

#[test]
fn unsupported_remasked_and_basis_drift_never_open_native_access() {
    assert_unsupported();
    assert_remasked();
    assert_basis_drift();
}

#[test]
fn bounded_access_can_stop_after_one_exact_value_without_retaining_it() {
    let (mut world, completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31355, 0), SHARED_VALUE);
    let pending = initial_fact(&mut world);
    let batch = completion_batch(&mut world, completion);
    let budget =
        UiProjectionConsumptionBudget::bounded(UiProjectionConsumptionLimits::new(1, 1, 1))
            .expect("one-byte QP05 budget is valid");
    let stopped = consume_batch(&mut world, batch, pending, budget);

    assert_stop(&stopped, UiProjectionFactStopKind::BudgetExceeded);
    assert_eq!(stopped.work().native_indexed_accesses(), 1);
    assert_eq!(stopped.work().native_access_scan_work(), 0);
    assert!(stopped.retained_predecessor().is_some());
}

#[test]
fn reporting_projections_remain_observations_not_operational_inputs() {
    let (mut world, completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31356, 0), SHARED_VALUE);
    let pending = initial_fact(&mut world);
    let current = world.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    assert_eq!(current.work().native_indexed_accesses(), 1);
    let (fact, _) = current.into_fact_and_predecessor();
    let observation = UiProjectionObservation::Scalar(fact.into_observation());
    let reporting = UiQueryObservationReportingProjection::from_observation(&observation);

    assert!(!reporting.query_world().is_empty());
    assert!(!reporting.binding().is_empty());
    assert!(!reporting.source_generation().is_empty());
    assert!(!reporting.result_generation().is_empty());
}

fn assert_equal_printable_controls(local: &ScalarLifecycleWorld, foreign: &ScalarLifecycleWorld) {
    assert_eq!(
        local.binding.view_identity(),
        foreign.binding.view_identity()
    );
    assert_eq!(
        local.binding.requirement().selected_field(),
        foreign.binding.requirement().selected_field()
    );
    assert_eq!(
        local.request.request_handle().generation(),
        foreign.request.request_handle().generation()
    );
    assert_eq!(
        local.request.lowered().declaration_identity(),
        foreign.request.lowered().declaration_identity()
    );
}

fn assert_unsupported() {
    match super::super::projection_lifecycle::support::unsupported_admission() {
        UiScalarProjectionBindingAdmission::Unavailable(receipt) => {
            assert_eq!(receipt.kind(), UiProjectionUnavailableKind::Unsupported)
        }
        other => panic!("reporting-only support must remain unavailable, got {other:?}"),
    }
}

fn assert_remasked() {
    let (mut world, completion) =
        ScalarLifecycleWorld::remasked(NodeId::new(31353, 0), SHARED_VALUE);
    let initial = world.initial();
    assert_unavailable(initial.fact(), UiProjectionUnavailableKind::Remasked);
    assert_zero_native_work(initial.work());
    let predecessor = initial.into_fact_and_predecessor().0;
    let remasked = world.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(predecessor),
    );
    assert_unavailable(remasked.fact(), UiProjectionUnavailableKind::Remasked);
    assert_zero_native_work(remasked.work());
}

fn assert_basis_drift() {
    let (mut world, completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31354, 0), SHARED_VALUE);
    let pending = initial_fact(&mut world);
    let current = world.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    let (current, _) = current.into_fact_and_predecessor();
    let revalidation = world
        .bridge
        .revalidate_async_request(
            &world.request,
            super::super::projection_lifecycle::async_fixture::authoritative_async_basis(
                "commit-authority-drift",
                "snapshot-authority-drift",
            ),
        )
        .expect("QP05 revalidation lineage");
    let revalidating = world.advance(
        BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(revalidation),
        Some(current),
    );
    let (revalidating, _) = revalidating.into_fact_and_predecessor();
    let late = worth_runtime_bridge::certification_for_test::observe_late_async_completion_for_test(
        &world.bridge,
        &world.request,
    );
    let drift = world.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(late),
        Some(revalidating),
    );

    assert_unavailable(drift.fact(), UiProjectionUnavailableKind::BasisDrift);
    assert_zero_native_work(drift.work());
    assert!(drift.retained_predecessor().is_some());
}
