use crate::runtime::compute_callbacks;
use crate::runtime::tests::support::*;
use crate::runtime::tests::worker_runtime::fixtures::portable_counter_graph::{
    double_counter_observation_attach_request, portable_counter_publication, set_counter,
    worker_shell_with_counter_graph, worker_shell_with_observed_counter,
};
use crate::runtime::worker_host::{
    certify_worker_phase7_performance_contracts, certify_worker_phase7_product_guidance,
    certify_worker_phase7_test_requirements, certify_worker_unavailable_compatibility_artifact,
    RuntimeEnvelopeCallbackReattachment, WorkerCompatibilityCertificationScenario,
    WorkerObservationDeliveryDetachRequest, WorkerOutputDeliveryRequest,
    WorkerPhase7CloseoutCertificationPackage, WorkerPortableGraphPublication, WorkerRuntimeShell,
};

fn complete_phase7_closeout_shell() -> WorkerRuntimeShell {
    let mut shell = worker_shell_with_counter_graph();
    certify_same_runtime_restore(&mut shell);
    certify_checkpoint_retained_history(&mut shell);
    certify_import_export_callback_unavailability(&mut shell);
    shell
        .attach_observation_delivery(double_counter_observation_attach_request())
        .unwrap();
    certify_phase5_public_delivery_evidence(&mut shell);
    shell
}

fn certify_phase5_public_delivery_evidence(shell: &mut WorkerRuntimeShell) {
    shell.apply_committed_transaction(set_counter(7.0)).unwrap();
    shell.deliver_latest_observation().unwrap();
    shell
        .deliver_outputs(WorkerOutputDeliveryRequest {
            output_ids: vec!["doubleCounter".to_owned()],
        })
        .unwrap();
    shell.read_diagnostics_summary().unwrap();
}

fn certify_same_runtime_restore(shell: &mut WorkerRuntimeShell) {
    let main_branch = shell.branch_truth_envelope().unwrap();
    let feature_branch = shell.create_branch("phase7-restore".to_owned()).unwrap();
    shell.switch_branch(feature_branch.id.0).unwrap();
    shell
        .apply_committed_transaction(set_counter(11.0))
        .unwrap();
    let snapshot = shell.branch_snapshot(feature_branch.id.0).unwrap();
    shell.switch_branch(main_branch.branch_id).unwrap();
    shell.apply_committed_transaction(set_counter(3.0)).unwrap();
    shell
        .restore_branch_snapshot_with_capability_report(feature_branch.id.0, snapshot)
        .unwrap();
    shell.certify_worker_replay_restore_capability().unwrap();
}

fn certify_checkpoint_retained_history(shell: &mut WorkerRuntimeShell) {
    let main_branch = shell.branch_truth_envelope().unwrap();
    let branch = shell.create_branch("phase7-checkpoint".to_owned()).unwrap();
    shell.switch_branch(branch.id.0).unwrap();
    shell.apply_committed_transaction(set_counter(5.0)).unwrap();
    let checkpoint = shell.branch_snapshot(branch.id.0).unwrap();
    shell.apply_committed_transaction(set_counter(8.0)).unwrap();
    shell
        .apply_committed_transaction(set_counter(13.0))
        .unwrap();
    shell
        .record_worker_replay_checkpoint_retained_history(branch.id.0, checkpoint)
        .unwrap();
    shell
        .certify_worker_replay_checkpoint_retained_history()
        .unwrap();
    // Exact restore artifacts are exported from the root branch; the
    // checkpoint story ends back on it like the restore story does.
    shell.switch_branch(main_branch.branch_id).unwrap();
}

fn certify_import_export_callback_unavailability(shell: &mut WorkerRuntimeShell) {
    shell
        .define_main_thread_hosted_callback_for_test(
            "phase7HostedCallback".to_owned(),
            Box::new(|| hosted_callback_result(21.0)),
        )
        .unwrap();
    let envelope = shell.export_worker_runtime_envelope().unwrap();
    shell.certify_worker_callback_capability_export().unwrap();
    shell
        .admit_worker_runtime_envelope_import(envelope.clone())
        .unwrap();
    shell
        .admit_worker_runtime_envelope_import_with_callback_reattachments(
            envelope,
            vec![reattachment("phase7HostedCallback", 34.0)],
        )
        .unwrap();
    shell
        .certify_worker_import_export_callback_unavailability()
        .unwrap();
}

fn hosted_callback_result(
    value: f64,
) -> Result<
    compute_callbacks::ComputeCallbackInvocationResult,
    compute_callbacks::ComputeCallbackFailure,
> {
    Ok(compute_callbacks::ComputeCallbackInvocationResult {
        value: SignalValue::Number(value),
        captured_read_ids: vec!["counter".to_owned()],
        captured_host_capability_reads: Vec::new(),
        runtime_read_breadth: 1,
        return_serialization_breadth: 1,
    })
}

fn reattachment(callback_id: &str, value: f64) -> RuntimeEnvelopeCallbackReattachment {
    let token = compute_callbacks::register_native_compute_result(Box::new(move || {
        hosted_callback_result(value)
    }));
    let invocation = compute_callbacks::invoke_compute(token).unwrap();
    RuntimeEnvelopeCallbackReattachment {
        callback_id: callback_id.to_owned(),
        token,
        invocation,
    }
}

