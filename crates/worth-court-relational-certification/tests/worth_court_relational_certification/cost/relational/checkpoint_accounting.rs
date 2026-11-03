//! Frozen original-accounting outcomes from the unoptimized checkpoint path.
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy as Allocation, ExecutionRequest,
    ExecutionWorkCeiling, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::RelationalNativeCheckpointCaptureDenial,
    identity::{KindId, PartitionId},
    symbols::ClientKey,
    transactions::{
        CreateIntent, EntitySpec, MutationIntent, RelationalExecutionDenialCause, WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::{
    publications::begin_main_transaction, AspectSchemaFixture,
};

#[test]
fn checkpoint_original_work_matches_unfixed_literal_totals() {
    // Measured on the unfixed path. These are not derived from encoded lengths
    // or a second implementation of its fragment charging algorithm.
    for (rows, expected_work) in [(0, 2_376), (8, 175_890), (64, 1_342_165)] {
        let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = AspectSchemaFixture::default().build_runtime();
        for ordinal in 0..rows {
            let mut transaction = begin_main_transaction(&runtime);
            transaction
                .push_batch(
                    WorkerIntentBatch::new("retained-publication").push(MutationIntent::Create(
                        CreateIntent::Entity(EntitySpec {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: ClientKey::raw(format!("record-{ordinal}")),
                            fields: Default::default(),
                        }),
                    )),
                    Allocation::SystemAllocation,
                )
                .unwrap();
            transaction.commit(&execution, &runtime).unwrap();
        }
        let retained_before = request.memory().unwrap().charged_memory_bytes();
        let (checkpoint, report) = ExecutionWorkCeiling::new(None)
            .run_serial(&request, || {
                runtime
                    .durability_authority()
                    .native_checkpoint(&execution, Allocation::SystemAllocation)
                    .unwrap()
            })
            .unwrap();
        assert_eq!(report.charged_work(), expected_work, "rows={rows}");
        drop(checkpoint);
        assert_eq!(
            request.memory().unwrap().charged_memory_bytes(),
            retained_before
        );
    }
}

#[test]
fn empty_checkpoint_original_work_ceilings_keep_the_unfixed_outcomes() {
    for (ceiling, succeeds, consults) in [
        (784, true, 5_758),
        (783, true, 5_758),
        (1, false, 16),
        (0, false, 6),
    ] {
        let request = SerialRequest::from_policy(
            &ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(ceiling)),
            ),
            CancellationSource::new().token(),
            None,
        );
        let execution = ExecutionRequest::serial(&request);
        let (runtime, _) = crate::content::transaction::fixture();
        let before = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0;
        let observer =
            worth_execution::RequestConsultObservationForTest::install_on_this_thread_for_test(
                |_| {},
            );
        let result = runtime
            .durability_authority()
            .native_checkpoint(&execution, Allocation::SystemAllocation);
        // Frozen from the old production serializer. The two emissions of a
        // latched work stop include its mandatory trailing zero checkpoint.
        assert_eq!(observer.count_for_test(), consults, "ceiling={ceiling}");
        let expected_origins: &[(usize, usize)] = match ceiling {
            0 => &[(4, 4), (5, 4), (6, 6)],
            1 => &[(14, 14), (15, 14), (16, 16)],
            _ => &[],
        };
        assert_eq!(observer.stop_origins_for_test(), expected_origins);
        drop(observer);
        if succeeds {
            let checkpoint = result.expect("the unfixed path accepted this work ceiling");
            assert_eq!(checkpoint.bytes().len(), 1_568);
            drop(checkpoint);
        } else {
            assert!(
                matches!(
                    result,
                    Err(RelationalNativeCheckpointCaptureDenial::Request(
                        RelationalExecutionDenialCause::WorkExhausted,
                    ))
                ),
                "ceiling={ceiling}: {result:?}",
            );
        }
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
        assert_eq!(
            runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0,
            before
        );
    }
}
