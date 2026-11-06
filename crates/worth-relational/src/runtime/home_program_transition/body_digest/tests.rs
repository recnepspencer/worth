//! Pure commitments and shared leaf wire truth, independent of a live runtime.
use super::*;
use crate::aspect_wire::{write_aspect_field_locator, write_aspect_value};
use crate::transactions::data::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InlineBytes};

fn batch(value: i64) -> WorkerIntentBatch {
    WorkerIntentBatch::new("authored").push(MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: crate::identity::data::PartitionId(1),
            kind_id: crate::identity::data::KindId(2),
            client_key: crate::symbols::data::ClientKey::raw("first"),
            fields: AspectFieldPatch::from_locator(
                planned_single_field_locator(
                    AspectKey::new("test.body").unwrap(),
                    FieldKey::new("value").unwrap(),
                ),
                AspectValue::Int64(value),
            ),
        },
    )))
}

#[test]
fn authored_body_commitment_covers_values_order_and_batch_posture() {
    let request = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let intent = RelationalTransactionIntent::ordinary();
    let hash = |batches: &[WorkerIntentBatch]| {
        home_program_transition_body_digest(&execution, &intent, batches).unwrap()
    };
    assert_eq!(hash(&[batch(1)]), hash(&[batch(1)]));
    assert_ne!(hash(&[batch(1)]), hash(&[batch(2)]));
    assert_ne!(hash(&[batch(1), batch(2)]), hash(&[batch(2), batch(1)]));
    let mut changed = batch(1);
    changed.worker_local_only = false;
    assert_ne!(hash(&[batch(1)]), hash(&[changed]));
    assert_ne!(
        hash(&[batch(1)]),
        hash(&[batch(1).with_partition_key("other")])
    );
}

#[test]
fn body_fingerprinting_refuses_the_original_cancelled_request_and_unrepresentable_length() {
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        cancellation.token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    assert!(matches!(
        home_program_transition_body_digest(
            &execution,
            &RelationalTransactionIntent::ordinary(),
            &[batch(1)]
        ),
        Err(RelationalHomeProgramBodyDenial::Request(_))
    ));
    let request = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut out = BodyOutput {
        request: &execution,
        hash: Sha256::new(),
        stopped: None,
    };
    if let Ok(length) = usize::try_from(u64::from(u32::MAX) + 1) {
        assert!(matches!(
            out.length(length),
            Err(RelationalHomeProgramBodyDenial::CapacityOverflow)
        ));
    }
}

#[test]
fn streamed_leaf_commitments_keep_the_preexisting_canonical_wire_bytes() {
    let request = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for (value, golden) in [
        (
            AspectValue::InlineBytes(InlineBytes::try_new(vec![0, 128, 255]).unwrap()),
            vec![24, 3, 0, 0, 0, 0, 128, 255],
        ),
        (
            AspectValue::Int64(-12),
            vec![5, 244, 255, 255, 255, 255, 255, 255, 255],
        ),
    ] {
        let mut out = BodyOutput {
            request: &execution,
            hash: Sha256::new(),
            stopped: None,
        };
        write_aspect_value(&mut out, &value).unwrap();
        assert_eq!(
            <[u8; 32]>::from(out.hash.finalize()),
            <[u8; 32]>::from(Sha256::digest(&golden))
        );
        assert_eq!(crate::aspect_wire::encode_aspect_value(&value), golden);
    }
    let locator =
        planned_single_field_locator(AspectKey::new("a").unwrap(), FieldKey::new("b").unwrap());
    let golden = [2, 5, 1, 0, 0, 0, b'a', 1, 0, 0, 0, 1, 0, 0, 0, b'b'];
    let mut out = BodyOutput {
        request: &execution,
        hash: Sha256::new(),
        stopped: None,
    };
    write_aspect_field_locator(&mut out, &locator).unwrap();
    assert_eq!(
        <[u8; 32]>::from(out.hash.finalize()),
        <[u8; 32]>::from(Sha256::digest(golden))
    );
    assert_eq!(
        crate::aspect_wire::encode_aspect_field_locator(&locator),
        golden
    );
}

#[test]
fn authored_body_commitment_covers_schema_policy_and_refuses_live_materialization() {
    use crate::schema::data::{
        ProposedSchemaTransition, SchemaId, SchemaReconciliationPolicy, SchemaVersionId,
    };
    let request = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let proposal = ProposedSchemaTransition {
        source_schema_id: SchemaId("source".into()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("target".into()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![],
    };
    let one = RelationalTransactionIntent::ordinary().with_schema_transition(
        proposal.clone(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );
    let other = RelationalTransactionIntent::ordinary().with_schema_transition(
        proposal,
        Some(SchemaReconciliationPolicy::PreserveTargetContract),
    );
    assert_ne!(
        home_program_transition_body_digest(&execution, &one, &[]).unwrap(),
        home_program_transition_body_digest(&execution, &other, &[]).unwrap()
    );
    assert_ne!(
        home_program_transition_body_digest(&execution, &one, &[]).unwrap(),
        home_program_transition_body_digest(
            &execution,
            &RelationalTransactionIntent::ordinary(),
            &[]
        )
        .unwrap()
    );
    let materialization = RelationalTransactionIntent::materialization(
        crate::mvcc::RelationalMaterializationTransactionMode::Suspend,
    );
    assert!(matches!(
        home_program_transition_body_digest(&execution, &materialization, &[]),
        Err(RelationalHomeProgramBodyDenial::UnsupportedMaterialization)
    ));
}

#[test]
fn scalar_framing_distinguishes_embedded_nul_from_separate_values() {
    let request = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut together = BodyOutput {
        request: &execution,
        hash: Sha256::new(),
        stopped: None,
    };
    together.scalar(&"a\0b").unwrap();
    let mut separate = BodyOutput {
        request: &execution,
        hash: Sha256::new(),
        stopped: None,
    };
    separate.scalar(&"a").unwrap();
    separate.scalar(&"b").unwrap();
    assert_ne!(
        <[u8; 32]>::from(together.hash.finalize()),
        <[u8; 32]>::from(separate.hash.finalize())
    );
}
