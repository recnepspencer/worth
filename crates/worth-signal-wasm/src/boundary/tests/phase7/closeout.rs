use crate::boundary::tests::support::*;
use crate::boundary::types::SignalWorkerRuntime;
use crate::expression::model::IdentitySpec;
use crate::recipe::model::{RecipeSpec, SourceSpec, TransactionOp};
use crate::runtime::compute_callbacks;
use crate::runtime::worker_host::{
    RuntimeEnvelopeCallbackReattachment, WorkerCompatibilityCertificationScenario,
    WorkerObservationDeliveryAttachRequest, WorkerOutputDeliveryRequest,
    WorkerPortableGraphPublication, WorkerRuntimeShell,
};

#[test]
fn worker_runtime_boundary_exposes_phase7_closeout_certification() {
    let worker_runtime = SignalWorkerRuntime::new().unwrap();
    {
        let mut shell = worker_runtime.shell.borrow_mut();
        shell.publish_graph(portable_counter_publication()).unwrap();
        certify_same_runtime_restore(&mut shell);
        certify_checkpoint_retained_history(&mut shell);
        certify_import_export_callback_unavailability(&mut shell);
        shell
            .attach_observation_delivery(WorkerObservationDeliveryAttachRequest {
                signal_id: "doubleCounter".to_owned(),
            })
            .unwrap();
        certify_phase5_public_delivery_evidence(&mut shell);
    }

    let package = worker_runtime
        .certify_worker_phase7_closeout_for_test(portable_counter_compatibility_scenario())
        .unwrap();

    assert_eq!(
        package.certification_family,
        "workerPhase7CloseoutCertification"
    );
    assert_eq!(package.suite0_status, "Suite0FinalCloseoutCertified");
    assert!(package.milestone_closed);
    assert_eq!(package.final_closeout_pending_count, 0);
    assert_digest_shape(&package.proof_family_digest);
    assert_digest_shape(&package.performance_counter_catalog_digest);
    assert_digest_shape(&package.performance_complexity_contract_digest);
    assert_digest_shape(&package.performance_failure_mode_digest);
    assert_digest_shape(&package.bridge_allocation_posture_digest);
    assert_digest_shape(&package.acceptance_artifact_digest);
    assert_digest_shape(&package.worker_first_truth_digest);
    assert_digest_shape(&package.capability_parity_digest);
    assert_digest_shape(&package.boundary_performance_digest);
    assert_digest_shape(&package.certification_digest);
}

fn assert_digest_shape(digest: &str) {
    assert_eq!(digest.len(), 64);
    assert!(digest
        .chars()
        .all(|character| character.is_ascii_hexdigit()));
}

fn certify_phase5_public_delivery_evidence(shell: &mut WorkerRuntimeShell) {
    shell
        .apply_committed_transaction(set_counter_transaction(7.0))
        .unwrap();
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
    let feature_branch = shell
        .create_branch("boundary-phase7-restore".to_owned())
        .unwrap();
    shell.switch_branch(feature_branch.id.0).unwrap();
    shell
        .apply_committed_transaction(set_counter_transaction(11.0))
        .unwrap();
    let snapshot = shell.branch_snapshot(feature_branch.id.0).unwrap();
    shell.switch_branch(main_branch.branch_id).unwrap();
    shell
        .apply_committed_transaction(set_counter_transaction(3.0))
        .unwrap();
    shell
        .restore_branch_snapshot_with_capability_report(feature_branch.id.0, snapshot)
        .unwrap();
    shell.certify_worker_replay_restore_capability().unwrap();
}

fn certify_checkpoint_retained_history(shell: &mut WorkerRuntimeShell) {
    let main_branch = shell.branch_truth_envelope().unwrap();
    let branch = shell
        .create_branch("boundary-phase7-checkpoint".to_owned())
        .unwrap();
    shell.switch_branch(branch.id.0).unwrap();
    shell
        .apply_committed_transaction(set_counter_transaction(5.0))
        .unwrap();
    let checkpoint = shell.branch_snapshot(branch.id.0).unwrap();
    shell
        .apply_committed_transaction(set_counter_transaction(8.0))
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
            "boundaryPhase7HostedCallback".to_owned(),
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
            vec![reattachment("boundaryPhase7HostedCallback", 34.0)],
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
        publication: portable_counter_publication(),
        transaction_ops: set_counter_transaction(5.0),
        feature_transaction_ops: set_counter_transaction(7.0),
        main_transaction_ops: set_counter_transaction(3.0),
        observed_signal_id: "doubleCounter".to_owned(),
        async_signal_id: "doubleCounter".to_owned(),
        async_payload_contract_id: 42,
        async_payload_byte_len: 16,
        independent_region_recipe_ids: vec!["doubleCounter".to_owned()],
    }
}

fn portable_counter_publication() -> WorkerPortableGraphPublication {
    WorkerPortableGraphPublication {
        policy: RuntimePolicySpec::default(),
        sources: vec![SourceSpec {
            id: "counter".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        }],
        recipes: vec![RecipeSpec {
            id: "doubleCounter".to_owned(),
            reads: vec![RecipeReadSpec::LegacyId("counter".to_owned())],
            expr: Expr::Sum {
                args: vec![
                    Expr::Read {
                        id: "counter".to_owned(),
                    },
                    Expr::Read {
                        id: "counter".to_owned(),
                    },
                ],
            },
            when: None,
            identity: Some(IdentitySpec::Exact),
            produces_aspects: None,
        }],
        output_ids: vec!["doubleCounter".to_owned()],
    }
}

fn set_counter_transaction(value: f64) -> Vec<TransactionOp> {
    vec![TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(value),
        aspect: None,
        aspects: None,
    }]
}
