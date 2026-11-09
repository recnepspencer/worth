//! Real Native population and independent MessagePack observation for the courts.
use super::*;
pub(super) fn runtime_with_rows(rows: usize) -> RelationalRuntime {
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
pub(super) fn embedded_envelope_lengths(checkpoint: &[u8]) -> Vec<usize> {
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
pub(super) fn memory_outcome(leased: bool, memory: u64, cancelled: bool) -> Result<usize, Denial> {
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
