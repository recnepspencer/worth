use crate::runtime::worker_host::{WorkerBranchLifecycleTruthReport, WorkerRuntimeShell};

use crate::runtime::tests::support::*;
use crate::runtime::tests::worker_runtime::fixtures::portable_counter_graph::{
    define_portable_counter_graph, portable_counter_publication,
};

#[test]
fn worker_runtime_branch_restore_preserves_compatibility_truth() {
    let publication = portable_counter_publication();
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();
    let mut compatibility_runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    worker_shell.publish_graph(publication).unwrap();
    define_portable_counter_graph(&mut compatibility_runtime);

    let worker_main = worker_shell.branch_truth_envelope().unwrap();
    let compatibility_main = compatibility_runtime.current_branch();
    let worker_feature = worker_shell.create_branch("what-if".to_owned()).unwrap();
    let compatibility_feature = compatibility_runtime
        .create_branch("what-if".to_owned())
        .unwrap();

    worker_shell.switch_branch(worker_feature.id.0).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_feature.id.0)
        .unwrap();
    let feature_transaction = vec![TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(11.0),
        aspect: None,
        aspects: None,
    }];
    worker_shell
        .apply_committed_transaction(feature_transaction.clone())
        .unwrap();
    compatibility_runtime
        .apply_transaction(feature_transaction)
        .unwrap();
    let worker_feature_snapshot = worker_shell.branch_snapshot(worker_feature.id.0).unwrap();
    let compatibility_feature_snapshot = compatibility_runtime
        .branch_snapshot(compatibility_feature.id.0)
        .unwrap();

    worker_shell.switch_branch(worker_main.branch_id).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_main.id.0)
        .unwrap();
    let main_transaction = vec![TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(3.0),
        aspect: None,
        aspects: None,
    }];
    worker_shell
        .apply_committed_transaction(main_transaction.clone())
        .unwrap();
    compatibility_runtime
        .apply_transaction(main_transaction)
        .unwrap();

    let restored_worker_feature = worker_shell
        .restore_branch_snapshot(worker_feature.id.0, worker_feature_snapshot)
        .unwrap();
    compatibility_runtime
        .restore_branch_snapshot(compatibility_feature.id.0, compatibility_feature_snapshot)
        .unwrap();
    let compatibility_feature_digest = compatibility_runtime
        .branch_state_proof(compatibility_feature.id.0)
        .unwrap()
        .state_digest;

    let feature_report = WorkerBranchLifecycleTruthReport::compare(
        &restored_worker_feature,
        compatibility_feature_digest,
    );

    assert!(feature_report.branch_truth_matches);
    assert_eq!(feature_report.worker_envelope_family, "lifecycleControl");
    assert_eq!(restored_worker_feature.lifecycle_artifact, "branchTruth");
    worker_shell.switch_branch(worker_feature.id.0).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_feature.id.0)
        .unwrap();
    assert_eq!(
        worker_shell.read_value("doubleCounter").unwrap(),
        compatibility_runtime.read_value("doubleCounter").unwrap()
    );
}

#[test]
fn worker_runtime_branch_switch_preserves_isolated_materialization() {
    let publication = portable_counter_publication();
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();
    let mut compatibility_runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    worker_shell.publish_graph(publication).unwrap();
    define_portable_counter_graph(&mut compatibility_runtime);

    let worker_main = worker_shell.branch_truth_envelope().unwrap();
    let compatibility_main = compatibility_runtime.current_branch();
    let worker_feature = worker_shell.create_branch("feature".to_owned()).unwrap();
    let compatibility_feature = compatibility_runtime
        .create_branch("feature".to_owned())
        .unwrap();

    worker_shell.switch_branch(worker_feature.id.0).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_feature.id.0)
        .unwrap();
    let feature_transaction = vec![TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(17.0),
        aspect: None,
        aspects: None,
    }];
    worker_shell
        .apply_committed_transaction(feature_transaction.clone())
        .unwrap();
    compatibility_runtime
        .apply_transaction(feature_transaction)
        .unwrap();

    worker_shell.switch_branch(worker_main.branch_id).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_main.id.0)
        .unwrap();
    let main_transaction = vec![TransactionOp::Set {
        id: "counter".to_owned(),
        value: SignalValue::Number(5.0),
        aspect: None,
        aspects: None,
    }];
    worker_shell
        .apply_committed_transaction(main_transaction.clone())
        .unwrap();
    compatibility_runtime
        .apply_transaction(main_transaction)
        .unwrap();

    let worker_main_truth = worker_shell.branch_truth_envelope().unwrap();
    let compatibility_main_digest = compatibility_runtime
        .branch_state_proof(compatibility_main.id.0)
        .unwrap()
        .state_digest;
    let main_report =
        WorkerBranchLifecycleTruthReport::compare(&worker_main_truth, compatibility_main_digest);
    assert!(main_report.branch_truth_matches);

    let worker_feature_truth = worker_shell.switch_branch(worker_feature.id.0).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_feature.id.0)
        .unwrap();
    let compatibility_feature_digest = compatibility_runtime
        .branch_state_proof(compatibility_feature.id.0)
        .unwrap()
        .state_digest;
    let feature_report = WorkerBranchLifecycleTruthReport::compare(
        &worker_feature_truth,
        compatibility_feature_digest,
    );

    assert!(feature_report.branch_truth_matches);
    assert_eq!(
        worker_shell.read_value("doubleCounter").unwrap(),
        SignalValue::Number(34.0)
    );
    worker_shell.switch_branch(worker_main.branch_id).unwrap();
    assert_eq!(
        worker_shell.read_value("doubleCounter").unwrap(),
        SignalValue::Number(10.0)
    );
}

