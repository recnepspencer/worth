//! Formatter proofs. The literal native payload below is not native admission
//! evidence; real captured producer readmission is covered by certification.

use sha2::{Digest, Sha256};
use worth_relational::facade::{
    durability::{RecoveryFailureClass, RelationalNativeCheckpoint},
    errors::{ErrorOperation, RelationalSubsystem},
    identity::{EntityId, PartitionId},
};

use super::{reserve_buffer, sizing::ValidatedCheckpointSize};
use crate::domain_computation::primary_graph::{
    application_attempt::{WorthQueryApplicationObservedFact, WorthQueryCheckpointOutputRole},
    application_contribution::WorthQueryProducerDemandResources,
    application_output_demand::{
        WorthQueryAcceptedOutputCheckpointIdentity, WorthQueryAcceptedOutputCheckpointPosture,
    },
    tests::recoverable_commit_support::recoverable_application_world,
    WorthQueryApplicationCheckpoint, WorthQueryApplicationOutputPosture,
};

#[test]
fn checkpoint_one_buffer_matches_independent_v8_grammar_and_sections() {
    let (world, receipt) = recoverable_application_world(141, "checkpoint-wire-proof");
    let scope = receipt.principal_scope().scope();
    let facts = super::super::facts::encode(&[WorthQueryApplicationObservedFact::SourceEntity {
        entity_id: entity(9),
    }])
    .unwrap();
    let rows = vec![
        WorthQueryAcceptedOutputCheckpointIdentity {
            producer: "producer-a".to_owned(),
            posture: WorthQueryAcceptedOutputCheckpointPosture::Performed,
            source: [1; 32],
            scope,
            source_partition: [2; 32],
            producer_dependency: Some([3; 32]),
            idempotency_key: [4; 32],
            resources: Some(WorthQueryProducerDemandResources::new(4096, 8192)),
            roles: vec![
                role(
                    "a",
                    WorthQueryApplicationOutputPosture::Preserve,
                    "alpha",
                    5,
                ),
                role("b", WorthQueryApplicationOutputPosture::Create, "beta", 6),
                role("c", WorthQueryApplicationOutputPosture::Retire, "gamma", 7),
            ],
            producer_facts: Some(facts.clone()),
            producer_fact_wire_version: 8,
        },
        WorthQueryAcceptedOutputCheckpointIdentity {
            producer: "producer-b".to_owned(),
            posture: WorthQueryAcceptedOutputCheckpointPosture::StableReused,
            source: [11; 32],
            scope,
            source_partition: [12; 32],
            producer_dependency: None,
            idempotency_key: [14; 32],
            resources: None,
            roles: Vec::new(),
            producer_facts: None,
            producer_fact_wire_version: 0,
        },
    ];
    let native = b"literal-native-formatter-payload";
    let mut scope_wire = Vec::new();
    scope_wire.extend_from_slice(&scope.partition_id().to_be_bytes());
    scope_wire.extend_from_slice(&scope.local_slot().to_be_bytes());
    scope_wire.extend_from_slice(&scope.generation().to_be_bytes());
    let expected = historical_v8(
        world.application.publication().bootstrap_commit_id().0,
        &scope_wire,
        native,
        &facts,
    );
    let (checkpoint, sections) = WorthQueryApplicationCheckpoint::encode(
        RelationalNativeCheckpoint::from_untrusted_bytes(native.to_vec().into_boxed_slice()),
        world.application.publication(),
        &rows,
    )
    .unwrap();
    assert_eq!(checkpoint.bytes(), expected);
    assert_eq!(sections.framing_bytes(), 66);
    assert_eq!(sections.native_bytes(), native.len());
    assert_eq!(sections.accepted_output_count(), 2);
    // Two 189-byte fixed rows, literal text widths, three 33-byte role frames,
    // and the independently encoded fact payload. No production size helper.
    let expected_rows = 2 * 189 + 20 + 3 * 33 + 3 + 14 + facts.len();
    assert_eq!(sections.accepted_output_bytes(), expected_rows);
    assert_eq!(sections.total_bytes(), 66 + native.len() + expected_rows);
    assert_eq!(sections.total_bytes(), checkpoint.bytes().len());
    assert_eq!(sections.native_sections(), None);

    // Observe this platform's real reserved capacity before boxing. The Vec
    // contract allows excess capacity; the claim is no growth while writing.
    let size = ValidatedCheckpointSize::new(native.len(), &rows).unwrap();
    let mut buffer = reserve_buffer(size.total()).unwrap();
    let capacity = buffer.capacity();
    assert!(capacity >= expected.len());
    buffer.extend_from_slice(b"WQAPCP01");
    buffer.extend_from_slice(&[0; 32]);
    super::body::write(
        &mut buffer,
        native,
        world.application.publication(),
        &rows,
        &size,
    )
    .unwrap();
    assert_eq!(buffer.len(), expected.len());
    assert_eq!(buffer.capacity(), capacity);

    // A stale size witness cannot permit even one byte beyond its quoted
    // frame. Exercise the real writer guard rather than allocator injection.
    let short_size = ValidatedCheckpointSize::new(3, &[]).unwrap();
    let mut short_buffer = reserve_buffer(short_size.total()).unwrap();
    let short_capacity = short_buffer.capacity();
    short_buffer.extend_from_slice(b"WQAPCP01");
    short_buffer.extend_from_slice(&[0; 32]);
    let denial = super::body::write(
        &mut short_buffer,
        b"abcd",
        world.application.publication(),
        &[],
        &short_size,
    )
    .unwrap_err();
    assert_eq!(
        denial.class,
        RecoveryFailureClass::CheckpointFrameSizeMismatch
    );
    assert_capture_context(&denial);
    assert!(short_buffer.len() <= short_size.total());
    assert_eq!(short_buffer.capacity(), short_capacity);
}

