use std::sync::Arc;

use super::{RelationalBranchRoot, RelationalBranchRootCaptureDenial};
use crate::config::data::CascadeDeletePolicy;
use crate::history::data::BranchId;
use crate::schema::data::SchemaVersionId;
use crate::tests::support::*;

#[path = "root_authoritative_payload_tests/word_codec.rs"]
mod word_codec;

#[test]
fn content_old_retained_contract_cannot_hide_a_reference_after_current_binding_is_removed() {
    use crate::{
        content::*, mvcc::RelationalTransactionIntent, schema::data::*, transactions::data::*,
    };
    use worth_foundational::facade::*;
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let format = CanonicalDigestId::new([3; 32]);
    let contract = AspectContract::struct_aspect(
        AspectKey::new("word").unwrap(),
        AspectIdentity(741),
        AspectContractRevision(1),
        StructAspectShape::new([FieldDeclaration::new(
            FieldKey::new("value").unwrap(),
            ScalarAspectType::ContentRef,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
        .with_codec(format)])
        .unwrap(),
    );
    let schema = AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new("word").unwrap(),
            },
            contract: contract.clone(),
        }],
        ..Default::default()
    }
    .build_registry()
    .register_content_format(
        format,
        Arc::new(word_codec::WordCodec(
            ContentFormat::new("word", 1).unwrap(),
        )),
    )
    .unwrap()
    .bind_content_field(format, format)
    .unwrap();
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let identity = transaction
        .admit_content(&execution, format, &17_i64)
        .unwrap();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Struct(
            StructAspectValue::new([(
                FieldKey::new("value").unwrap(),
                AspectValue::ContentRef(identity),
            )])
            .unwrap(),
        ),
    }]);
    transaction
        .push_batch(
            WorkerIntentBatch::new("word").push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: crate::identity::data::PartitionId::main(),
                    kind_id: crate::identity::data::KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("word"),
                    aspect_patch: patch,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(&execution, &runtime).unwrap();
    let (root, mut envelope) =
        committed_root_and_envelope(&execution, &runtime, committed.commit.commit_id);
    let mut changed = root.schema_authority().registry().clone();
    changed
        .entity_kinds
        .get_mut(&crate::identity::data::KindId(1))
        .unwrap()
        .aspect_contract_declarations = KindAspectContractDeclarations::default();
    envelope.body_mut_for_test().schema_authority = changed.authority_snapshot();
    let authority = super::RelationalBranchRootSchemaAuthority::readmit_exact(
        2,
        changed,
        root.schema_authority().retained_aspect_contracts().to_vec(),
        &envelope.schema_authority,
        envelope.descriptor_semantics_version,
        crate::schema::SchemaCompilationControl::Configuration,
    )
    .expect("configuration compilation cannot refuse")
    .unwrap();
    let descriptor = super::RelationalBranchRootDescriptor::new(
        *root.descriptor().unwrap().truth_root(),
        authority.registry().authority_digest_bytes(),
    );
    let partitions = root
        .partition_ids_iter()
        .map(|id| {
            (
                id,
                root.resident_full_region_witness(id)
                    .unwrap()
                    .unwrap()
                    .full_view()
                    .partition()
                    .clone(),
            )
        })
        .collect();
    let refusal = RelationalBranchRoot::readmit(
        &super::RelationalBranchRootIdentityIssuer::default(),
        &execution,
        &RelationalContentAttachments::default(),
        partitions,
        crate::branch::RelationalOriginalRootCommitEvidence::prepare(
            &execution,
            Arc::new(envelope),
        )
        .unwrap(),
        descriptor,
        authority,
        &runtime.services.symbols.interner_snapshot(),
    )
    .unwrap_err();
    assert!(
        matches!(
            refusal,
            RelationalBranchRootCaptureDenial::Content(
                RelationalContentDenial::FormatBindingMissing
            )
        ),
        "{refusal:?}"
    );
}

