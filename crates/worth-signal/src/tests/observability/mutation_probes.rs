use crate::data::telemetry::InvalidationPerformedCounter;
use crate::facade::{
    DependencyEdge, EvaluationRequestMode, SignalGraph, SignalInvalidationExecutionReceipt,
    SignalObservationRequest, SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::support::{version_ab, ASPECT_A};

fn assert_gate_shape(source: &str, gate: &str, gate_count: usize, effect: &str) {
    assert_eq!(
        source.matches(gate).count(),
        gate_count,
        "the named owner must retain every gated call site"
    );
    assert!(
        source.find(gate).expect("owner must retain its named gate")
            < source.find(effect).expect("owner must retain its effect"),
        "the first gate must precede the first effect"
    );
}

fn assert_timer_is_constructed_only_after_optional_gate(source: &str) {
    let gate = source
        .find("captures_observation_surface(")
        .expect("timer owner must check the optional surface");
    let timer = source
        .find("then(RuntimeInstant::now)")
        .expect("timer owner must construct its timer through the gated branch");
    assert!(
        gate < timer,
        "the optional-surface gate must precede timer construction"
    );
    assert_eq!(
        source.matches("then(RuntimeInstant::now)").count(),
        1,
        "the owner must not retain an unconditional timer construction"
    );
}

#[test]
fn structural_telemetry_owner_gates_are_present_before_their_writes() {
    let mutation = include_str!("../../data/telemetry_mutation.rs");
    assert!(mutation.contains("hard gate"));
    assert!(!mutation.contains("scratch copy"));
    assert!(!mutation.contains("Suppressed"));
    let runtime = include_str!("../../data/graph/diagnostics_access/runtime.rs");
    assert!(runtime.contains("return None"));
    assert!(runtime.contains("RuntimeTelemetryMutation::active"));

    let epoch = include_str!("../../data/graph/lifecycle/epoch.rs");
    assert_gate_shape(
        epoch,
        "self.telemetry_mut()",
        1,
        "telemetry.storage.gc_epoch_count += 1",
    );
    assert!(epoch.contains("telemetry.storage.gc_epoch_nanos += elapsed_nanos"));
    assert_timer_is_constructed_only_after_optional_gate(epoch);

    let snapshots = include_str!("../../data/graph/storage/entries/snapshots.rs");
    assert_timer_is_constructed_only_after_optional_gate(snapshots);

    let scratch = include_str!("../../data/graph/runtime/graph/scratch_lease.rs");
    assert_gate_shape(
        scratch,
        "self.telemetry_mut()",
        1,
        "scratch_reentry_error_count += 1",
    );

    for (execution_owner, gate, gate_count) in [
        (include_str!("../../logic/transaction/runtime/execution/runtime_execution/orchestration.rs"), "captures_observation_surface(", 1),
        (include_str!("../../logic/transaction/runtime/execution/runtime_execution/effect_execution.rs"), "captures_observation_surface(", 1),
        (include_str!("../../logic/transaction/runtime/execution/transaction_evaluation/orchestration.rs"), "with_telemetry(", 1),
        (include_str!("../../logic/transaction/runtime/execution/transaction_evaluation/effect_execution.rs"), "with_telemetry(", 1),
        (include_str!("../../logic/transaction/runtime/execution/transaction_keyed.rs"), "with_telemetry(", 5),
    ] {
        assert_gate_shape(
            execution_owner,
            gate,
            gate_count,
            "absorb_execution_report_telemetry("
        );
    }
}

fn method_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("owner must retain {signature}"));
    let body = &source[start..];
    let end = body
        .find("\n    }\n")
        .or_else(|| body.find("\n    }\r\n"))
        .expect("method body must have a closing brace");
    &body[..end]
}

fn prepared_runtime() -> (SignalRuntime<(), (), (), (), ()>, crate::facade::NodeId) {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(
        SignalRuntimePolicy::operational()
            .with_explanation_retention(crate::diagnostics::policy::ArtifactRetentionPolicy::Retain)
            .with_provenance_retention(crate::diagnostics::policy::ArtifactRetentionPolicy::Retain)
            .with_observation_activation(
                worth_foundational::ObservationActivationProfile::OnDemand,
            ),
    );
    let source = runtime.graph_mut().node().build();
    let target = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .set_dependencies(target, [DependencyEdge::new(source, ASPECT_A)])
        .unwrap();
    let bootstrap = runtime
        .graph_mut()
        .build_evaluation_plan(&[target], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    runtime
        .graph_mut()
        .execute_prepared_plan(&bootstrap, &(), &|context| {
            Ok(context.finish(version_ab(1, 0)))
        })
        .unwrap();
    crate::facade::mark_dirty(runtime.graph_mut(), target, ASPECT_A).unwrap();
    (runtime, target)
}

fn execute_observed(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    target: crate::facade::NodeId,
    request: SignalObservationRequest,
    revision: u64,
) -> SignalInvalidationExecutionReceipt {
    let (_, receipt) = runtime
        .observe_execution(request, |runtime| {
            let plan = runtime
                .graph_mut()
                .build_evaluation_plan(&[target], EvaluationRequestMode::ForceOnDemand)?;
            crate::logic::planner::execute_prepared_plan(
                &mut runtime.graph_mut(),
                &plan,
                &(),
                &|context: &mut crate::logic::context::EvaluationContext<'_, ()>| {
                    Ok(context.finish(version_ab(revision, 0)))
                },
            )?;
            Ok(())
        })
        .unwrap();
    receipt
}

