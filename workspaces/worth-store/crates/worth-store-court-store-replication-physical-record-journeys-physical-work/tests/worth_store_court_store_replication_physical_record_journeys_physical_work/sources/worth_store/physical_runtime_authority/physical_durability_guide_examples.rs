mod mutation_example {
    use worth_proof::TransitionOutcome;
    use worth_store::physical_runtime::{
        AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalMutationHandle,
        PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
        PhysicalMutationPreparationSuccess, PhysicalMutationRequest, RecordAppendBatch,
        ServingPhysicalRuntime,
    };

    fn submit_platform_durable_mutation(
        runtime: &ServingPhysicalRuntime,
        batch: RecordAppendBatch,
        placement: AdmittedRecordPlacementPolicy,
        idempotency_material: [u8; 32],
    ) {
        let submission = runtime.record_submission();
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
                idempotency_material,
            ))
            .expect("the Store must still be accepting mutation identities");
        let deadline = PhysicalMutationDeadline::after_milliseconds(1_000)
            .expect("the deadline must be nonzero");
        let request = PhysicalMutationRequest::platform_durable(key, deadline);

        match submission
            .prepare_durable_append(batch, placement, request)
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                observe_mutation(prepared.start());
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(
                completed,
            )) => {
                let acknowledgment = completed.into_acknowledgment();
                let _persisted_records = acknowledgment.persisted_records();
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(
                fate,
            )) => {
                let _diagnostic = fate.diagnostic_evidence();
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
                let _diagnostic = fate.diagnostic_evidence();
            }
            TransitionOutcome::Denied(_)
            | TransitionOutcome::Deferred(_)
            | TransitionOutcome::Stale(_)
            | TransitionOutcome::RebindRequired(_)
            | TransitionOutcome::Failed(_) => {}
        }
    }

    fn observe_mutation(handle: PhysicalMutationHandle) {
        match handle.wait() {
            PhysicalMutationOutcome::Completed(completed) => {
                let acknowledgment = completed.into_acknowledgment();
                let _cost = acknowledgment.performance_evidence();
            }
            PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                let _diagnostic = fate.diagnostic_evidence();
            }
            PhysicalMutationOutcome::Indeterminate(fate) => {
                let _diagnostic = fate.diagnostic_evidence();
            }
        }
    }

    fn main() {}
}

mod checkpoint_example {
    use worth_proof::TransitionOutcome;
    use worth_store::physical_runtime::{
        PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
        PhysicalCheckpointRequest, ServingPhysicalRuntime,
    };

    fn publish_fuzzy_checkpoint(runtime: &ServingPhysicalRuntime, idempotency_key: [u8; 32]) {
        let deadline = PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .expect("the deadline must be nonzero");
        let request = PhysicalCheckpointRequest::fuzzy(
            PhysicalCheckpointIdempotencyKey::new(idempotency_key),
            deadline,
        );

        let TransitionOutcome::Success(handle) = runtime.checkpoints().start(request).into_raw()
        else {
            return;
        };
        match handle.wait() {
            PhysicalCheckpointOutcome::Completed(completed) => {
                let _bytes = completed.encoded_bytes();
                let _tail = completed.retained_wal_tail();
                let _reclamation = completed.wal_reclamation();
            }
            PhysicalCheckpointOutcome::ProvenNoEffect(no_effect) => {
                let _cause = no_effect.cause();
            }
            PhysicalCheckpointOutcome::Indeterminate(indeterminate) => {
                let _failure = indeterminate.failure();
            }
        }
    }

    fn main() {}
}

fn main() {}
