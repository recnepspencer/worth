use super::*;
use crate::branch::basis_admission_tests::installed_history;
use crate::config::data::RelationalRuntimeConfig;
use crate::runtime::state::subsystems::{
    history::HistorySubsystem,
    services::instrumentation::RuntimeInstrumentation,
    storage::{PartitionEditionCopyLane, StorageSubsystem},
};
use crate::storage::overlay::small_relation_partition::relation_partition;
use crate::validation::engine::test_view::invariant_view_with_custom_rules;

#[test]
fn checked_custom_evaluation_observes_cancellation_during_structural_views() {
    for leased in [false, true] {
        let cancellation = CancellationSource::new();
        let target = Arc::new(OnceLock::new());
        let visits = Arc::new(AtomicUsize::new(0));
        let partition = PartitionId::main();
        let (state, entities, _) = relation_partition(partition, 1, &[]);
        let storage = StorageSubsystem::default();
        let copy_counters = std::sync::Mutex::default();
        storage.install_owned(
            &copy_counters,
            PartitionEditionCopyLane::Ordinary,
            [(partition, state)],
        );
        let edition = storage.acquire();
        target.set(entities[0]).unwrap();
        let history: HistorySubsystem = installed_history(1);
        history.install_recovered_sequence_floor(2, 2);
        let head = history.current_version_id();
        let instrumentation = RuntimeInstrumentation::default();
        let mut config = RelationalRuntimeConfig::default();
        config.schema.registry = super::preparation::checked_structural_registry();
        let view = invariant_view_with_custom_rules(
            &instrumentation,
            config,
            head,
            vec![CustomInvariantRegistration::new(CheckedCounterRule {
                maximum_work_units: None,
                steps: 1_000,
                visits: Arc::clone(&visits),
                preparation_visits: Arc::new(AtomicUsize::new(0)),
                cancellation: Some(cancellation.clone()),
                preparation_steps: 0,
                preparation_scratch_bytes: 0,
                target: Some(Arc::clone(&target)),
            })
            .unwrap()],
        );
        let lease = checked_lease(8 * 1024 * 1024, 10_000, cancellation.token());
        let request = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed(edition),
            head,
            None,
            None,
        );
        let serial =
            crate::execution::test_policy::serial_request().with_cancellation(cancellation.token());
        let original = if leased {
            worth_execution::ExecutionRequest::leased(&lease)
        } else {
            worth_execution::ExecutionRequest::serial(&serial)
        };
        let result = InvariantEngine::from_view(&view).execute(&original, request);
        assert_eq!(history.current_version_id(), head);
        let visited = visits.load(Ordering::SeqCst);
        assert!(
            visited > 0 && visited < 1_000,
            "the real evaluator must run before interruption"
        );
        assert!(matches!(
            result,
            Err(
                crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
                    denial: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                            crate::execution::RelationalExecutionDenialCause::Cancelled
                        ),
                        ..
                    },
                    ..
                }
            )
        ));
    }
}
