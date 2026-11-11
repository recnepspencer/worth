//! A capture cannot replace canonical adjacency authority by rehashing leaves.
use super::*;
use crate::durability::data::*;
use crate::tests::support::*;

#[test]
fn checkpoint_readmission_refuses_retired_integer_array_digest_before_restore() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    create_entity(&execution, &runtime, "binary-digest-checkpoint");
    let native = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    decode_checkpoint(&execution, native.bytes()).unwrap();
    let wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    let digest = wire.manifest.mirror[0].kind_authority.head;
    let mut current = rmp_serde::to_vec("head").unwrap();
    current.extend(rmp_serde::to_vec(&digest).unwrap());
    let mut retired = rmp_serde::to_vec("head").unwrap();
    retired.extend(rmp_serde::to_vec(digest.bytes()).unwrap());
    assert_ne!(current, retired);
    // Rewrite only the representation of this actual captured digest. Every
    // semantic value and every other field keeps the owning producer's bytes.
    let mut bytes = Vec::new();
    let mut remaining = native.bytes();
    let mut replacements = 0;
    while let Some(at) = remaining
        .windows(current.len())
        .position(|part| part == current)
    {
        bytes.extend_from_slice(&remaining[..at]);
        bytes.extend_from_slice(&retired);
        remaining = &remaining[at + current.len()..];
        replacements += 1;
    }
    bytes.extend_from_slice(remaining);
    assert!(replacements > 0);
    let stop = decode_checkpoint(&execution, &bytes).unwrap_err();
    assert_eq!(
        expect_durability_failure(stop).class,
        RecoveryFailureClass::CorruptCheckpoint
    );
}
