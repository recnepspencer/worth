//! One checkpoint capture is one stage of its original request.
//!
//! The control owner documents the rate: one work unit per byte produced by
//! each serializer pass (sizing and emission are each charged), per payload
//! byte hashed and per byte copied; zero-work boundaries are free. Every
//! oracle below is derived from encoded byte counts read independently of the
//! charging code: the checkpoint length, its reported sections, the journal
//! artifact payloads and a MessagePack walk of the embedded envelopes.
use worth_execution::{
    CancellationSource, ExecutionAllocationDenialClass, ExecutionAllocationPolicy as Allocation,
    ExecutionRequest, ExecutionScopeSetupCountsForTest as Setup,
    ExecutionScopeSetupObservationForTest, ExecutionWorkCeiling, MapKernelStop, SerialRequest,
    WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::{NativeCheckpointSectionBytes, RelationalNativeCheckpointCaptureDenial as Denial},
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

/// `WORTH-REL-CHK20\n`: the prefix is emitted but not sized.
const CHECKPOINT_MAGIC_BYTES: u64 = 16;
/// Journal artifact header: magic 8, version 2, position 8, commit 8,
/// digest 32, payload length 8.
const ARTIFACT_HEADER_BYTES: usize = 66;

#[test]
fn checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte() {
    // Compare the invoice to actual encoded bytes. Digest bytes have variable
    // MessagePack widths, so a new commitment encoding must not require a
    // replacement table of frozen byte or work totals.
    for rows in [0, 8, 64] {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let journal: usize = runtime
            .durability()
            .canonical_commit_artifacts(execution.allocation_policy())
            .map(|artifact| artifact.unwrap().as_bytes().len() - ARTIFACT_HEADER_BYTES)
            .sum();
        let (checkpoint, report) = ExecutionWorkCeiling::new(None)
            .run_serial(&request, || {
                runtime
                    .durability_authority()
                    .native_checkpoint(&execution, Allocation::SystemAllocation)
                    .unwrap()
            })
            .unwrap();
        let bytes = checkpoint.bytes().len();
        let NativeCheckpointSectionBytes {
            total,
            envelopes,
            branch_roots,
            branch_cells,
            partition_mirror,
            derived_indexes,
            framing_and_metadata,
        } = checkpoint.captured_sections().unwrap();
        let classified =
            envelopes + branch_roots + branch_cells + partition_mirror + derived_indexes;
        assert_eq!(total, bytes);
        assert_eq!(classified + framing_and_metadata, bytes);
        let embedded: usize = embedded_envelope_lengths(checkpoint.bytes()).iter().sum();
        let derived = 2 * bytes as u64 - CHECKPOINT_MAGIC_BYTES
            + classified as u64
            + journal as u64
            + 2 * embedded as u64;
        assert_eq!(report.charged_work(), derived, "rows={rows}");
        drop(checkpoint);
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    }
}

#[test]
fn checkpoint_capture_sets_up_exactly_one_stage_whatever_the_row_count() {
    for rows in [0, 8, 64] {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint(&execution, Allocation::SystemAllocation)
            .unwrap();
        let expected = Setup {
            stage_setups: 1,
            limit_derivations: 1,
            admissions: 1,
        };
        assert_eq!(observer.counts_for_test(), expected, "rows={rows}");
        drop(checkpoint);
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

/// Independent MessagePack walk: the byte length of each embedded envelope.
fn embedded_envelope_lengths(checkpoint: &[u8]) -> Vec<usize> {
    fn int(bytes: &[u8], at: &mut usize, width: usize) -> usize {
        let value = bytes[*at..*at + width]
            .iter()
            .fold(0_usize, |sum, byte| (sum << 8) | usize::from(*byte));
        *at += width;
        value
    }
    fn length(bytes: &[u8], at: &mut usize) -> (u8, usize) {
        let tag = bytes[*at];
        *at += 1;
        match tag {
            0x80..=0x8f => (b'm', usize::from(tag & 0x0f)),
            0x90..=0x9f => (b'a', usize::from(tag & 0x0f)),
            0xa0..=0xbf => (b's', usize::from(tag & 0x1f)),
            0xc4 => (b'b', int(bytes, at, 1)),
            0xc5 => (b'b', int(bytes, at, 2)),
            0xc6 => (b'b', int(bytes, at, 4)),
            0xd9 => (b's', int(bytes, at, 1)),
            0xda => (b's', int(bytes, at, 2)),
            0xdb => (b's', int(bytes, at, 4)),
            0xdc => (b'a', int(bytes, at, 2)),
            0xdd => (b'a', int(bytes, at, 4)),
            0xde => (b'm', int(bytes, at, 2)),
            0xdf => (b'm', int(bytes, at, 4)),
            0xc7 => (b'b', int(bytes, at, 1) + 1),
            0xc8 => (b'b', int(bytes, at, 2) + 1),
            0xc9 => (b'b', int(bytes, at, 4) + 1),
            0x00..=0x7f | 0xe0..=0xff | 0xc0 | 0xc2 | 0xc3 => (b'b', 0),
            0xcc | 0xd0 => (b'b', 1),
            0xcd | 0xd1 => (b'b', 2),
            0xca | 0xce | 0xd2 => (b'b', 4),
            0xcb | 0xcf | 0xd3 => (b'b', 8),
            0xd4..=0xd8 => (b'b', 1 + (1 << (tag - 0xd4))),
            tag => panic!("unexpected MessagePack tag {tag:#x}"),
        }
    }
    fn skip(bytes: &[u8], at: &mut usize) {
        match length(bytes, at) {
            (b'm', pairs) => (0..pairs * 2).for_each(|_| skip(bytes, at)),
            (b'a', items) => (0..items).for_each(|_| skip(bytes, at)),
            (_, len) => *at += len,
        }
    }
    let mut at = CHECKPOINT_MAGIC_BYTES as usize;
    let (b'm', fields) = length(checkpoint, &mut at) else {
        panic!("checkpoint wire is a named map")
    };
    for _ in 0..fields {
        let (b's', key) = length(checkpoint, &mut at) else {
            panic!("named field")
        };
        let name = &checkpoint[at..at + key];
        at += key;
        if name == b"envelopes" {
            let (b'a', items) = length(checkpoint, &mut at) else {
                panic!("envelope array")
            };
            return (0..items)
                .map(|_| {
                    let (_, len) = length(checkpoint, &mut at);
                    at += len;
                    len
                })
                .collect();
        }
        skip(checkpoint, &mut at);
    }
    panic!("checkpoint wire lacks envelopes")
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
