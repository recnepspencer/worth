#[cfg(test)]
use crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test;
use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeExecutionOutcome, MergeExecutionRequest,
    MergeIntent,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::merge::data::PreparedMergeExecution;
use crate::tests::support::{
    aspect_key, create_branch_from_main, create_entity, entity_field_aspect, field_key,
    string_aspect_field_patch, CascadeDeletePolicy, CrossContextPolicy,
};
use crate::transactions::data::PublishedMergeExecutionAuthority;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommitFixturePort as _;
use worth_relational_durability::internal_port::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactOwnerPort as _;

pub(super) fn runtime_with_collaboration_merge_history() -> RelationalRuntime {
    let name_key = aspect_key("name");
    let status_key = aspect_key("status");
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), field_key("name")),
                entity_field_aspect(status_key.clone(), field_key("status")),
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(
                    std::sync::Arc::<[worth_foundational::facade::AspectKey]>::from([name_key])
                        .into(),
                ),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key,
                policy: AspectMergePolicyKind::PreferRicher,
            }]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("phase 13 schema registry");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration")
}

pub(super) fn execute_feature_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> MergeExecutionOutcome {
    let prepared = prepared_feature_merge(execution, runtime);
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed feature merge")
}

pub(super) fn prepared_feature_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> PreparedMergeExecution {
    install_merge_scenario(execution, runtime);
    runtime
        .merge()
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("prepared feature merge")
}

pub(super) fn alternate_branch_basis(
    runtime: &RelationalRuntime,
) -> crate::history::data::RelationalMergeBranchBasis {
    runtime
        .history()
        .resolve_merge_branch_basis(&BranchId("main".to_string()), &BranchId("alt".to_string()))
        .expect("alternate branch basis")
}

#[derive(Debug)]
struct EmptyColdSource;
impl crate::durability::medium::RelationalDurableColdSource for EmptyColdSource {
    fn retain_members(
        &self,
        members: &[crate::durability::medium::RelationalDurableMemberLocator],
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        if members.is_empty() {
            Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
        } else {
            Err(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)
        }
    }
    fn read_member(
        &self,
        _: crate::durability::medium::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        Err(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        Err(crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable)
    }
}

pub(super) fn recover_with_rewritten_commit_err(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source_runtime: &RelationalRuntime,
    commit_id: CommitId,
    edit: impl FnOnce(&mut crate::history::data::CanonicalCommitEnvelopeBody),
) -> crate::durability::data::DurabilityError {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
    use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
    use crate::durability::published_medium::RelationalDurableRecoverySource;
    use crate::history::data::PositionedCanonicalCommit;
    use std::sync::Arc;

    let mut edit = Some(edit);
    let tail: Vec<_> = source_runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .map(|artifact| {
            let artifact = artifact.unwrap();
            if artifact.commit_id() == commit_id {
                let readmitted =
                    RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution)
                        .expect("original merge artifact readmits before rewrite");
                let mut envelope = copy_original_for_test(readmitted.envelope(), execution);
                edit.take().expect("one matching commit")(envelope.body_mut_for_test());
                RelationalCanonicalCommitArtifact::from_positioned_for_test(
                    &PositionedCanonicalCommit::positioned_commit_for_test(
                        artifact.position(),
                        Arc::new(envelope),
                    ),
                    execution.allocation_policy(),
                )
                .expect("rewritten envelope encodes")
            } else {
                RelationalCanonicalCommitArtifact::from_untrusted_bytes(
                    artifact.as_bytes().to_vec(),
                )
                .unwrap()
            }
        })
        .collect();
    assert!(
        edit.is_none(),
        "expected merge commit {commit_id:?} in tail"
    );
    let transitions: Vec<_> = source_runtime
        .durability()
        .branch_transition_artifacts(execution.allocation_policy())
        .map(Result::unwrap)
        .collect();
    let source =
        RelationalDurableRecoverySource::new(None, tail, transitions, Arc::new(EmptyColdSource));
    let mut recovered = runtime_with_collaboration_merge_history();
    let plan = recovered
        .durability()
        .recovery_plan_from_source(execution, source, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    crate::tests::support::expect_durability_failure(error)
}

pub(super) fn published_merge_authority(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit_id: CommitId,
) -> PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}

fn install_merge_scenario(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) {
    if runtime
        .history()
        .branch_head(&BranchId("feature".to_string()))
        .is_some()
    {
        return;
    }
    let shared = create_entity(execution, runtime, "shared");
    create_branch_from_main(runtime, "feature");
    create_branch_from_main(runtime, "alt");
    update_entity_status_on_branch(execution, runtime, shared, "inactive", "main");
    update_entity_status_on_branch(execution, runtime, shared, "active", "feature");
    update_entity_status_on_branch(execution, runtime, shared, "pending", "alt");
}

fn update_entity_status_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    status: &str,
    branch: &str,
) {
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_branch(
                runtime,
                BranchId(branch.to_string()),
            );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        crate::facade::transactions::WorkerIntentBatch::new(format!("status-{branch}")).push(
            crate::facade::transactions::MutationIntent::Entity(
                crate::facade::transactions::EntityMutationIntent::UpdateFields(
                    crate::facade::transactions::UpdateEntityFieldsIntent {
                        entity_id,
                        fields: string_aspect_field_patch([(
                            aspect_key("status"),
                            field_key("status"),
                            status,
                        )]),
                    },
                ),
            ),
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, runtime)
        .expect("update entity status on branch");
}
