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

fn int(bytes: &[u8], at: &mut usize, width: usize) -> usize {
    let value = bytes[*at..*at + width]
        .iter()
        .fold(0_usize, |sum, byte| (sum << 8) | usize::from(*byte));
    *at += width;
    value
}

/// One MessagePack header: its family and its pair, item or byte count.
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
        0xc7 => (b'x', int(bytes, at, 1) + 1),
        0xc8 => (b'x', int(bytes, at, 2) + 1),
        0xc9 => (b'x', int(bytes, at, 4) + 1),
        0x00..=0x7f | 0xe0..=0xff | 0xc0 | 0xc2 | 0xc3 => (b'x', 0),
        0xcc | 0xd0 => (b'x', 1),
        0xcd | 0xd1 => (b'x', 2),
        0xca | 0xce | 0xd2 => (b'x', 4),
        0xcb | 0xcf | 0xd3 => (b'x', 8),
        0xd4..=0xd8 => (b'x', 1 + (1 << (tag - 0xd4))),
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

/// Independent MessagePack walk: the bytes of each inline member, in order.
pub(super) fn inline_members(checkpoint: &[u8]) -> Vec<&[u8]> {
    let mut at = CHECKPOINT_MAGIC.len();
    let (b'm', fields) = length(checkpoint, &mut at) else {
        panic!("checkpoint wire is a named map")
    };
    for _ in 0..fields {
        let (b's', key) = length(checkpoint, &mut at) else {
            panic!("named field")
        };
        let name = &checkpoint[at..at + key];
        at += key;
        if name == b"members" {
            let (b'a', items) = length(checkpoint, &mut at) else {
                panic!("member array")
            };
            return (0..items)
                .map(|_| {
                    let (b'a', 2) = length(checkpoint, &mut at) else {
                        panic!("locator and bytes")
                    };
                    skip(checkpoint, &mut at);
                    let (b'b', len) = length(checkpoint, &mut at) else {
                        panic!("member bytes")
                    };
                    at += len;
                    &checkpoint[at - len..at]
                })
                .collect();
        }
        skip(checkpoint, &mut at);
    }
    panic!("checkpoint wire lacks members")
}

/// Independent walk of one journal payload, a map of named fields: the bytes
/// its digest hashes, which are each field's encoded value and the root.
pub(super) fn digest_input_bytes(payload: &[u8]) -> usize {
    let mut at = 0;
    let (b'm', fields) = length(payload, &mut at) else {
        panic!("journal payload is a named map")
    };
    let mut values = 0;
    for _ in 0..fields {
        let (b's', key) = length(payload, &mut at) else {
            panic!("named payload field")
        };
        at += key;
        let start = at;
        skip(payload, &mut at);
        values += at - start;
    }
    assert_eq!(at, payload.len(), "the payload is exactly its fields");
    values + DIGEST_ROOT_BYTES
}

/// The checkpoint length and what its request holds while the checkpoint
/// lives, or its refusal; every retained byte is released afterwards.
pub(super) fn memory_outcome(
    leased: bool,
    memory: u64,
    cancelled: bool,
) -> Result<(usize, u64), Denial> {
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
        .map(|checkpoint| {
            let held = match &lease {
                Some(lease) => lease.charged_memory_bytes(),
                None => serial.memory().unwrap().charged_memory_bytes(),
            };
            (checkpoint.bytes().len(), held)
        });
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    if let Some(lease) = &lease {
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    outcome
}