#[test]
fn worker_runtime_branch_restore_uses_branch_local_snapshot_identity() {
    let publication = portable_counter_publication();
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();
    let mut compatibility_runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    worker_shell.publish_graph(publication).unwrap();
    define_portable_counter_graph(&mut compatibility_runtime);

    let worker_main = worker_shell.current_branch();
    let compatibility_main = compatibility_runtime.current_branch();
    let worker_baseline_snapshot = worker_shell.export_worker_snapshot_envelope().unwrap();
    let compatibility_baseline_snapshot = compatibility_runtime.snapshot().unwrap();
    let worker_feature = worker_shell.create_branch("feature".to_owned()).unwrap();
    let compatibility_feature = compatibility_runtime
        .create_branch("feature".to_owned())
        .unwrap();

    worker_shell.switch_branch(worker_feature.id.0).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_feature.id.0)
        .unwrap();
    worker_shell
        .apply_committed_transaction(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(11.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    compatibility_runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(11.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let worker_feature_snapshot = worker_shell.branch_snapshot(worker_feature.id.0).unwrap();
    let compatibility_feature_snapshot = compatibility_runtime
        .branch_snapshot(compatibility_feature.id.0)
        .unwrap();

    // Restoring the main baseline from the feature branch reactivates main
    // on both deployments and leaves the feature branch's truth alone.
    worker_shell
        .restore_snapshot(worker_baseline_snapshot)
        .unwrap();
    compatibility_runtime
        .restore_snapshot(compatibility_baseline_snapshot)
        .unwrap();
    assert_eq!(worker_shell.current_branch().id, worker_main.id);
    assert_eq!(
        compatibility_runtime.current_branch().id,
        compatibility_main.id
    );
    assert_eq!(
        worker_shell.read_value("counter").unwrap(),
        compatibility_runtime.read_value("counter").unwrap()
    );
    assert_ne!(
        worker_shell.read_value("counter").unwrap(),
        SignalValue::Number(11.0),
        "main holds its baseline, not the feature edit"
    );

    worker_shell.switch_branch(worker_feature.id.0).unwrap();
    compatibility_runtime
        .switch_branch(compatibility_feature.id.0)
        .unwrap();
    assert_eq!(
        worker_shell.read_value("counter").unwrap(),
        SignalValue::Number(11.0),
        "the feature branch kept its edit across the main restore"
    );
    let restored_worker_feature = worker_shell
        .restore_branch_snapshot(worker_feature.id.0, worker_feature_snapshot)
        .unwrap();
    compatibility_runtime
        .restore_branch_snapshot(compatibility_feature.id.0, compatibility_feature_snapshot)
        .unwrap();

    let compatibility_feature_digest = compatibility_runtime
        .branch_state_proof(compatibility_feature.id.0)
        .unwrap()
        .state_digest;
    let feature_report = WorkerBranchLifecycleTruthReport::compare(
        &restored_worker_feature,
        compatibility_feature_digest,
    );

    assert!(feature_report.branch_truth_matches);
    assert_eq!(
        worker_shell.read_value("doubleCounter").unwrap(),
        compatibility_runtime.read_value("doubleCounter").unwrap()
    );
}
