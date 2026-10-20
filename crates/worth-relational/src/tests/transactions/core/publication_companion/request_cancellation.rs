use super::*;

struct CancellingCompanion {
    cell: CompanionBranchCell<u64>,
    original_cancellation: worth_execution::CancellationSource,
    calls: AtomicUsize,
    domain_refusal: bool,
}

impl fmt::Debug for CancellingCompanion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("CancellingCompanion").finish()
    }
}

impl RelationalPublicationCompanion for CancellingCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let reserved = self.cell.reserve_preflight(context)?;
        let effect = context.seal_replacement(reserved, Arc::new(9_u64))?;
        if self.calls.fetch_add(1, Ordering::Relaxed) == 0 {
            self.original_cancellation.cancel();
            if self.domain_refusal {
                return Err(CompanionPreflightStop::TopologyPending);
            }
        }
        Ok(effect)
    }
}

#[test]
fn original_request_cancellation_dominates_companion_result_and_preserves_retry() {
    for domain_refusal in [true, false] {
        let original_cancellation = worth_execution::CancellationSource::new();
        let original = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(Some(64 * 1024 * 1024)),
            original_cancellation.token(),
            None,
        );
        let execution = &worth_execution::ExecutionRequest::serial(&original);
        let runtime = runtime_with_test_schema();
        let anchor = create_entity(execution, &runtime, "original-cancellation-anchor");
        let before = test_owner_main_basis(&runtime).unwrap();
        let before_partition = before
            .inner
            .root
            .partition_state(anchor.partition_id)
            .unwrap();
        let before_record = before_partition.entity_arena.get(&anchor).unwrap();
        let before_payload = before_record.extra().authoritative_aspect_state.clone();
        let pending = runtime
            .publication_companion_port()
            .begin_required_registration()
            .unwrap();
        let cell = pending
            .with_branch_cell_at_head(
                &runtime,
                &runtime.main_branch_identity(),
                Arc::new(7_u64),
                |cell| cell,
            )
            .unwrap();
        let participant = Arc::new(CancellingCompanion {
            cell: cell.clone(),
            original_cancellation,
            calls: AtomicUsize::new(0),
            domain_refusal,
        });
        let _registration = pending
            .activate(
                participant.clone(),
                CompanionPreflightBudget {
                    maximum_work_visits: Some(64),
                    maximum_preparation_bytes: Some(8192),
                },
            )
            .unwrap();
        let candidate = prepare_create(&runtime, execution);
        let outcome = runtime
            .publication_port()
            .compare_and_publish(execution, candidate);
        assert_eq!(
            participant.calls.load(Ordering::Relaxed),
            1,
            "the registered native callback ran"
        );
        assert!(matches!(outcome,RelationalPublicationOutcome::RequestStopped(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        )),"the original request stop must dominate both domain refusal and a prepared successful companion: {outcome:?}");
        let unchanged = test_owner_main_basis(&runtime).unwrap();
        assert_eq!(unchanged.descriptor(), before.descriptor());
        let partition = unchanged
            .inner
            .root
            .partition_state(anchor.partition_id)
            .unwrap();
        let record = partition.entity_arena.get(&anchor).unwrap();
        assert_eq!(record.extra().authoritative_aspect_state, before_payload);
        assert_eq!(**cell.read_image().payload(), 7);

        let fresh = crate::test_execution_policy::serial_request();
        let funded_retry = &worth_execution::ExecutionRequest::serial(&fresh);
        let retry = prepare_create(&runtime, funded_retry);
        let RelationalPublicationOutcome::Performed(performed) = runtime
            .publication_port()
            .compare_and_publish(funded_retry, retry)
        else {
            panic!("a fresh funded request retries the unchanged native basis");
        };
        assert_eq!(participant.calls.load(Ordering::Relaxed), 2);
        assert_eq!(**cell.read_image().payload(), 9);
        let committed = runtime.settle_performed_publication(performed).unwrap();
        release_test_commit_snapshot(&runtime, &committed);
    }
}

fn prepare_create(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::mvcc::PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create("original-cancellation-selected-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}
