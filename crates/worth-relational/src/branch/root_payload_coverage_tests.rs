//! Malformed twins start from actual native Create artifacts; none mints custody.
use super::*;
use crate::history::data::CanonicalCommitEnvelopeBody;
use crate::history::data::{CanonicalCommitEnvelope, CanonicalRecordAllocation};
use crate::identity::data::KindId;
use crate::identity::data::PartitionId;
use crate::publication::patch::data::PublishedAuthoritativePatch;
use crate::schema::data::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_foundational::facade::{
    validate_aspect_value, AspectContract, AspectContractRevision, AspectIdentity, AspectKey,
    AspectValue, AuthoritativeRecordAspectPatch, ContractValidationInput, FieldKey,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

fn validate(
    execution: &ExecutionRequest<'_, '_>,
    envelope: &CanonicalCommitEnvelopeBody,
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
    let request = crate::execution::test_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let fixture = birth_fixture(execution, "real-empty-birth", None);
    let (id, envelope, plan, payload) = (
        fixture.id,
        &fixture.envelope,
        &fixture.plan,
        fixture.payload.as_ref(),
    );
    assert!(payload.is_none());
    assert!(validate(execution, envelope, id, plan, payload).is_ok());

    let mut missing_allocation = envelope.independent_fixture_image();
    missing_allocation
        .body_mut_for_test()
        .record_allocations
        .clear();
    let mut duplicate_allocation = envelope.independent_fixture_image();
    duplicate_allocation
        .body_mut_for_test()
        .record_allocations
        .push(envelope.record_allocations[0].clone());
    let mut missing_create = envelope.independent_fixture_image();
    missing_create
        .body_mut_for_test()
        .patch
        .authoritative_record_patches
        .clear();
    let mut duplicate_create = envelope.independent_fixture_image();
    duplicate_create
        .body_mut_for_test()
        .patch
        .authoritative_record_patches
        .push(envelope.patch.authoritative_record_patches[0].clone());
    let mut updated = envelope.independent_fixture_image();
    updated
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .structural_change = RecordStructuralChange::Updated;
    let mut missing_schema = envelope.independent_fixture_image();
    let mut parts = (*missing_schema.schema_authority).clone();
    parts.entity_kinds.clear();
    missing_schema.body_mut_for_test().schema_authority = parts.into();
    let mut wrong_kind = envelope.independent_fixture_image();
    let mut parts = (*wrong_kind.schema_authority).clone();
    parts.entity_kinds[0].kind_id = KindId(99);
    wrong_kind.body_mut_for_test().schema_authority = parts.into();
    let mut wrong_revision = envelope.independent_fixture_image();
    let mut parts = (*wrong_revision.schema_authority).clone();
    parts.entity_kinds[0].aspect_plan_revision.0 ^= 1;
    wrong_revision.body_mut_for_test().schema_authority = parts.into();
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
    let mut opaque = envelope.independent_fixture_image();
    opaque
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect = true;
    assert!(matches!(
        validate(execution, &opaque, id, plan, payload),
        Err(OutputPayloadCoverageFailure::Unsupported(
            UnsupportedPayload::OpaqueRecord
        ))
    ));
    let failure = validate(execution, &opaque, id, plan, payload).unwrap_err();
    let error = super::super::readmission::coverage_commit_stop(failure, &opaque, id, plan, 0);
    assert!(error.detail().contains("reason=opaque-record"));
    assert_eq!(error.context().affected_records, [RecordRef::Entity(id)]);
    assert_eq!(fixture.root.commit_id(), Some(envelope.commit.commit_id));
}

#[test]
fn classified_nonempty_birth_cannot_use_empty_storage_or_lose_operation_coverage() {
    let request = crate::execution::test_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let fixture = scalar_birth_fixture(execution, "real-present-birth");
    let (id, envelope, plan, payload) = (
        fixture.id,
        &fixture.envelope,
        &fixture.plan,
        fixture.payload.as_ref().unwrap(),
    );
    assert!(!plan.executable_bindings.is_empty());
    assert!(validate(execution, envelope, id, plan, Some(payload)).is_ok());
    assert!(matches!(
        validate(execution, envelope, id, plan, None),
        Err(OutputPayloadCoverageFailure::Unsupported(
            UnsupportedPayload::EmptyWithOperations(_)
        ))
    ));
    let failure = validate(execution, envelope, id, plan, None).unwrap_err();
    let error = super::super::readmission::coverage_commit_stop(failure, envelope, id, plan, 0);
    assert!(error
        .detail()
        .contains("reason=empty-payload-with-operations"));
    assert!(error.detail().contains("operation_ordinal=0"));
    assert_eq!(error.context().affected_records, [RecordRef::Entity(id)]);
    let mut omitted_operation = envelope.independent_fixture_image();
    omitted_operation
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .authoritative_patch =
        crate::publication::patch::data::PublishedAuthoritativePatch::empty();
    assert!(matches!(
        validate(execution, &omitted_operation, id, plan, Some(payload)),
        Err(OutputPayloadCoverageFailure::Corrupt(_))
    ));
    // Empty operations do not turn a nonempty admitted schema into a fieldless kind.
    assert!(matches!(
        validate(execution, &omitted_operation, id, plan, None),
        Err(OutputPayloadCoverageFailure::Unsupported(
            UnsupportedPayload::EmptyWithBindings { .. }
        ))
    ));
    let mut opaque = envelope.independent_fixture_image();
    opaque
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect = true;
    assert!(matches!(
        validate(execution, &opaque, id, plan, Some(payload)),
        Err(OutputPayloadCoverageFailure::Unsupported(
            UnsupportedPayload::OpaqueRecord
        ))
    ));
}

struct BirthFixture {
    id: EntityId,
    envelope: CanonicalCommitEnvelope,
    plan: LoweredAspectContractPlan,
    payload: Option<AuthoritativeRecordAspectState>,
    root: std::sync::Arc<crate::branch::root::RelationalBranchRoot>,
}

fn aspect_key(name: &str) -> AspectKey {
    AspectKey::new(name).unwrap()
}

fn field_key(name: &str) -> FieldKey {
    FieldKey::new(name).unwrap()
}

fn scalar_birth_fixture(execution: &ExecutionRequest<'_, '_>, label: &str) -> BirthFixture {
    birth_fixture(
        execution,
        label,
        Some((
            AspectContract::scalar(
                aspect_key("name"),
                AspectIdentity(1),
                AspectContractRevision(1),
                ScalarAspectType::String,
            ),
            ContractValidationInput::Scalar(AspectValue::String(label.into())),
        )),
    )
}

fn birth_fixture(
    execution: &ExecutionRequest<'_, '_>,
    label: &str,
    source: Option<(AspectContract, ContractValidationInput)>,
) -> BirthFixture {
    use crate::schema::aspect_plans::lower_aspect_plans;
    use crate::transactions::data::{
        aspect_field_patch::AspectFieldPatch,
        intents::mutation_intent::{CreateIntent, MutationIntent},
        primitives::EntitySpec,
    };

    let declaration = source
        .as_ref()
        .map(|(contract, _)| DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: field_key(contract.key().as_str()),
            },
            contract: contract.clone(),
        });
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "payload-fixture".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(
                declaration.into_iter().collect(),
            ),
        })
        .unwrap();
    let plan = lower_aspect_plans(&registry)
        .entity_plans
        .remove(&KindId(1))
        .unwrap();
    let (mut partition, entities, _) =
        crate::storage::overlay::small_relation_partition::relation_partition(
            PartitionId::main(),
            1,
            &[],
        );
    let id = entities[0];
    let row = partition.entity_arena.get(&id).unwrap();
    assert!(row.is_live());
    let record = RecordRef::Entity(id);
    let mut envelope =
        crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update(
            record.clone(),
            id,
        );
    let body = envelope.body_mut_for_test();
    body.schema_authority = registry.authority_snapshot();
    body.install_record_allocations(vec![CanonicalRecordAllocation::new(0, record)]);
    body.merged_plan.merged_intents =
        vec![MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: id.partition_id,
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(label),
            fields: AspectFieldPatch::default(),
        }))];
    body.patch.authoritative_record_patches[0].structural_change = RecordStructuralChange::Created;
    let payload = source.map(|(contract, input)| {
        let value = match &input {
            ContractValidationInput::Scalar(value) => Value::Scalar(value.clone()),
            ContractValidationInput::Struct(value) => Value::Struct(value.clone()),
        };
        body.patch.authoritative_record_patches[0].authoritative_patch =
            PublishedAuthoritativePatch::new(vec![Operation::WholeAspectSet {
                aspect_key: contract.key().clone(),
                aspect_identity: contract.identity(),
                contract_revision: contract.revision(),
                binding: AspectBinding::EntityField {
                    field: field_key(contract.key().as_str()),
                },
                value,
            }]);
        let TransitionOutcome::Success(admitted) = validate_aspect_value(&contract, input) else {
            panic!("the physical row carries valid retained data");
        };
        let TransitionOutcome::Success(patch) =
            AuthoritativeRecordAspectPatch::whole_aspect([admitted], [])
        else {
            panic!("one typed aspect cannot duplicate a binding");
        };
        let TransitionOutcome::Success(Some(payload)) = patch.apply_to_optional(None) else {
            panic!("the row carries its admitted aspect");
        };
        payload.into_parts().into_parts().0
    });
    use crate::storage::substrate::EntityExtra;
    partition
        .entity_arena
        .apply_extra_update(
            id.slot_index(),
            EntityExtra::from_unbudgeted_parts(None, None, payload),
            envelope.commit.version_id,
        )
        .unwrap();
    let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
    journal.entity_slots.insert(id.slot_index());
    let delta = crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
        &std::collections::BTreeMap::from([(id.partition_id, (partition.clone(), journal))]),
    );
    let root = crate::branch::root::RelationalBranchRoot::prepare_capture(
        execution,
        &crate::branch::root::RelationalBranchRootIdentityIssuer::default(),
        &std::collections::BTreeMap::from([(id.partition_id, partition)]),
        &delta,
        None,
        std::sync::Arc::new(envelope.independent_fixture_image()),
        &Default::default(),
        &registry,
        &Default::default(),
        None,
    )
    .unwrap()
    .into_root();
    let payload = root
        .partition_state(id.partition_id)
        .unwrap()
        .unwrap()
        .entity_arena
        .get(&id)
        .unwrap()
        .extra()
        .authoritative_aspect_state()
        .cloned();
    BirthFixture {
        id,
        envelope,
        plan,
        payload,
        root,
    }
}

#[path = "root_payload_coverage_tests/diagnostics.rs"]
mod diagnostics;
