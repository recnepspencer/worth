use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    ContractValidationInput, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableRecordAspectPatch,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    definition::{aspect_key, field_key, optional_string_contract},
    publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
        string_aspect_field_patch, text,
    },
};

#[test]
fn plan_revision_commits_to_group_id_membership_and_requirement_in_canonical_order() {
    let format = group("format", &["text", "csv"], true);
    let other = group("singleton", &["json"], false);
    let canonical = revision(vec![format.clone(), other.clone()]);
    assert_eq!(canonical, revision(vec![other.clone(), format.clone()]));
    assert_eq!(
        canonical,
        revision(vec![group("format", &["csv", "text"], true), other.clone()])
    );
    for changed in [
        group("format", &["text", "csv"], false),
        group("encoding", &["text", "csv"], true),
        group("format", &["csv", "json"], true),
    ] {
        assert_ne!(canonical, revision(vec![changed, other.clone()]));
    }
    assert_ne!(canonical, revision(vec![format]));
}

#[test]
fn declaration_rejects_empty_unknown_and_duplicate_group_terms() {
    for declarations in [
        vec![group(" ", &["text"], true)],
        vec![group("format", &[], true)],
        vec![group("format", &["text", "unknown"], false)],
        vec![
            group("format", &["text"], true),
            group("format", &["csv"], false),
        ],
    ] {
        assert!(registry(declarations).is_err());
    }
    registry(vec![group("format", &["text"], true)]).unwrap();
}

#[test]
fn required_group_refuses_raw_birth_activation_and_removal_and_accepts_a_swap() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime(true);
    let initial = head(&runtime);
    for (key, cases, count) in [("absent", &[][..], 0), ("both", &["text", "csv"][..], 2)] {
        assert_refusal(
            commit(&execution, &runtime, create(key, cases)),
            true,
            count,
        );
        assert_eq!(head(&runtime), initial);
    }
    let seeded = commit(&execution, &runtime, create("one", &["text"])).unwrap();
    let entity = changed_entities(&seeded)[0];
    let before = head(&runtime);
    let activate_csv = PortableAspectPatchOperation::SetWhole {
        basis: basis("csv"),
        value: ContractValidationInput::Scalar(text("csv")),
    };
    let clear_text = PortableAspectPatchOperation::ClearWhole {
        basis: basis("text"),
    };
    for (intent, count) in [
        (update(entity, "csv"), 2),
        (patch(entity, [activate_csv.clone()]), 2),
        (patch(entity, [clear_text.clone()]), 0),
        (
            MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: entity,
                replacement: entity_spec("empty-replacement", &[]),
            })),
            0,
        ),
    ] {
        assert_refusal(commit(&execution, &runtime, intent), true, count);
        assert_eq!(head(&runtime), before);
    }
    let swapped = commit(
        &execution,
        &runtime,
        patch(entity, [clear_text, activate_csv]),
    )
    .unwrap();
    assert_ne!(head(&runtime), before);
    let view = runtime
        .read_truth()
        .project_snapshot(&swapped.snapshot)
        .unwrap();
    for (case, expected) in [("text", None), ("csv", Some(text("csv")))] {
        let scope = worth_relational::facade::runtime::ProjectionAspectScope::fields(
            aspect_key(case),
            [field_key(case)],
        );
        let actual = view
            .entity_record_with_projection_scope(&execution, entity, scope, |record| {
                record
                    .observe_aspect_field(&aspect_key(case), &field_key(case))
                    .and_then(|observed| observed.value().cloned())
            })
            .unwrap();
        assert_eq!(actual, expected);
    }
    let deleted = commit(
        &execution,
        &runtime,
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id: entity,
        })),
    )
    .unwrap();
    for result in [&seeded, &swapped, &deleted] {
        release_test_commit_snapshot(&runtime, result);
    }
}

#[test]
fn optional_group_accepts_zero_and_one_and_refuses_two() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime(false);
    let seeded = commit(&execution, &runtime, create("zero", &[])).unwrap();
    let entity = changed_entities(&seeded)[0];
    let updated = commit(&execution, &runtime, update(entity, "text")).unwrap();
    let before = head(&runtime);
    assert_refusal(
        commit(&execution, &runtime, update(entity, "csv")),
        false,
        2,
    );
    assert_eq!(head(&runtime), before);
    let cleared = commit(
        &execution,
        &runtime,
        patch(
            entity,
            [PortableAspectPatchOperation::ClearWhole {
                basis: basis("text"),
            }],
        ),
    )
    .unwrap();
    for result in [&seeded, &updated, &cleared] {
        release_test_commit_snapshot(&runtime, result);
    }
}

fn group(id: &str, cases: &[&str], required: bool) -> AspectExclusionGroupDeclaration {
    AspectExclusionGroupDeclaration {
        group_id: id.into(),
        aspects: cases.iter().map(|case| aspect_key(case)).collect(),
        required,
    }
}

fn registry(
    groups: Vec<AspectExclusionGroupDeclaration>,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    RelationalSchemaRegistry::new().register_entity_kind(EntityKindRegistration {
        kind_id: KindId(1),
        kind_name: "document".into(),
        schema_id: SchemaId("exclusion".into()),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: KindAspectContractDeclarations::new(
            ["text", "csv", "json"]
                .into_iter()
                .map(|case| DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
                        field: field_key(case),
                    },
                    contract: optional_string_contract(case),
                })
                .collect(),
        )
        .with_exclusion_groups(groups),
    })
}

fn revision(groups: Vec<AspectExclusionGroupDeclaration>) -> AspectContractPlanRevision {
    registry(groups)
        .unwrap()
        .entity_registration(KindId(1))
        .unwrap()
        .aspect_contract_declarations
        .plan_revision
}

fn basis(case: &str) -> PortableAspectContractBasis {
    PortableAspectContractBasis::from_contract(&optional_string_contract(case))
}

fn runtime(required: bool) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(registry(vec![group("format", &["text", "csv"], required)]).unwrap())
        .build()
        .unwrap()
}

fn head(runtime: &RelationalRuntime) -> worth_relational::facade::branch::RelationalBranchVersion {
    runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap()
        .truth_version()
}

fn create(key: &str, cases: &[&str]) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(entity_spec(key, cases)))
}

fn entity_spec(key: &str, cases: &[&str]) -> EntitySpec {
    EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: key.into(),
        fields: string_aspect_field_patch(
            cases
                .iter()
                .map(|case| (aspect_key(case), field_key(case), *case)),
        ),
    }
}

fn update(entity_id: EntityId, case: &str) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id,
            fields: string_aspect_field_patch([(aspect_key(case), field_key(case), case)]),
        },
    ))
}

fn patch(
    entity_id: EntityId,
    operations: impl IntoIterator<Item = PortableAspectPatchOperation>,
) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id,
            aspect_patch: PortableRecordAspectPatch::new(operations),
        },
    ))
}

fn commit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    intent: MutationIntent,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("exclusive-write").push(intent),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(execution, runtime)
}

fn assert_refusal(
    outcome: Result<CommitResult, TransactionCommitError>,
    required: bool,
    count: usize,
) {
    let TransactionCommitError::Conflict { error, .. } = outcome.unwrap_err() else {
        panic!("expected transaction validation conflict");
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::EntityAspectExclusion {
                kind_id: KindId(1),
                group_id,
                required: actual_required,
                active_count,
                ..
            },
        ..
    } = error.class
    else {
        panic!(
            "expected typed aspect exclusion evidence: {:?}",
            error.class
        );
    };
    assert_eq!(
        (group_id.as_str(), actual_required, active_count),
        ("format", required, count)
    );
}
