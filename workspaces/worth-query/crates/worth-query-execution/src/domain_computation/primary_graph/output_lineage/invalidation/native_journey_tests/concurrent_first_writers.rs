//! Competing first publications retain the one lookup readers will use.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::{
    CompanionPreflightStop, PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    RelationalPublicationCompanion,
};

#[derive(Debug)]
struct FirstWriterGate {
    owner: Arc<SourceInvalidationOwner>,
    entered: AtomicUsize,
    first_ready: mpsc::Sender<()>,
    release_first: std::sync::Mutex<mpsc::Receiver<()>>,
    second_ready: mpsc::Sender<()>,
    first_done: std::sync::Mutex<mpsc::Receiver<()>>,
}
impl RelationalPublicationCompanion for FirstWriterGate {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let index = self.entered.fetch_add(1, Ordering::Relaxed);
        let result = self.owner.prepare(context);
        if index == 0 {
            assert!(result.is_ok(), "the first candidate must prepare");
            self.first_ready.send(()).unwrap();
            self.release_first.lock().unwrap().recv().unwrap();
        } else {
            self.second_ready.send(()).unwrap();
            // On the defective implementation the second owns another cell.
            // Keep its prepared effect alive, but let the first perform first.
            if result.is_ok() {
                self.first_done.lock().unwrap().recv().unwrap();
            }
        }
        result
    }
}

fn candidate(
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity: EntityId,
    field: AspectFieldLocator,
    value: &str,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    let basis = runtime
        .admit_branch_basis(&runtime.branch_identity(branch).unwrap())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("concurrent-first-writer").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        field,
                        AspectValue::String(InternedString::Raw(value.to_owned())),
                    )])),
                }),
            )),
        )
        .unwrap();
    runtime.prepare_branch_transaction(transaction).unwrap()
}

#[test]
fn two_first_candidates_cannot_orphan_the_performed_publications_cell() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = Arc::clone(&handle.source_owner.invalidation_owner);
    let reference = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(reference.entity(), reference.aspect(), reference.field())
        .unwrap()
        .clone();
    let (visible, expected) = handle.with_runtime_mut(|runtime| {
        // A real Native fork has no Query lookup cell before its first write.
        // No derived table, Native head or existing registration is erased.
        let branch = BranchId("concurrent-first-native-fork".to_owned());
        let (_, source) = runtime
            .observe_fork_source(runtime.main_branch_identity().branch_id())
            .unwrap();
        runtime.fork_branch(branch.clone(), source).unwrap();
        assert!(!owner.branches.lock().unwrap().cells.contains_key(&branch));
        let (first_ready, observe_first) = mpsc::channel();
        let (release_first, wait_release) = mpsc::channel();
        let (second_ready, observe_second) = mpsc::channel();
        let (first_done, wait_done) = mpsc::channel();
        let registration = runtime
            .publication_companion_port()
            .begin_required_registration()
            .unwrap()
            .activate(
                Arc::new(FirstWriterGate {
                    owner: Arc::clone(&owner),
                    entered: AtomicUsize::new(0),
                    first_ready,
                    release_first: std::sync::Mutex::new(wait_release),
                    second_ready,
                    first_done: std::sync::Mutex::new(wait_done),
                }),
                owner.resources.preflight_budget(),
            )
            .unwrap();
        let first = candidate(runtime, &branch, entity, status.clone(), "first");
        let second = candidate(runtime, &branch, entity, status, "second");
        let port = runtime.publication_port();
        let (performed, second) = std::thread::scope(|threads| {
            let first_thread = threads.spawn(|| {
                let published = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    port.compare_and_publish(first)
                }));
                first_done.send(()).unwrap();
                published.unwrap()
            });
            if observe_first
                .recv_timeout(std::time::Duration::from_secs(10))
                .is_err()
            {
                let _ = release_first.send(());
                if let RelationalPublicationOutcome::Performed(performed) =
                    first_thread.join().unwrap()
                {
                    let committed = runtime.settle_performed_publication(performed).unwrap();
                    release_test_commit_snapshot(runtime, &committed);
                }
                panic!("the first candidate did not reach its prepared gate");
            }
            let second_thread = threads.spawn(|| port.compare_and_publish(second));
            let overlapping = observe_second.recv_timeout(std::time::Duration::from_secs(10));
            release_first.send(()).unwrap();
            let results = (first_thread.join().unwrap(), second_thread.join().unwrap());
            assert!(
                overlapping.is_ok(),
                "both candidates reached preflight before the first performed"
            );
            results
        });
        let RelationalPublicationOutcome::Performed(performed) = performed else {
            panic!("the first candidate performs");
        };
        let committed = runtime.settle_performed_publication(performed).unwrap();
        assert!(
            !matches!(second, RelationalPublicationOutcome::Performed(_)),
            "a competitor with the old head cannot perform"
        );
        let branch = committed.snapshot.branch_id();
        let cell = owner
            .branches
            .lock()
            .unwrap()
            .cells
            .get(branch)
            .and_then(|slot| slot.admitted());
        let visible = cell.map(|cell| {
            let image = cell.read_image();
            (image.commit_id(), image.position())
        });
        let expected = (
            Some(committed.commit.commit_id),
            Some(committed.patch_position()),
        );
        release_test_commit_snapshot(runtime, &committed);
        drop(registration);
        (visible, expected)
    });
    assert_eq!(
        visible,
        Some(expected),
        "readers see the cell of the performed first candidate"
    );
}
