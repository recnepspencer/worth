use super::*;
use crate::capabilities::AspectPlanSource;
use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};

#[test]
fn authored_noops_and_whole_aspects_remain_visible_beyond_semantic_revisions() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = test_owner_main_basis(&runtime).unwrap();
    let equal = update_entity(&execution, &runtime, entity, "before");
    let changed = update_entity(&execution, &runtime, entity, "after");
    let restored = update_entity(&execution, &runtime, entity, "before");
    let contract = runtime
        .entity_aspect_plan(crate::identity::data::KindId(1))
        .unwrap()
        .executable_bindings
        .iter()
        .find(|binding| binding.aspect_key() == &aspect_key("name"))
        .unwrap()
        .contract
        .clone();
    let whole = patch(
        &runtime,
        execution,
        entity,
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
            value: ContractValidationInput::Scalar(AspectValue::String("before".into())),
        },
    );
    let target = test_owner_main_basis(&runtime).unwrap();
    let (commits, _) = collect(
        &runtime,
        execution,
        cursor(&runtime, execution, &original, &target, entity),
    );
    assert_eq!(
        commits,
        [
            equal.commit.commit_id,
            changed.commit.commit_id,
            restored.commit.commit_id,
            whole.commit.commit_id
        ]
    );
    let old = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let noop = runtime
        .read_truth()
        .project_snapshot(&equal.snapshot)
        .unwrap();
    let current = runtime
        .read_truth()
        .project_snapshot(&whole.snapshot)
        .unwrap();
    assert_eq!(
        old.entity_field_revision(&execution, entity, &name_locator())
            .unwrap(),
        noop.entity_field_revision(&execution, entity, &name_locator())
            .unwrap()
    );
    assert_ne!(
        old.entity_field_revision(&execution, entity, &name_locator())
            .unwrap(),
        current
            .entity_field_revision(&execution, entity, &name_locator())
            .unwrap()
    );
    assert!(
        old.compare_entity_field_support(&execution, &current, entity, &name_locator())
            .unwrap(),
        "matching actual support does not erase the four authored occurrences above"
    );
    let worker_only = worth_execution::SerialRequest::from_policy(
        &worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    assert!(worker_only.memory().is_none());
    assert!(
        old.compare_entity_field_support(
            &ExecutionRequest::serial(&worker_only),
            &current,
            entity,
            &name_locator()
        )
        .unwrap(),
        "borrowed support comparison preserves the original worker-only posture"
    );
    for outcome in [&created, &equal, &changed, &restored, &whole] {
        release_test_commit_snapshot(&runtime, outcome);
    }
    absent_field_writes(execution);
}

fn absent_field_writes(execution: ExecutionRequest<'_, '_>) {
    let binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let worth_foundational::facade::AspectShape::Struct(shape) = binding.contract.shape() else {
        unreachable!("actual summary fixture is a struct contract");
    };
    let optional = worth_foundational::facade::aspects()
        .contract()
        .for_key(binding.contract.key().clone())
        .identified_by(binding.contract.identity())
        .at_revision(binding.contract.revision())
        .struct_with(
            shape.clone(),
            binding.contract.masks().clone(),
            worth_foundational::facade::AbsenceLaw::Optional,
            binding.contract.equivalence(),
            binding.contract.evolution(),
        );
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: binding.binding,
            contract: optional,
        }],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let entity = super::super::struct_field_patch_authority::create_entity_with_summary_fields(
        &execution,
        &runtime,
        "absent-writes",
        "title",
        "open",
        false,
        false,
    );
    let original = test_owner_main_basis(&runtime).unwrap();
    let contract = runtime
        .entity_aspect_plan(crate::identity::data::KindId(1))
        .unwrap()
        .contract_for(&aspect_key("summary"))
        .unwrap();
    let clear = || PortableAspectPatchOperation::PatchFields {
        basis: PortableAspectContractBasis::from_contract(&contract),
        selected_fields: vec![field_key("status")],
        field_sets: vec![],
        field_clears: vec![field_key("status")],
    };
    let removed = patch(&runtime, execution, entity, clear());
    let equal_absent = patch(&runtime, execution, entity, clear());
    let target = test_owner_main_basis(&runtime).unwrap();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key("summary"),
        CanonicalFieldPath::single(field_key("status")),
    );
    let key = RelationalAuthoredWriteSelector::Record {
        record: RecordRef::Entity(entity),
        scope: RelationalAuthoredWriteScope::Field(
            aspect_key("summary"),
            locator.field_path().clone(),
        ),
    };
    let selected = selected(&runtime, &target);
    let cursor = runtime
        .authored_write_history_cursor(execution, &original, selected, &[key])
        .unwrap();
    assert_eq!(
        collect(&runtime, execution, cursor).0,
        [removed.commit.commit_id, equal_absent.commit.commit_id]
    );
    // Use the real commit snapshots already issued by the mutations.
    let before = runtime
        .read_truth()
        .project_snapshot(&removed.snapshot)
        .unwrap();
    let after = runtime
        .read_truth()
        .project_snapshot(&equal_absent.snapshot)
        .unwrap();
    assert!(before
        .compare_entity_field_support(&execution, &after, entity, &locator)
        .unwrap());
    let no_aspect = patch(
        &runtime,
        execution,
        entity,
        PortableAspectPatchOperation::ClearWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
        },
    );
    let sibling_only = patch(
        &runtime,
        execution,
        entity,
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
            value: ContractValidationInput::Struct(
                worth_foundational::facade::StructAspectValue::new([(
                    field_key("title"),
                    AspectValue::String("sibling-only".into()),
                )])
                .unwrap(),
            ),
        },
    );
    let absent_container = runtime
        .read_truth()
        .project_snapshot(&no_aspect.snapshot)
        .unwrap();
    let with_sibling = runtime
        .read_truth()
        .project_snapshot(&sibling_only.snapshot)
        .unwrap();
    assert_eq!(
        absent_container
            .entity_aspect_presence_scalar_field(
                &execution,
                &with_sibling,
                entity,
                crate::identity::data::KindId(1),
                &aspect_key("summary"),
            )
            .unwrap(),
        None,
        "struct field lowering cannot initialize or remove its aspect container"
    );
    assert_eq!(
        absent_container
            .entity_field_revision(&execution, entity, &locator)
            .unwrap(),
        with_sibling
            .entity_field_revision(&execution, entity, &locator)
            .unwrap()
    );
    assert!(
        absent_container
            .compare_entity_field_support(&execution, &with_sibling, entity, &locator)
            .unwrap(),
        "an absent selected field does not depend on its sibling's aspect container"
    );
    let title = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key("summary"),
        CanonicalFieldPath::single(field_key("title")),
    );
    assert!(
        !absent_container
            .compare_entity_field_support(&execution, &with_sibling, entity, &title)
            .unwrap(),
        "the actual newly present sibling remains distinguishable"
    );
    for outcome in [&removed, &equal_absent, &no_aspect, &sibling_only] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

fn patch(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    entity: EntityId,
    operation: PortableAspectPatchOperation,
) -> CommitResult {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("authored-aspect").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: PortableRecordAspectPatch::new([operation]),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(&execution, runtime).unwrap()
}