#[test]
fn representative_counter_gate_deletion_probe_is_source_and_runtime_bound() {
    let source = include_str!("../../data/graph/runtime/graph/performed_counter_state.rs");
    assert_gate_shape(
        source,
        "captures(SignalObservationSurface::PerformedCounters)",
        3,
        "fetch_add",
    );
    assert_gate_shape(
        method_body(source, "pub(crate) fn set("),
        "captures(SignalObservationSurface::PerformedCounters)",
        1,
        ".store(",
    );

    let (mut runtime, target) = prepared_runtime();
    let receipt = execute_observed(
        &mut runtime,
        target,
        SignalObservationRequest::counters(),
        2,
    );
    assert!(
        receipt
            .realized_counters()
            .value(InvalidationPerformedCounter::NodesEvaluated)
            > 0
    );

    let (mut idle, target) = prepared_runtime();
    let plan = idle
        .graph_mut()
        .build_evaluation_plan(&[target], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    idle.graph_mut()
        .execute_prepared_plan(&plan, &(), &|context| Ok(context.finish(version_ab(3, 0))))
        .unwrap();
    assert!(InvalidationPerformedCounter::ALL
        .into_iter()
        .all(|counter| idle
            .graph()
            .invalidation_performed_counters()
            .value(counter)
            == 0));
}

#[test]
fn lineage_gate_relocation_after_construction_probe_stays_before_recording() {
    let source = include_str!("../../diagnostics/runtime/recorder/artifacts.rs");
    assert_gate_shape(
        source,
        "captures_observation_surface(",
        2,
        "record_evaluation_lineage",
    );
    let lineage_transition_start = source
        .find("pub(crate) fn stamp_trace_summary_and_record_lineage_transition_from_image(")
        .expect("lineage transition owner must remain present");
    let lineage_transition_end = source
        .find("pub(crate) fn record_invalidation_lineage(")
        .expect("invalidation lineage owner must remain present");
    assert_gate_shape(
        &source[lineage_transition_start..lineage_transition_end],
        "captures_observation_surface(",
        1,
        "allocate_lineage_sequence",
    );
    let (mut runtime, target) = prepared_runtime();
    execute_observed(&mut runtime, target, SignalObservationRequest::lineage(), 2);
    assert!(!runtime.graph().observe().lineage_records().is_empty());
}

#[test]
fn provenance_gate_bypass_probe_stays_at_the_retention_owner() {
    let source = include_str!("../../data/graph/diagnostics_access/artifacts/retained.rs");
    assert_gate_shape(
        source,
        "retains_provenance_facts()",
        2,
        "record_provenance_fact",
    );
    assert_gate_shape(
        method_body(source, "pub(crate) fn record_operational_diagnostic_facts("),
        "captures_observation_surface(",
        1,
        "compact_explanation_from_runtime_projection",
    );
    let (mut runtime, target) = prepared_runtime();
    execute_observed(&mut runtime, target, SignalObservationRequest::facts(), 2);
    assert!(runtime.graph().observe().provenance_fact(target).is_some());

    let (mut unobserved, target) = prepared_runtime();
    let plan = unobserved
        .graph_mut()
        .build_evaluation_plan(&[target], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    unobserved
        .graph_mut()
        .execute_prepared_plan(&plan, &(), &|context| Ok(context.finish(version_ab(3, 0))))
        .unwrap();
    assert!(unobserved
        .graph()
        .observe()
        .provenance_fact(target)
        .is_none());
}

#[test]
fn tier_to_strategy_recoupling_probe_uses_the_objective_axis() {
    use worth_foundational::ExecutionObjectiveProfile;

    let development = SignalRuntimePolicy::development()
        .with_execution_objective(ExecutionObjectiveProfile::Throughput);
    let operational = SignalRuntimePolicy::operational()
        .with_execution_objective(ExecutionObjectiveProfile::Throughput);
    assert_eq!(
        development.default_execution_strategy(),
        operational.default_execution_strategy()
    );
    assert_eq!(
        development.default_maintenance_strategy(),
        operational.default_maintenance_strategy()
    );
    let admission = include_str!("../../logic/planner/precompute/admission.rs");
    assert!(admission.contains("installed_policy.execution_objective()"));
    assert!(
        !admission.contains("DiagnosticsTier") && !admission.contains("installed_policy.tier()"),
        "planner parallel admission must not recouple strategy to DiagnosticsTier or installed tier()"
    );
    assert!(admission.contains("AdmittedThroughput"));
    assert!(!admission.contains("AdmittedOperational"));
    assert!(!admission.contains("AdmittedDevelopment"));
    assert!(!admission.contains("AdmittedForensic"));
}

#[test]
fn stable_identity_omission_probe_keeps_identity_outside_optional_lineage_gate() {
    let source = include_str!("../../diagnostics/runtime/recorder/artifacts.rs");
    let identity_write = source
        .find("stamp_runtime_artifact_lineage_and_execution")
        .expect("stable identity must be stamped by the artifact owner");
    let optional_gate = source
        .find("DescriptiveLineage")
        .expect("optional lineage gate must remain explicit");
    assert!(identity_write < optional_gate);
}

#[test]
fn post_hoc_receipt_minting_probe_requires_an_active_session() {
    let mut owner: SignalRuntime<(), (), (), (), ()> =
        SignalRuntime::operational(SignalGraph::new());
    let mut foreign: SignalRuntime<(), (), (), (), ()> =
        SignalRuntime::operational(SignalGraph::new());
    let session = owner
        .begin_observation_session(crate::facade::SignalObservationRequest::counters())
        .unwrap();
    let foreign_session = foreign
        .begin_observation_session(crate::facade::SignalObservationRequest::counters())
        .unwrap();
    assert!(foreign.finish_observation_session(&session).is_err());
    assert!(foreign.cancel_observation_session(&foreign_session).is_ok());
    owner.cancel_observation_session(&session).unwrap();

    let (mut unobserved, target) = prepared_runtime();
    let plan = unobserved
        .graph_mut()
        .build_evaluation_plan(&[target], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    unobserved
        .graph_mut()
        .execute_prepared_plan(&plan, &(), &|context| Ok(context.finish(version_ab(2, 0))))
        .unwrap();
    let session = unobserved
        .begin_observation_session(SignalObservationRequest::counters())
        .unwrap();
    assert!(unobserved.finish_observation_session(&session).is_err());
}

#[test]
fn restore_evidence_fabrication_probe_keeps_readmission_quarantine_in_the_owner() {
    let source = include_str!("../../logic/invalidation/causality/revalidation.rs");
    assert_eq!(
        source
            .matches("ensure_cause_readmission_complete()?")
            .count(),
        1
    );
    assert!(source.matches("cause_readmission_required").count() >= 2);

    let (mut runtime, target) = prepared_runtime();
    let snapshot = runtime.graph_mut().capture_snapshot();
    let restored =
        SignalGraph::restore_from_checkpoint_authority(&snapshot.checkpoint_image.authority)
            .expect("checkpoint authority must reconstruct operational truth");
    assert!(restored.get_entry(target).is_ok());
    assert!(!restored.node_dirty_aspects(target).unwrap().is_empty());
    assert!(restored.observe().provenance_fact(target).is_none());
    assert!(restored.observe().explanation_fact(target).is_none());
    assert!(restored.observe().lineage_records().is_empty());
    assert!(matches!(
        restored.node_invalidation_input(target),
        Ok(crate::data::proof::invalidation::revalidation::NodeInvalidationInput::Resolved(_))
    ));
}

#[test]
fn included_excluded_work_misclassification_probe_requires_counter_surface() {
    let source = include_str!("../../data/proof/invalidation/foundational_receipt.rs");
    assert_eq!(source.matches("ObservationSurfaceUnavailable").count(), 2);
    assert_eq!(source.matches("PerformedCounters").count(), 1);

    let (mut counters, target) = prepared_runtime();
    let counter_receipt = execute_observed(
        &mut counters,
        target,
        SignalObservationRequest::counters(),
        2,
    );
    assert!(counter_receipt
        .request()
        .includes(crate::facade::SignalObservationSurface::PerformedCounters));
    assert!(!counter_receipt
        .request()
        .includes(crate::facade::SignalObservationSurface::PerformedWork));
    assert!(
        counter_receipt
            .realized_counters()
            .value(InvalidationPerformedCounter::NodesEvaluated)
            > 0
    );

    let (mut work, target) = prepared_runtime();
    let work_receipt = execute_observed(&mut work, target, SignalObservationRequest::work(), 2);
    assert!(work_receipt
        .request()
        .includes(crate::facade::SignalObservationSurface::PerformedWork));
    assert!(!work_receipt
        .request()
        .includes(crate::facade::SignalObservationSurface::PerformedCounters));
    assert!(InvalidationPerformedCounter::ALL
        .into_iter()
        .all(|counter| work_receipt.realized_counters().value(counter) == 0));
    assert!(work_receipt.retains_executed_target(work.graph().runtime_instance_id(), target));
}