#[test]
fn strategy_artifacts_are_bound_into_the_committed_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = crate::commit_strategies::facade::intent_strategy_runtime();
    let entity = create_entity(execution, &runtime, "strategy-before");
    let outcome = crate::commit_strategies::facade::execute_persisted_intent_strategy_commit(
        execution,
        &mut runtime,
        entity,
    );
    let (root, mut envelope) =
        committed_root_and_envelope(execution, &runtime, outcome.commit.commit_id);
    assert!(envelope
        .body_mut_for_test()
        .strategy_artifacts
        .take()
        .is_some());

    assert_axis_mutation_rejected(execution, &runtime, &root, envelope, "strategy artifacts");
}

#[test]
fn merge_execution_authority_is_bound_into_the_committed_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "merge-main");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_owned()),
            &BranchId("main".to_owned()),
        )
        .expect("feature branch forks");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "merge-feature",
        BranchId("feature".to_owned()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            crate::merge::data::MergeExecutionRequest::new(
                BranchId("main".to_owned()),
                BranchId("feature".to_owned()),
                crate::merge::data::MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("governed merge prepares");
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("governed merge executes");
    let (root, mut envelope) =
        committed_root_and_envelope(execution, &runtime, outcome.commit.commit.commit_id);
    assert!(envelope
        .body_mut_for_test()
        .merge_execution_authority
        .take()
        .is_some());

    assert_axis_mutation_rejected(
        execution,
        &runtime,
        &root,
        envelope,
        "merge execution authority",
    );
}

#[test]
fn schema_transition_descriptors_are_each_bound_into_the_committed_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "schema-before");
    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let input = test_owner_transaction_validation_input_for_main(&runtime).with_schema_transition(
        schema_transition(),
        Some(crate::schema::data::SchemaReconciliationPolicy::PreserveInformation),
    );
    let mut transaction = runtime
        .begin_branch_transaction(input.basis(), input.intent().clone())
        .expect("schema transition transaction binds");
    transaction
        .push_batch(
            batch_create("schema-after"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let outcome = transaction
        .commit(execution, &runtime)
        .expect("schema transition commits");
    let (root, envelope) =
        committed_root_and_envelope(execution, &runtime, outcome.commit.commit_id);

    let mut transition = envelope.copy_original_for_test(execution);
    assert!(transition
        .body_mut_for_test()
        .schema_transition
        .take()
        .is_some());
    assert_axis_mutation_rejected(execution, &runtime, &root, transition, "schema transition");
    let mut continuation = envelope.copy_original_for_test(execution);
    assert!(continuation
        .body_mut_for_test()
        .schema_continuation_descriptor
        .take()
        .is_some());
    assert_axis_mutation_rejected(
        execution,
        &runtime,
        &root,
        continuation,
        "continuation descriptor",
    );
    let mut reconciliation = envelope;
    assert!(reconciliation
        .body_mut_for_test()
        .schema_reconciliation_descriptor
        .take()
        .is_some());
    assert_axis_mutation_rejected(
        execution,
        &runtime,
        &root,
        reconciliation,
        "reconciliation descriptor",
    );
}

fn committed_root_and_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    commit_id: crate::history::data::CommitId,
) -> (
    Arc<RelationalBranchRoot>,
    crate::history::data::CanonicalCommitEnvelope,
) {
    let root = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("commit installs one complete main root");
    let envelope = root
        .canonical_envelope()
        .expect("root carries canonical envelope");
    assert_eq!(envelope.commit.commit_id, commit_id);
    (
        Arc::clone(&root),
        envelope.copy_original_for_test(execution),
    )
}

fn assert_axis_mutation_rejected(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    root: &Arc<RelationalBranchRoot>,
    mutant: crate::history::data::CanonicalCommitEnvelope,
    axis: &str,
) {
    let denial = root
        .relink_canonical_envelope(
            execution,
            crate::branch::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::new(mutant),
            )
            .unwrap(),
            &runtime.services.symbols.interner_snapshot(),
        )
        .expect_err("authoritative payload mutation cannot relink");
    assert!(
        matches!(
            denial,
            RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch { .. }
        ),
        "{axis} mutation must change the committed root"
    );
}

fn schema_transition() -> crate::schema::data::ProposedSchemaTransition {
    use crate::schema::data::*;
    ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_owned()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_owned()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_owned()),
                SchemaVersionId(2),
                Some(crate::identity::data::KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: crate::tests::support::field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(
            SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}