fn portable_counter_compatibility_scenario() -> WorkerCompatibilityCertificationScenario {
    WorkerCompatibilityCertificationScenario {
        publication: WorkerPortableGraphPublication {
            output_ids: vec!["doubleCounter".to_owned()],
            ..portable_counter_publication()
        },
        transaction_ops: set_counter(5.0),
        feature_transaction_ops: set_counter(7.0),
        main_transaction_ops: set_counter(3.0),
        observed_signal_id: "doubleCounter".to_owned(),
        async_signal_id: "doubleCounter".to_owned(),
        async_payload_contract_id: 42,
        async_payload_byte_len: 16,
        independent_region_recipe_ids: vec!["doubleCounter".to_owned()],
    }
}

#[test]
fn worker_phase7_closeout_certifies_suite0_final_closure() {
    let shell = complete_phase7_closeout_shell();
    let worker_unavailable = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();

    let package = shell
        .certify_worker_phase7_closeout(worker_unavailable)
        .unwrap();

    assert_eq!(
        package.certification_family,
        "workerPhase7CloseoutCertification"
    );
    assert_eq!(package.suite0_status, "Suite0FinalCloseoutCertified");
    assert!(package.milestone_closed);
    assert_eq!(package.required_proof_family_count, 13);
    assert_eq!(package.covered_proof_family_count, 13);
    assert_eq!(package.final_closeout_pending_count, 0);
    assert_digest_shape(&package.phase5_certification_digest);
    assert_digest_shape(&package.phase6_certification_digest);
    assert_digest_shape(&package.performance_contract_certification_digest);
    assert_digest_shape(&package.product_guidance_certification_digest);
    assert_digest_shape(&package.test_requirements_certification_digest);
    assert_digest_shape(&package.proof_family_digest);
    assert_digest_shape(&package.performance_counter_catalog_digest);
    assert_digest_shape(&package.performance_complexity_contract_digest);
    assert_digest_shape(&package.performance_failure_mode_digest);
    assert_digest_shape(&package.bridge_allocation_posture_digest);
    assert_digest_shape(&package.suite0_closeout_digest);
    assert_digest_shape(&package.certification_digest);
}

#[test]
fn worker_phase7_closeout_rejects_missing_phase5_evidence() {
    let mut shell = complete_phase7_closeout_shell();
    shell
        .detach_observation_delivery(WorkerObservationDeliveryDetachRequest {
            lifecycle_subscription_id: 1,
        })
        .unwrap();
    let worker_unavailable = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();

    let error = shell
        .certify_worker_phase7_closeout(worker_unavailable)
        .unwrap_err();

    assert!(error.message.contains("lifecycle evidence"));
}

#[test]
fn worker_phase7_closeout_rejects_missing_phase6_evidence() {
    let mut shell = worker_shell_with_observed_counter();
    certify_phase5_public_delivery_evidence(&mut shell);
    let worker_unavailable = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();

    let error = shell
        .certify_worker_phase7_closeout(worker_unavailable)
        .unwrap_err();

    assert!(error.message.contains("Phase 6"));
}

#[test]
fn worker_phase7_closeout_rejects_hidden_allocation_posture() {
    let shell = complete_phase7_closeout_shell();
    let phase5 = shell.certify_worker_phase5_closeout().unwrap();
    let worker_unavailable = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();
    let phase6 = shell
        .certify_worker_phase6_closeout(worker_unavailable)
        .unwrap();
    let mut performance = certify_worker_phase7_performance_contracts().unwrap();
    performance
        .bridge_allocation_posture
        .hidden_allocation_allowed = true;

    let error = WorkerPhase7CloseoutCertificationPackage::from_certified_phase7_evidence(
        phase5,
        phase6,
        performance,
        certify_worker_phase7_product_guidance().unwrap(),
        certify_worker_phase7_test_requirements().unwrap(),
        package_truth_digest(&shell),
    )
    .unwrap_err();

    assert!(error.message.contains("performance contract"));
}

#[test]
fn worker_phase7_closeout_rejects_pending_test_requirement_row() {
    let shell = complete_phase7_closeout_shell();
    let phase5 = shell.certify_worker_phase5_closeout().unwrap();
    let worker_unavailable = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();
    let phase6 = shell
        .certify_worker_phase6_closeout(worker_unavailable)
        .unwrap();
    let mut test_requirements = certify_worker_phase7_test_requirements().unwrap();
    test_requirements.proof_families[0].readiness = "CoveredPendingFinalCloseout";

    let error = WorkerPhase7CloseoutCertificationPackage::from_certified_phase7_evidence(
        phase5,
        phase6,
        certify_worker_phase7_performance_contracts().unwrap(),
        certify_worker_phase7_product_guidance().unwrap(),
        test_requirements,
        package_truth_digest(&shell),
    )
    .unwrap_err();

    assert!(error.message.contains("test requirement tracking"));
}

fn package_truth_digest(shell: &WorkerRuntimeShell) -> String {
    shell
        .certify_worker_phase5_closeout()
        .unwrap()
        .worker_first_truth_digest
}
