use crate::boundary::types::{SignalRuntime, SignalWorkerRuntime};
use crate::expression::model::{Expr, IdentitySpec, SignalValue};
use crate::recipe::model::{RecipeReadSpec, RecipeSpec, SourceSpec, TransactionOp};
use crate::runtime::policy::RuntimePolicySpec;
use crate::runtime::worker_host::{
    WorkerCompatibilityCertificationScenario, WorkerObservationDeliveryAttachRequest,
    WorkerOutputDeliveryRequest, WorkerPortableGraphPublication,
};

#[test]
fn worker_runtime_bootstrap_exposes_worker_first_deployment_posture() {
    let worker_runtime = SignalWorkerRuntime::new().unwrap();

    let bootstrap = worker_runtime.bootstrap_record_for_test().unwrap();

    assert_eq!(
        bootstrap.shell_lock.identity.deployment_posture,
        "workerFirst"
    );
    assert_eq!(
        bootstrap.shell_lock.identity.runtime_authority,
        "workerOwnedRuntime"
    );
    assert_eq!(
        bootstrap.shell_lock.graph_publication_admission,
        "portableDefinitionsOnly"
    );
    assert_eq!(bootstrap.boundary_surface, "workerFirstConstruction");
    assert_eq!(
        bootstrap.transport_posture,
        "inProcessBootstrapBeforeTransportBridge"
    );
    assert_eq!(bootstrap.host_capability_ingress, "deferredToHostBridge");
    assert_eq!(bootstrap.host_effect_egress, "deferredToHostBridge");
}

