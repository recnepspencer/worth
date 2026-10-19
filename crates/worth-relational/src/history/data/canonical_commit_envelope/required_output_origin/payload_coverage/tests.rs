//! Malformed twins start from actual native Create artifacts; none mints custody.
use super::*;
use crate::identity::data::KindId;
use crate::tests::support::*;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};

fn validate(
    execution: &ExecutionRequest<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
    id: EntityId,
    plan: &LoweredAspectContractPlan,
    payload: Option<&AuthoritativeRecordAspectState>,
) -> Result<(), OutputPayloadCoverageFailure> {
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| validate_created_output_payload(envelope, id, plan, payload, context),
            )
        })
        .unwrap()
        .unwrap()
        .0
}

#[test]
fn classified_empty_birth_refuses_missing_or_corrupt_native_create_evidence() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = AspectSchemaFixture::default().build_runtime();
    let result = create_entity_outcome(execution, &runtime, "real-empty-birth");
    let id = changed_entities(&result)[0];
    let envelope = &result.publication().envelope;
    let basis = test_owner_main_basis(&runtime).unwrap();
    let plan = basis
        .inner
        .root
        .schema_authority()
        .entity_aspect_plan(KindId(1))
        .unwrap();
    let partition = basis.inner.root.partition_state(id.partition_id).unwrap();
    let row = partition.entity_arena.get(&id).unwrap();
    let payload = row.extra().authoritative_aspect_state.as_ref();
    assert!(payload.is_none());
    assert!(validate(execution, envelope, id, plan, payload).is_ok());

    let mut missing_allocation = envelope.as_ref().clone();
    missing_allocation.record_allocations.clear();
    let mut duplicate_allocation = envelope.as_ref().clone();
    duplicate_allocation
        .record_allocations
        .push(envelope.record_allocations[0].clone());
    let mut missing_create = envelope.as_ref().clone();
    missing_create.patch.authoritative_record_patches.clear();
    let mut duplicate_create = envelope.as_ref().clone();
    duplicate_create
        .patch
        .authoritative_record_patches
        .push(envelope.patch.authoritative_record_patches[0].clone());
    let mut updated = envelope.as_ref().clone();
    updated.patch.authoritative_record_patches[0].structural_change =
        RecordStructuralChange::Updated;
    let mut missing_schema = envelope.as_ref().clone();
    missing_schema.schema_authority.entity_kinds.clear();
    let mut wrong_kind = envelope.as_ref().clone();
    wrong_kind.schema_authority.entity_kinds[0].kind_id = KindId(99);
    let mut wrong_revision = envelope.as_ref().clone();
    wrong_revision.schema_authority.entity_kinds[0]
        .aspect_plan_revision
        .0 ^= 1;
    for corrupt in [
        missing_allocation,
        duplicate_allocation,
        missing_create,
        duplicate_create,
        updated,
        missing_schema,
        wrong_kind,
        wrong_revision,
    ] {
        assert!(matches!(
            validate(execution, &corrupt, id, plan, payload),
            Err(OutputPayloadCoverageFailure::Corrupt(_))
        ));
    }
    let mut opaque = envelope.as_ref().clone();
    opaque.patch.authoritative_record_patches[0].contains_opaque_aspect = true;
    assert!(matches!(
        validate(execution, &opaque, id, plan, payload),
        Err(OutputPayloadCoverageFailure::Unsupported)
    ));
    assert_eq!(basis.inner.root.commit_id(), Some(result.commit.commit_id));
    release_test_commit_snapshot(&runtime, &result);
}

#[test]
fn classified_nonempty_birth_cannot_use_empty_storage_or_lose_operation_coverage() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    let result = create_entity_outcome(execution, &runtime, "real-present-birth");
    let id = changed_entities(&result)[0];
    let envelope = &result.publication().envelope;
    let basis = test_owner_main_basis(&runtime).unwrap();
    let plan = basis
        .inner
        .root
        .schema_authority()
        .entity_aspect_plan(KindId(1))
        .unwrap();
    assert!(!plan.executable_bindings.is_empty());
    let partition = basis.inner.root.partition_state(id.partition_id).unwrap();
    let row = partition.entity_arena.get(&id).unwrap();
    let payload = row.extra().authoritative_aspect_state.as_ref().unwrap();
    assert!(validate(execution, envelope, id, plan, Some(payload)).is_ok());
    assert!(matches!(
        validate(execution, envelope, id, plan, None),
        Err(OutputPayloadCoverageFailure::Unsupported)
    ));
    let mut omitted_operation = envelope.as_ref().clone();
    omitted_operation.patch.authoritative_record_patches[0].authoritative_patch =
        crate::publication::patch::data::PublishedAuthoritativePatch::empty();
    assert!(matches!(
        validate(execution, &omitted_operation, id, plan, Some(payload)),
        Err(OutputPayloadCoverageFailure::Corrupt(_))
    ));
    // Empty operations do not turn a nonempty admitted schema into a fieldless kind.
    assert!(matches!(
        validate(execution, &omitted_operation, id, plan, None),
        Err(OutputPayloadCoverageFailure::Unsupported)
    ));
    let mut opaque = envelope.as_ref().clone();
    opaque.patch.authoritative_record_patches[0].contains_opaque_aspect = true;
    assert!(matches!(
        validate(execution, &opaque, id, plan, Some(payload)),
        Err(OutputPayloadCoverageFailure::Unsupported)
    ));
    release_test_commit_snapshot(&runtime, &result);
}
