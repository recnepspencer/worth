//! A stage may not compose a different field on an earlier stage's record.
use super::*;
struct WriteStatus<'a> {
    world: &'a World,
    target: EntityId,
    conflict: RefCell<Option<(RecordRef, usize, usize)>>,
}
impl RelationalAuthoredCoupledProducerAssembly for WriteStatus<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        let outcome = candidate.append_authored_stage(
            execution,
            self.world.carrier,
            &RelationalContentAttachments::default(),
            vec![
                WorkerIntentBatch::new("stage-status").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: self.target,
                        fields: string_aspect_field_patch([(
                            aspect_key("status"),
                            field_key("status"),
                            "changed",
                        )]),
                    }),
                )),
            ],
        );
        if let Err(cause) = &outcome
            && let Some((record, earlier, later)) = cause.authored_stage_conflict()
        {
            *self.conflict.borrow_mut() = Some((record.clone(), earlier, later));
        }
        outcome.map(|()| ControlFlow::Continue(()))
    }
}
#[test]
fn different_fields_on_one_authored_record_refuse_with_exact_writer_positions() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new_with_status(execution);
    let before = world.basis();
    let before_rows = cold::facts(&world.runtime, execution, &[world.source, world.carrier]);
    let parts = WriteStatus {
        world: &world,
        target: world.source,
        conflict: RefCell::new(None),
    };
    let error = world
        .prepare_stages(execution, &[world.carrier], &parts)
        .unwrap_err();
    assert!(
        matches!(error, TransactionCommitError::Conflict { ref error, .. }
        if matches!(error.class, ConflictClass::ConflictingIntent { .. })),
        "{error:?}"
    );
    assert_eq!(
        *parts.conflict.borrow(),
        Some((RecordRef::Entity(world.source), 0, 1))
    );
    assert_eq!(world.basis(), before);
    assert_eq!(
        cold::facts(&world.runtime, execution, &[world.source, world.carrier]),
        before_rows
    );
}
#[test]
fn the_same_field_write_on_a_disjoint_authored_record_commits() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new_with_status(execution);
    let before = world.basis();
    let parts = WriteStatus {
        world: &world,
        target: world.carrier,
        conflict: RefCell::new(None),
    };
    let proposal = world
        .prepare_stages(execution, &[world.carrier], &parts)
        .unwrap();
    let result = world.publish(execution, proposal);
    assert_eq!(*parts.conflict.borrow(), None);
    assert_ne!(world.basis(), before);
    assert_eq!(changed_entities(&result).len(), 2);
    release_test_commit_snapshot(&world.runtime, &result);
}