#[test]
fn worker_runtime_boundary_publishes_portable_graph_through_committed_envelope() {
    let worker_runtime = SignalWorkerRuntime::new().unwrap();

    let publication = WorkerPortableGraphPublication {
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
        output_ids: Vec::new(),
    };

    let summary = worker_runtime
        .publish_portable_graph_for_test(publication)
        .unwrap();
    let envelope = worker_runtime
        .apply_transaction_for_test(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(8.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();

    assert_eq!(summary.published_source_count, 1);
    assert_eq!(summary.published_recipe_count, 1);
    assert_eq!(summary.denied_callback_count, 0);
    assert_eq!(envelope.deployment_posture, "workerFirst");
    assert_eq!(envelope.envelope_family, "transactionResult");
    assert_eq!(envelope.run_summary.touched_nodes, 2);
}

#[test]
fn worker_runtime_boundary_exposes_phase5_closeout_certification() {
    let worker_runtime = SignalWorkerRuntime::new().unwrap();
    worker_runtime
        .publish_portable_graph_for_test(portable_counter_publication_with_output())
        .unwrap();
    worker_runtime
        .attach_observation_delivery_for_test(WorkerObservationDeliveryAttachRequest {
            signal_id: "doubleCounter".to_owned(),
        })
        .unwrap();
    worker_runtime
        .apply_transaction_for_test(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(7.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    worker_runtime
        .deliver_latest_observation_for_test()
        .unwrap();
    worker_runtime
        .deliver_outputs_for_test(WorkerOutputDeliveryRequest {
            output_ids: vec!["doubleCounter".to_owned()],
        })
        .unwrap();
    worker_runtime.read_diagnostics_summary_for_test().unwrap();

    let package = worker_runtime
        .certify_worker_phase5_closeout_for_test()
        .unwrap();

    assert_eq!(
        package.certification_family,
        "workerPhase5CloseoutCertification"
    );
    assert_eq!(package.covered_suite_count, 2);
    assert_eq!(package.observation_delivery_breadth, 1);
    assert_eq!(package.output_delivery_breadth, 1);
    assert_eq!(package.diagnostics_cold_reconstruction_count, 0);
    assert_eq!(package.active_lifecycle_subscription_count, 1);
}

#[test]
fn worker_runtime_boundary_exposes_replay_restore_capability_certification() {
    let worker_runtime = SignalWorkerRuntime::new().unwrap();
    worker_runtime
        .publish_portable_graph_for_test(portable_counter_publication_with_output())
        .unwrap();
    let feature_branch = worker_runtime
        .create_worker_branch_for_test("feature".to_owned())
        .unwrap();
    worker_runtime
        .switch_worker_branch_for_test(feature_branch.id.0)
        .unwrap();
    worker_runtime
        .apply_transaction_for_test(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(7.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let snapshot = worker_runtime
        .worker_branch_snapshot_for_test(feature_branch.id.0)
        .unwrap();
    worker_runtime
        .apply_transaction_for_test(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(9.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let report = worker_runtime
        .restore_branch_snapshot_with_capability_report_for_test(feature_branch.id.0, snapshot)
        .unwrap();

    let package = worker_runtime
        .certify_worker_replay_restore_capability_for_test()
        .unwrap();

    assert_eq!(report.restore_outcome, "SameRuntimeExactRestore");
    assert_eq!(
        package.certification_family,
        "workerReplayRestoreCapabilityCertification"
    );
    assert_eq!(
        package.exact_restore_artifact,
        "sameRuntimeBranchSnapshotStore"
    );
    assert_eq!(package.incompatibility_artifact, "none");
    assert_eq!(package.fallback_count, 0);
}

#[test]
fn worker_runtime_boundary_exposes_replay_checkpoint_retained_history_certification() {
    let worker_runtime = SignalWorkerRuntime::new().unwrap();
    worker_runtime
        .publish_portable_graph_for_test(portable_counter_publication_with_output())
        .unwrap();
    let feature_branch = worker_runtime
        .create_worker_branch_for_test("checkpoint-feature".to_owned())
        .unwrap();
    worker_runtime
        .switch_worker_branch_for_test(feature_branch.id.0)
        .unwrap();
    worker_runtime
        .apply_transaction_for_test(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(7.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let checkpoint = worker_runtime
        .worker_branch_snapshot_for_test(feature_branch.id.0)
        .unwrap();
    worker_runtime
        .apply_transaction_for_test(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(11.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let report = worker_runtime
        .record_worker_replay_checkpoint_retained_history_for_test(feature_branch.id.0, checkpoint)
        .unwrap();

    let package = worker_runtime
        .certify_worker_replay_checkpoint_retained_history_for_test()
        .unwrap();

    assert_eq!(report.envelope_family, "replayCheckpointRetainedHistory");
    assert_eq!(
        package.certification_family,
        "workerReplayCheckpointRetainedHistoryCertification"
    );
    assert_eq!(
        package.checkpoint_artifact,
        "workerBranchCheckpointSnapshot"
    );
    assert_eq!(
        package.retained_history_artifact,
        "checkpointPlusRetainedReplayHistory"
    );
    assert_eq!(
        package.exact_restore_artifact,
        "checkpointPlusRetainedReplayHistory"
    );
    assert_eq!(package.incompatibility_artifact, "none");
    assert_eq!(package.fallback_count, 0);
    assert!(package.retained_replay_frame_count > 0);
}

#[test]
fn diagnostics_boundary_exposes_worker_unavailable_compatibility_certification() {
    let runtime = SignalRuntime::new().unwrap();

    let package = runtime
        .diagnostics()
        .worker_unavailable_compatibility_certification_for_test(
            portable_counter_compatibility_scenario(),
        )
        .unwrap();

    assert_eq!(
        package.certification_family,
        "workerUnavailableCompatibilityCertification"
    );
    assert_eq!(package.worker_support_posture, "workerUnavailable");
    assert_eq!(
        package.selected_deployment_posture,
        "mainThreadCompatibility"
    );
    assert_eq!(
        package.incompatibility_artifact,
        "dedicatedWorkerUnavailable"
    );
    assert!(!package.hidden_fallback_allowed);
    assert_eq!(package.fallback_count, 0);
    assert_eq!(package.callback_declaration_count, 0);
    assert_eq!(package.denial_digest.len(), 64);
    assert!(package
        .denial_digest
        .chars()
        .all(|ch| ch.is_ascii_hexdigit()));
    assert_eq!(package.fallback_digest.len(), 64);
    assert!(package
        .fallback_digest
        .chars()
        .all(|ch| ch.is_ascii_hexdigit()));
}

fn portable_counter_publication_with_output() -> WorkerPortableGraphPublication {
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

fn portable_counter_compatibility_scenario() -> WorkerCompatibilityCertificationScenario {
    WorkerCompatibilityCertificationScenario {
        publication: portable_counter_publication_with_output(),
        transaction_ops: vec![set_counter_transaction(5.0)],
        feature_transaction_ops: vec![set_counter_transaction(7.0)],
        main_transaction_ops: vec![set_counter_transaction(3.0)],
        observed_signal_id: "doubleCounter".to_owned(),
        async_signal_id: "doubleCounter".to_owned(),
        async_payload_contract_id: 42,
        async_payload_byte_len: 16,
        independent_region_recipe_ids: vec!["doubleCounter".to_owned()],
    }
}

fn set_counter_transaction(value: f64) -> TransactionOp {
    TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(value),
        aspect: None,
        aspects: None,
    }
}
