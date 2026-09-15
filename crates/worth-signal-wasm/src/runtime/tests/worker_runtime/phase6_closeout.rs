use crate::runtime::tests::support::*;
use crate::runtime::tests::worker_runtime::fixtures::portable_counter_graph::{
    portable_counter_publication, set_counter, worker_shell_with_counter_graph,
};
use crate::runtime::worker_host::{
    certify_worker_unavailable_compatibility_artifact, RuntimeEnvelopeCallbackReattachment,
    WorkerCompatibilityCertificationScenario, WorkerPortableGraphPublication, WorkerRuntimeShell,
};

fn complete_phase6_closeout_evidence() -> WorkerRuntimeShell {
    let mut shell = worker_shell_with_counter_graph();
    certify_same_runtime_restore(&mut shell);
    certify_checkpoint_retained_history(&mut shell);
    certify_import_export_callback_unavailability(&mut shell);
    shell
}

fn certify_same_runtime_restore(shell: &mut WorkerRuntimeShell) {
    let main_branch = shell.branch_truth_envelope().unwrap();
    let feature_branch = shell
        .create_branch("phase6-restore-feature".to_owned())
        .unwrap();
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
    let branch = shell
        .create_branch("phase6-checkpoint-feature".to_owned())
        .unwrap();
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
            "phase6HostedCallback".to_owned(),
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
            vec![reattachment("phase6HostedCallback", 34.0)],
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

#[test]
fn worker_phase6_closeout_certifies_historical_replay_import_and_compatibility_artifacts() {
    let shell = complete_phase6_closeout_evidence();
    let no_worker = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();

    let package = shell.certify_worker_phase6_closeout(no_worker).unwrap();

    assert_eq!(
        package.certification_family,
        "workerPhase6CloseoutCertification"
    );
    assert_eq!(
        package.phase_closeout_mode,
        "ReplayRestoreImportExportWorkerUnavailableParity"
    );
    assert_eq!(package.covered_suite_count, 4);
    assert_eq!(package.covered_phase6_artifact_count, 4);
    assert_eq!(
        package.replay_restore_exact_artifact,
        "sameRuntimeBranchSnapshotStore"
    );
    assert_eq!(
        package.checkpoint_retained_history_artifact,
        "checkpointPlusRetainedReplayHistory"
    );
    assert_eq!(
        package.import_export_unavailability_artifact,
        "computeCallbackUnavailableForPortableExport"
    );
    assert_eq!(
        package.worker_unavailable_incompatibility_artifact,
        "dedicatedWorkerUnavailable"
    );
    assert_eq!(package.fallback_count, 0);
    assert_eq!(package.exported_callback_count, 1);
    assert_eq!(package.unavailable_callback_count, 1);
    assert_eq!(package.reattached_callback_count, 1);
    assert_digest_shape(&package.replay_restore_certification_digest);
    assert_digest_shape(&package.checkpoint_retained_history_certification_digest);
    assert_digest_shape(&package.import_export_unavailability_certification_digest);
    assert_digest_shape(&package.worker_unavailable_compatibility_certification_digest);
    assert_digest_shape(&package.capability_parity_digest);
    assert_digest_shape(&package.phase6_artifact_digest);
    assert_digest_shape(&package.certification_digest);
}

#[test]
fn worker_phase6_closeout_rejects_missing_checkpoint_certification() {
    let mut shell = worker_shell_with_counter_graph();
    certify_same_runtime_restore(&mut shell);
    let no_worker = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();

    let error = shell.certify_worker_phase6_closeout(no_worker).unwrap_err();

    assert!(error.message.contains("checkpoint retained-history"));
}

#[test]
fn worker_phase6_closeout_rejects_worker_unavailable_hidden_fallback() {
    let shell = complete_phase6_closeout_evidence();
    let mut no_worker = certify_worker_unavailable_compatibility_artifact(
        portable_counter_compatibility_scenario(),
    )
    .unwrap();
    no_worker.fallback_count = 1;

    let error = shell.certify_worker_phase6_closeout(no_worker).unwrap_err();

    assert!(error
        .message
        .contains("explicit worker-unavailable compatibility"));
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
