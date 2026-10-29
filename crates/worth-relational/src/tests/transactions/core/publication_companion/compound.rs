//! Fixed two-cell Native cutover; these payloads do not model Query or World state.
use super::*;
use crate::identity::data::EntityId;
use crate::mvcc::{CompanionBranchImage, ReservedCompanionBranchCell};
use crate::runtime::PositionedRelationalSnapshot;
use crate::transactions::data::{EntityMutationIntent, UpdateEntityFieldsIntent};
use std::sync::Weak;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

enum Action {
    RetainSecond,
    RefuseSecond,
    Cancel(CancellationSource),
    Publish,
}

struct CompoundCompanion {
    marks: CompanionBranchCell<Vec<u64>>,
    managed: CompanionBranchCell<Vec<u64>>,
    action: Mutex<Option<Action>>,
    retained_second: Mutex<Option<ReservedCompanionBranchCell<Vec<u64>>>>,
    staged: Mutex<Option<[Weak<Vec<u64>>; 2]>>,
    second_stop: Mutex<Option<CompanionPreflightStop>>,
    sealed_pairs: AtomicUsize,
}

impl fmt::Debug for CompoundCompanion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CompoundCompanion")
    }
}

impl RelationalPublicationCompanion for CompoundCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let action = self.action.lock().unwrap().take().expect("armed receiver");
        if matches!(&action, Action::RetainSecond) {
            // Establish a genuine reservation from an actual earlier candidate.
            // This deliberate deferral is setup, not the second-seal refusal.
            *self.retained_second.lock().unwrap() = Some(self.managed.reserve_preflight(context)?);
            return Err(CompanionPreflightStop::TopologyPending);
        }
        let marks = self.marks.reserve_preflight(context)?;
        let managed = if matches!(&action, Action::RefuseSecond) {
            self.retained_second.lock().unwrap().take().unwrap()
        } else {
            self.managed.reserve_preflight(context)?
        };
        let next_marks = Arc::new(vec![marks.current().payload()[0] + 1]);
        let next_managed = Arc::new(vec![managed.current().payload()[0] + 10]);
        *self.staged.lock().unwrap() =
            Some([Arc::downgrade(&next_marks), Arc::downgrade(&next_managed)]);
        let effect =
            match context.seal_managed_source_replacement(marks, next_marks, managed, next_managed)
            {
                Ok(effect) => effect,
                Err(stop) => {
                    *self.second_stop.lock().unwrap() = Some(stop);
                    return Err(stop);
                }
            };
        self.sealed_pairs.fetch_add(1, Ordering::Relaxed);
        if let Action::Cancel(original) = action {
            original.cancel();
        }
        Ok(effect)
    }
}

fn prepare_update(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
    name: &str,
) -> crate::mvcc::PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(name).push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: name_field_patch(name),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}

fn assert_name(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
    expected: &str,
) {
    let mut read = test_owner_begin_transaction_for_main(runtime);
    let record = read.read_entity(execution, entity).unwrap();
    assert_eq!(
        record.base().and_then(read_entity_name).as_deref(),
        Some(expected)
    );
}

fn assert_image(
    image: &CompanionBranchImage<Vec<u64>>,
    source: &PositionedRelationalSnapshot,
    expected: &[u64],
) {
    assert_eq!(image.payload().as_slice(), expected);
    assert_eq!(image.root_id(), source.root_id());
    assert_eq!(image.commit_id(), source.commit_id());
    assert_eq!(image.position(), source.position());
}

