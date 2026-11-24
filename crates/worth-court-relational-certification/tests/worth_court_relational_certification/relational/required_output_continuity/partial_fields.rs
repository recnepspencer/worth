//! Real field-level clearing must preserve the sibling and unpatched companion.
use super::*;
use worth_foundational::facade::{
    ContractValidatedAspectValueView, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableRecordAspectPatch,
};
use worth_relational::facade::{
    schema::DeclaredAspectContractBinding,
    storage::RecordLifecycleState,
    transactions::{ApplyEntityAspectPatchIntent, EntityMutationIntent, RecordRef},
};
use worth_relational::facade::{
    schema::RelationalSchemaRegistry,
    transactions::{CreateIntent, EntitySpec, MutationIntent},
};
use worth_testkit_relational::aspect_storage::{entity_summary_struct_aspect, AspectSchemaFixture};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, field_key,
        publications::{release_test_commit_snapshot, string_aspect_field_patch},
    },
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn genuine_whole_clear_can_leave_a_live_member_without_stored_payload() {
    use worth_foundational::facade::{
        aspects, AbsenceLaw, AspectBinding, AspectEquivalenceBasis, AspectEvolutionPolicy,
        AspectIdentity, AspectMaskContract, ScalarAspectType,
    };
    let request = serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let contract = aspects()
        .contract()
        .for_key(aspect_key("note"))
        .identified_by(AspectIdentity(991))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar_with(
            ScalarAspectType::String,
            AspectMaskContract::scalar(),
            AbsenceLaw::Optional,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        );
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: field_key("note"),
            },
            contract: contract.clone(),
        }],
        ..AspectSchemaFixture::default()
    };
    let mut runtime = fixture.build_runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let targets = ["cleared", "untouched"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: key.into(),
    });
    let mut batch = WorkerIntentBatch::new("genuine-optional-payload-birth");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: string_aspect_field_patch([(
                aspect_key("note"),
                field_key("note"),
                "original payload",
            )]),
        })));
    }
    let birth = publish(
        &runtime,
        execution,
        &installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("native-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| birth.created_entity(&target).unwrap());
    let previous = birth.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &birth);
    let cleared = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("actual-last-optional-aspect-clear").push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: ids[0],
                aspect_patch: PortableRecordAspectPatch::new([
                    PortableAspectPatchOperation::ClearWhole {
                        basis: PortableAspectContractBasis::from_contract(&contract),
                    },
                ]),
            }),
        )),
        preserve(&owner, previous, ids, 2),
    )
    .unwrap();
    let view = runtime
        .read_truth()
        .project_snapshot(&cleared.snapshot)
        .unwrap();
    view.read_exact_entity_state(execution, ids[0], |metadata, state| {
        assert_eq!(metadata.lifecycle, RecordLifecycleState::Live);
        assert!(state.is_none());
    })
    .unwrap()
    .expect("the cleared entity remains visible");
    view.read_exact_entity_state(execution, ids[1], |_, state| {
        assert!(state.is_some());
    })
    .unwrap()
    .expect("the unpatched entity remains visible");
    release_test_commit_snapshot(&runtime, &cleared);
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-preserver",
            &[2; 32],
            &native_meaning_fixture("native-creator"),
            &native_roles_fixture(
                &ids.map(|id| (id, KindId(1))),
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .unwrap();
}

#[test]
fn cold_succession_applies_partial_field_clear_without_requiring_a_full_rewrite() {
    let request = serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let contract = binding.contract.clone();
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![binding],
        ..AspectSchemaFixture::default()
    };
    let mut runtime = fixture.build_runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let targets = ["partial", "unpatched"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: key.into(),
    });
    let mut batch = WorkerIntentBatch::new("full-two-member-birth");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: string_aspect_field_patch([
                (aspect_key("summary"), field_key("title"), "retained title"),
                (aspect_key("summary"), field_key("status"), "clear this"),
            ]),
        })));
    }
    let birth = publish(
        &runtime,
        execution,
        &installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("native-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| birth.created_entity(&target).unwrap());
    let previous = birth.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &birth);
    let cleared = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("real-optional-field-clear").push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: ids[0],
                aspect_patch: PortableRecordAspectPatch::new([
                    PortableAspectPatchOperation::PatchFields {
                        basis: PortableAspectContractBasis::from_contract(&contract),
                        selected_fields: vec![field_key("status")],
                        field_sets: vec![],
                        field_clears: vec![field_key("status")],
                    },
                ]),
            }),
        )),
        preserve(&owner, previous, ids, 2),
    )
    .unwrap();
    assert_eq!(cleared.changed_records, [RecordRef::Entity(ids[0])]);
    let view = runtime
        .read_truth()
        .project_snapshot(&cleared.snapshot)
        .unwrap();
    for (index, &id) in ids.iter().enumerate() {
        view.read_exact_entity_state(execution, id, |_, state| {
            let payload = state.unwrap().get(contract.key()).unwrap();
            let ContractValidatedAspectValueView::Struct(value) = payload.view() else {
                panic!("struct carrier")
            };
            assert!(value.get(&field_key("title")).is_some());
            assert_eq!(value.get(&field_key("status")).is_some(), index == 1);
        })
        .unwrap()
        .expect("the exact successor member remains visible");
    }
    release_test_commit_snapshot(&runtime, &cleared);
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-preserver",
            &[2; 32],
            &native_meaning_fixture("native-creator"),
            &native_roles_fixture(
                &ids.map(|id| (id, KindId(1))),
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .unwrap();
}