#[test]
fn checkpoint_impossible_vec_reservation_is_a_typed_reservation_refusal() {
    // Vec's actual capacity-overflow guard refuses this without trying to
    // exhaust host memory. This is not a synthetic allocator fault hook.
    let denial = reserve_buffer(usize::MAX).unwrap_err();
    assert_eq!(
        denial.class,
        RecoveryFailureClass::CheckpointAllocationUnavailable
    );
    assert_capture_context(&denial);
}

#[test]
fn checkpoint_native_length_overflow_is_a_typed_sizing_refusal() {
    for native_len in [usize::MAX, isize::MAX as usize] {
        let denial = match ValidatedCheckpointSize::new(native_len, &[]) {
            Ok(_) => panic!("framing cannot fit beside this native length"),
            Err(denial) => denial,
        };
        assert_eq!(denial.class, RecoveryFailureClass::CheckpointSizeOverflow);
        assert_capture_context(&denial);
    }
}

fn assert_capture_context(denial: &worth_relational::facade::durability::DurabilityError) {
    assert_eq!(denial.context.subsystem, RelationalSubsystem::Durability);
    assert_eq!(denial.context.operation, ErrorOperation::WriteDurableStore);
    assert_eq!(denial.context.suggested_fix, None);
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId(17), slot, 3)
}

fn role(
    name: &str,
    posture: WorthQueryApplicationOutputPosture,
    entity_name: &str,
    slot: u64,
) -> WorthQueryCheckpointOutputRole {
    WorthQueryCheckpointOutputRole {
        role: name.to_owned(),
        posture,
        entity_name: entity_name.to_owned(),
        entity: entity(slot),
    }
}

/// Independent historical v8 grammar with literal fixture branch choices.
/// It does not call the production body writer, resource codec, or size helper.
fn historical_v8(bootstrap: u64, scope: &[u8], native: &[u8], facts: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&8_u16.to_be_bytes());
    body.extend_from_slice(&bootstrap.to_be_bytes());
    body.extend_from_slice(&(native.len() as u64).to_be_bytes());
    body.extend_from_slice(&2_u64.to_be_bytes());
    body.extend_from_slice(native);
    text(&mut body, b"producer-a");
    body.push(0);
    body.extend_from_slice(&[1; 32]);
    body.extend_from_slice(scope);
    body.extend_from_slice(&[2; 32]);
    body.push(1);
    body.extend_from_slice(&[3; 32]);
    body.extend_from_slice(&[4; 32]);
    body.push(1);
    body.extend_from_slice(&4096_u64.to_be_bytes());
    body.extend_from_slice(&8192_u64.to_be_bytes());
    body.extend_from_slice(&3_u64.to_be_bytes());
    for (name, posture, entity_name, slot) in [
        (b"a".as_slice(), 0, b"alpha".as_slice(), 5_u64),
        (b"b".as_slice(), 1, b"beta".as_slice(), 6),
        (b"c".as_slice(), 2, b"gamma".as_slice(), 7),
    ] {
        text(&mut body, name);
        body.push(posture);
        text(&mut body, entity_name);
        body.extend_from_slice(&17_u32.to_be_bytes());
        body.extend_from_slice(&slot.to_be_bytes());
        body.extend_from_slice(&3_u32.to_be_bytes());
    }
    body.extend_from_slice(&8_u16.to_be_bytes());
    body.extend_from_slice(&(facts.len() as u64).to_be_bytes());
    body.extend_from_slice(facts);
    text(&mut body, b"producer-b");
    body.push(1);
    body.extend_from_slice(&[11; 32]);
    body.extend_from_slice(scope);
    body.extend_from_slice(&[12; 32]);
    body.push(0);
    body.extend_from_slice(&[0; 32]);
    body.extend_from_slice(&[14; 32]);
    body.extend_from_slice(&[0; 17]);
    body.extend_from_slice(&0_u64.to_be_bytes());
    body.extend_from_slice(&0_u16.to_be_bytes());
    body.extend_from_slice(&0_u64.to_be_bytes());
    let mut bytes = b"WQAPCP01".to_vec();
    bytes.extend_from_slice(&Sha256::digest(&body));
    bytes.extend_from_slice(&body);
    bytes
}

fn text(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value);
}