#[test]
fn compound_managed_source_publication_is_atomic_and_releases_failed_staging() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "compound-before");
    let old_handle = snapshot_for_owner_branch(&runtime, &BranchId("main".into()));
    let old_source = runtime
        .read_truth()
        .positioned_snapshot(&old_handle)
        .unwrap();
    let before = test_owner_main_basis(&runtime).unwrap();
    let port = runtime.publication_companion_port();
    let pending = port.begin_required_registration().unwrap();
    let marks = pending
        .with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            Arc::new(vec![7]),
            |cell| cell,
        )
        .unwrap();
    let managed = pending
        .with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            Arc::new(vec![11]),
            |cell| cell,
        )
        .unwrap();
    let old_marks = marks.read_image();
    let old_managed = managed.read_image();
    assert_image(&old_marks, &old_source, &[7]);
    assert_image(&old_managed, &old_source, &[11]);
    let participant = Arc::new(CompoundCompanion {
        marks: marks.clone(),
        managed: managed.clone(),
        action: Mutex::new(Some(Action::RetainSecond)),
        retained_second: Mutex::new(None),
        staged: Mutex::new(None),
        second_stop: Mutex::new(None),
        sealed_pairs: AtomicUsize::new(0),
    });
    let registration = pending.activate(participant.clone()).unwrap();
    let setup = prepare_update(&runtime, execution, entity, "compound-after-refusal");
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(execution, setup),
        RelationalPublicationOutcome::Deferred(RelationalPublicationDeferred::CompanionPreflight(
            CompanionPreflightStop::TopologyPending
        ))
    ));
    assert!(participant.retained_second.lock().unwrap().is_some());
    assert!(participant.staged.lock().unwrap().is_none());
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        before.descriptor()
    );

    *participant.action.lock().unwrap() = Some(Action::RefuseSecond);
    let candidate = prepare_update(&runtime, execution, entity, "compound-after-refusal");
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(execution, candidate),
        RelationalPublicationOutcome::Deferred(RelationalPublicationDeferred::CompanionPreflight(
            CompanionPreflightStop::SelectedSourceMismatch
        ))
    ));
    assert_eq!(
        *participant.second_stop.lock().unwrap(),
        Some(CompanionPreflightStop::SelectedSourceMismatch)
    );
    assert_eq!(participant.sealed_pairs.load(Ordering::Relaxed), 0);
    assert!(participant.retained_second.lock().unwrap().is_none());
    let refused = participant
        .staged
        .lock()
        .unwrap()
        .take()
        .expect("both payloads were prepared");
    assert!(
        refused.iter().all(|weak| weak.upgrade().is_none()),
        "partial seal releases both Vec backings"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        before.descriptor()
    );
    assert_name(&runtime, execution, entity, "compound-before");
    assert_image(&marks.read_image(), &old_source, &[7]);
    assert_image(&managed.read_image(), &old_source, &[11]);

    let fresh = crate::test_execution_policy::serial_request();
    let funded = &ExecutionRequest::serial(&fresh);
    *participant.action.lock().unwrap() = Some(Action::Publish);
    let retry = prepare_update(&runtime, funded, entity, "compound-after-refusal");
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(funded, retry)
    else {
        panic!("fresh candidate reserves BOTH cells after second-seal refusal");
    };
    assert_eq!(participant.sealed_pairs.load(Ordering::Relaxed), 1);
    let commit_id = performed.canonical_commit().commit.commit_id;
    let position = performed.patch_position();
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let source = runtime
        .read_truth()
        .positioned_snapshot(&committed.snapshot)
        .unwrap();
    assert_ne!(source.root_id(), old_source.root_id());
    assert_eq!(source.commit_id(), Some(commit_id));
    assert_eq!(source.position(), Some(position));
    assert_image(&marks.read_image(), &source, &[8]);
    assert_image(&managed.read_image(), &source, &[21]);
    assert_name(&runtime, funded, entity, "compound-after-refusal");
    let first_backing = participant.staged.lock().unwrap().take().unwrap();
    assert!(first_backing.iter().all(|weak| weak.upgrade().is_some()));
    release_test_commit_snapshot(&runtime, &committed);

    let cancellation = CancellationSource::new();
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(64 * 1024 * 1024)),
        cancellation.token(),
        None,
    );
    let original_execution = &ExecutionRequest::serial(&original);
    let before_cancel = test_owner_main_basis(&runtime).unwrap();
    *participant.action.lock().unwrap() = Some(Action::Cancel(cancellation));
    let cancelled = prepare_update(
        &runtime,
        original_execution,
        entity,
        "compound-after-cancel",
    );
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(original_execution, cancelled),
        RelationalPublicationOutcome::RequestStopped(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    ));
    assert_eq!(
        participant.sealed_pairs.load(Ordering::Relaxed),
        2,
        "cancellation follows a complete pair"
    );
    let abandoned = participant.staged.lock().unwrap().take().unwrap();
    assert!(
        abandoned.iter().all(|weak| weak.upgrade().is_none()),
        "original stop releases both staged Vec backings"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        before_cancel.descriptor()
    );
    assert_name(&runtime, funded, entity, "compound-after-refusal");
    assert_image(&marks.read_image(), &source, &[8]);
    assert_image(&managed.read_image(), &source, &[21]);

    let fresh = crate::test_execution_policy::serial_request();
    let funded = &ExecutionRequest::serial(&fresh);
    *participant.action.lock().unwrap() = Some(Action::Publish);
    let retry = prepare_update(&runtime, funded, entity, "compound-after-cancel");
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(funded, retry)
    else {
        panic!("fresh request reserves BOTH cells after original cancellation");
    };
    assert_eq!(participant.sealed_pairs.load(Ordering::Relaxed), 3);
    let commit_id = performed.canonical_commit().commit.commit_id;
    let position = performed.patch_position();
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let final_source = runtime
        .read_truth()
        .positioned_snapshot(&committed.snapshot)
        .unwrap();
    assert_ne!(final_source.root_id(), source.root_id());
    assert_eq!(final_source.commit_id(), Some(commit_id));
    assert_eq!(final_source.position(), Some(position));
    assert_image(&marks.read_image(), &final_source, &[9]);
    assert_image(&managed.read_image(), &final_source, &[31]);
    assert_name(&runtime, funded, entity, "compound-after-cancel");
    assert_image(&old_marks, &old_source, &[7]);
    assert_image(&old_managed, &old_source, &[11]);
    assert!(
        first_backing.iter().all(|weak| weak.upgrade().is_none()),
        "settlement releases retired backing"
    );
    let final_backing = participant.staged.lock().unwrap().take().unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    runtime.snapshots().release_snapshot(&old_handle).unwrap();
    port.remove_required(&registration).unwrap();
    drop(registration);
    drop(participant);
    drop(marks);
    drop(managed);
    assert!(
        final_backing.iter().all(|weak| weak.upgrade().is_none()),
        "last cell owners release published backing"
    );
}
