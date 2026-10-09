use super::*;
use crate::mvcc::{CompanionPublicationCompletion, CompanionPublicationCompletionObserver};

struct ObservedCompanion {
    cell: CompanionBranchCell<u64>,
    observer: Mutex<Option<CompanionPublicationCompletionObserver>>,
    retained_drops: Arc<AtomicUsize>,
    stop_after_prepare: bool,
}

impl fmt::Debug for ObservedCompanion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ObservedCompanion").finish()
    }
}

impl RelationalPublicationCompanion for ObservedCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let reserved = self.cell.reserve_preflight(context)?;
        let mut effect = context.seal_replacement(reserved, Arc::new(9_u64))?;
        let retained: Arc<dyn Send + Sync> =
            Arc::new(DropCounter(Arc::clone(&self.retained_drops)));
        let observer = effect.attach_completion_observer(context, retained)?;
        *self.observer.lock().unwrap() = Some(observer);
        if self.stop_after_prepare {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(effect)
    }
}

#[test]
fn actual_companion_cutover_resolves_only_its_prepared_observer() {
    for stop_after_prepare in [true, false] {
        let runtime = runtime_with_test_schema();
        create_entity(&runtime, "observer-anchor");
        let before = test_owner_main_basis(&runtime).expect("main branch has owner basis");
        let port = runtime.publication_companion_port();
        let pending = port.begin_required_registration().unwrap();
        let cell = pending
            .with_branch_cell_at_head(
                &runtime,
                &runtime.main_branch_identity(),
                Arc::new(7_u64),
                |cell| cell,
            )
            .unwrap();
        let retained_drops = Arc::new(AtomicUsize::new(0));
        let participant = Arc::new(ObservedCompanion {
            cell: cell.clone(),
            observer: Mutex::new(None),
            retained_drops: Arc::clone(&retained_drops),
            stop_after_prepare,
        });
        let registration = pending
            .activate(
                participant.clone(),
                CompanionPreflightBudget {
                    maximum_work_visits: 64,
                    maximum_preparation_bytes: 8_192,
                },
            )
            .unwrap();
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                batch_create("observer-selected-write"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let candidate = runtime
            .prepare_branch_transaction(
                transaction,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let outcome = runtime.publication_port().compare_and_publish(candidate);
        let observer = participant.observer.lock().unwrap().take().unwrap();
        if stop_after_prepare {
            assert!(matches!(
                outcome,
                RelationalPublicationOutcome::Deferred(
                    RelationalPublicationDeferred::CompanionPreflight(
                        CompanionPreflightStop::TopologyPending
                    )
                )
            ));
            assert_eq!(observer.state(), CompanionPublicationCompletion::Aborted);
            assert_eq!(
                test_owner_main_basis(&runtime).unwrap().descriptor(),
                before.descriptor()
            );
            assert_eq!(**cell.read_image().payload(), 7);
        } else {
            let RelationalPublicationOutcome::Performed(performed) = outcome else {
                panic!("the prepared companion and native commit perform together");
            };
            assert_eq!(observer.state(), CompanionPublicationCompletion::Installed);
            assert_eq!(**cell.read_image().payload(), 9);
            let committed = runtime
                .settle_performed_publication(performed)
                .expect("direct publication settles");
            release_test_commit_snapshot(&runtime, &committed);
        }
        port.remove_required(&registration).unwrap();
        drop(registration);
        drop(participant);
        assert_eq!(retained_drops.load(Ordering::Relaxed), 0);
        drop(observer);
        assert_eq!(retained_drops.load(Ordering::Relaxed), 1);
    }
}
