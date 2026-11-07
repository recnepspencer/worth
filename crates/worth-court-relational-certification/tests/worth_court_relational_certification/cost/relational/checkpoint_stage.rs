//! Checkpoint trust boundaries preserve exact data and fund cold verification.
//! Cold-region scopes may grow with the images being verified; ordinary commit
//! growth is guarded separately through the real SDK root-admission court.
use worth_execution::{
    CancellationSource, ExecutionAllocationDenialClass, ExecutionAllocationPolicy as Allocation,
    ExecutionRequest, ExecutionScopeSetupObservationForTest, MapKernelStop, SerialRequest,
    WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::RelationalNativeCheckpointCaptureDenial as Denial,
    identity::{KindId, PartitionId},
    runtime::RelationalRuntime,
    symbols::ClientKey,
    transactions::{
        CreateIntent, EntitySpec, MutationIntent, RelationalExecutionDenialCause as Cause,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::{
    publications::begin_main_transaction, AspectSchemaFixture,
};
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

#[test]
fn checkpoint_reconstruction_preserves_complete_identity_and_data() {
    for rows in [0, 8, 64] {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let before = runtime
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint(&execution, execution.allocation_policy())
            .unwrap();
        let after = runtime
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        // Export may not change any source identity, data, envelope or derived
        // commitment. This is the complete portable image, including private fields.
        assert_eq!(before, after, "source changed during export: rows={rows}");
        let mut restored = AspectSchemaFixture::default().build_runtime();
        restored
            .durability_recovery()
            .restore_native_checkpoint(&execution, &checkpoint)
            .unwrap();
        let reopened = restored
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        // Cold import mints fresh runtime affinity, while the actual arena
        // identities, values, MVCC history and selected commit remain exact.
        assert_eq!(
            before.partition_images, reopened.partition_images,
            "rows={rows}"
        );
        assert_eq!(before.coverage, reopened.coverage);
        assert_eq!(before.symbol_table, reopened.symbol_table);
        assert_eq!(before.aspect_contracts, reopened.aspect_contracts);
        assert_eq!(before.lineage, reopened.lineage);
        assert_eq!(before.index_definitions, reopened.index_definitions);
        drop((before, after, reopened, checkpoint, runtime, restored));
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    }
}

#[test]
fn checkpoint_reconstruction_scope_growth_is_linear_in_cold_input() {
    // The unchanged fixture grows current records and retained publication
    // metadata together. Cold admission may grow with that input, but must not
    // repeat whole-population scope preparation for every record.
    let capture = |rows| {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint(&execution, execution.allocation_policy())
            .unwrap();
        let counts = observer.counts_for_test();
        drop(checkpoint);
        counts
    };
    let (base, doubled) = (capture(32), capture(64));
    for (name, first, second) in [
        ("stage setups", base.stage_setups, doubled.stage_setups),
        (
            "limit derivations",
            base.limit_derivations,
            doubled.limit_derivations,
        ),
        ("admissions", base.admissions, doubled.admissions),
    ] {
        assert!(
            first > 0,
            "cold export must use its original request: {name}"
        );
        assert!(
            (second as u128) * 2 < (first as u128) * 5,
            "{name}: cold scope work outgrows cold input: {first}->{second}"
        );
    }
}

#[test]
fn checkpoint_refusals_preserve_request_custody_and_publication() {
    let _guard = crate::leased_preparation::ordinary_context::serial();
    for leased in [false, true] {
        assert!(memory_outcome(leased, 1 << 20, false).unwrap() > 0);
        assert!(matches!(
            memory_outcome(leased, 0, false),
            Err(Denial::Request(Cause::PolicyMemoryExhausted { .. })) | Err(Denial::Allocation(_))
        ));
        match (leased, memory_outcome(leased, 0, true)) {
            (false, Err(Denial::Request(Cause::Cancelled))) => {}
            (true, Err(Denial::Allocation(cause))) => assert_eq!(
                cause.classify(),
                ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Stopped(
                    MapKernelStop::Cancelled
                ))
            ),
            (_, outcome) => panic!("cancellation must outrank memory refusal: {outcome:?}"),
        }
    }
    let request = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(0)),
        ),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let runtime = runtime_with_rows(0);
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    assert!(matches!(
        runtime
            .durability_authority()
            .native_checkpoint(&execution, Allocation::SystemAllocation),
        Err(Denial::Request(Cause::WorkExhausted))
    ));
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
}

fn runtime_with_rows(rows: usize) -> RelationalRuntime {
    let request = serial_request();
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
    runtime
}

/// The checkpoint length or its refusal; every retained byte is released.
fn memory_outcome(leased: bool, memory: u64, cancelled: bool) -> Result<usize, Denial> {
    let source = CancellationSource::new();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(memory), None),
    );
    let serial = SerialRequest::from_policy(&policy, source.token(), None);
    let lease = leased.then(|| {
        crate::leased_preparation::ordinary_context::authority()
            .request_lease(worth_execution::LeaseRequest {
                policy,
                cancellation: source.token(),
                deadline: None,
            })
            .unwrap()
    });
    let request = match &lease {
        Some(lease) => ExecutionRequest::leased(lease),
        None => ExecutionRequest::serial(&serial),
    };
    let runtime = runtime_with_rows(0);
    if cancelled {
        source.cancel();
    }
    let outcome = runtime
        .durability_authority()
        .native_checkpoint(&request, request.allocation_policy())
        .map(|checkpoint| checkpoint.bytes().len());
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    if let Some(lease) = &lease {
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    outcome
}
