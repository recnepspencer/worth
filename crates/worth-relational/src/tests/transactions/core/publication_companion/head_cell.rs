//! Head selection and publication exclusion are Native authority, not caller observations.
use super::*;
use crate::mvcc::PublicationCompanionRegistrationStop;
use std::sync::mpsc;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[derive(Debug)]
struct UnindexedCompanion;
impl RelationalPublicationCompanion for UnindexedCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let cell = context.mint_selected_branch_cell(Arc::new(0_u64))?;
        let reserved = cell.reserve_preflight(context)?;
        context.seal_replacement(reserved, Arc::new(1_u64))
    }
}

fn budget() -> CompanionPreflightBudget {
    CompanionPreflightBudget {
        maximum_work_visits: 32,
        maximum_preparation_bytes: 8192,
    }
}

#[test]
fn head_cell_uses_the_new_head_after_an_unindexed_publication() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "head-cell-baseline");
    let old_handle = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let old = runtime
        .read_truth()
        .positioned_snapshot(&old_handle)
        .unwrap();
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(UnindexedCompanion), budget())
        .unwrap();
    let committed = create_entity_outcome(&runtime, "unindexed-publication");
    let head = runtime
        .read_truth()
        .positioned_snapshot(&committed.snapshot)
        .unwrap();
    assert_ne!(old.root_id(), head.root_id());
    let cell = registration
        .with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            Arc::new(9_u64),
            |cell| cell,
        )
        .unwrap();
    let minted = cell.read_image();
    assert_eq!(
        minted.root_id(),
        head.root_id(),
        "only Native's true head may mint a cell"
    );
    assert_eq!(minted.commit_id(), head.commit_id());
    assert_eq!(minted.position(), head.position());
    release_test_commit_snapshot(&runtime, &committed);
    runtime.snapshots().release_snapshot(&old_handle).unwrap();
}

#[derive(Debug)]
struct HeldPreparedCompanion {
    prepared: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}
impl RelationalPublicationCompanion for HeldPreparedCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let cell = context.mint_selected_branch_cell(Arc::new(0_u64))?;
        let reserved = cell.reserve_preflight(context)?;
        let effect = context.seal_replacement(reserved, Arc::new(1_u64))?;
        self.prepared.send(()).unwrap();
        self.release.lock().unwrap().recv().unwrap();
        Ok(effect)
    }
}

#[test]
fn head_cell_declines_a_prepared_runtime_publication_without_installing() {
    contention(false);
}
#[test]
fn head_cell_declines_an_independently_owned_publication_port_without_installing() {
    contention(true);
}

fn contention(independent_port: bool) {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "head-cell-contention-baseline");
    let old_handle = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let (prepared, observe_prepared) = mpsc::channel();
    let (release, observe_release) = mpsc::channel();
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(
            Arc::new(HeldPreparedCompanion {
                prepared,
                release: Mutex::new(observe_release),
            }),
            budget(),
        )
        .unwrap();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("head-cell-held-publication"),
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(transaction, AllocationPolicy::SystemAllocation)
        .unwrap();
    let port = runtime.publication_port();
    let drops = Arc::new(AtomicUsize::new(0));
    let initial = Arc::new(DropCounter(Arc::clone(&drops)));
    let weak = Arc::downgrade(&initial);
    let installed = AtomicUsize::new(0);
    std::thread::scope(|threads| {
        let publisher = threads.spawn(|| {
            if independent_port {
                let RelationalPublicationOutcome::Performed(performed) =
                    port.compare_and_publish(candidate)
                else {
                    panic!("held independent publication performs");
                };
                runtime.settle_performed_publication(performed).unwrap()
            } else {
                runtime.publish_prepared_candidate(candidate).unwrap()
            }
        });
        observe_prepared.recv().unwrap();
        let result = registration.with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            initial,
            |cell| {
                installed.fetch_add(1, Ordering::Relaxed);
                drop(cell);
            },
        );
        // Release before assertions: a fail-before result must not strand the publisher.
        release.send(()).unwrap();
        let committed = publisher.join().unwrap();
        assert_eq!(
            result,
            Err(PublicationCompanionRegistrationStop::HeadCellPublicationContended)
        );
        assert_eq!(
            installed.load(Ordering::Relaxed),
            0,
            "contention never calls install"
        );
        assert!(
            weak.upgrade().is_none(),
            "a declined initial image has no retained custody"
        );
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        release_test_commit_snapshot(&runtime, &committed);
    });
    runtime.snapshots().release_snapshot(&old_handle).unwrap();
}

#[test]
fn a_panicking_head_install_cannot_poison_source_publication() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "panic-install-baseline");
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(UnindexedCompanion), budget())
        .unwrap();
    let initial = Arc::new(DropCounter(Arc::new(AtomicUsize::new(0))));
    let dropped = Arc::clone(&initial.0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        registration
            .with_branch_cell_at_head(
                &runtime,
                &runtime.main_branch_identity(),
                initial,
                |_cell| -> () {
                    panic!("caller install failed");
                },
            )
            .unwrap();
    }));
    assert!(result.is_err(), "the caller's unwind continues");
    assert_eq!(
        dropped.load(Ordering::Relaxed),
        1,
        "failed install retains no initial payload"
    );
    let committed = create_entity_outcome(&runtime, "after-panicking-install");
    release_test_commit_snapshot(&runtime, &committed);
    let cell = registration
        .with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            Arc::new(9_u64),
            |cell| cell,
        )
        .expect("a later mint remains available");
    assert_eq!(
        cell.read_image().commit_id(),
        Some(committed.commit.commit_id)
    );
}
