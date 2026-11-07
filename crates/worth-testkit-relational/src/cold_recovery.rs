//! Real Native checkpoint recovery over immutable externally owned members.
use crate::aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot};
use crate::durable_medium::{
    immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium,
};
use crate::durable_publication::{create_entity_outcome, durable_runtime, field_patch};
use sha2::{Digest, Sha256};
use std::sync::{atomic::AtomicBool, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
        RelationalDurableCheckpointSource, RelationalDurableMemberLocator,
        RelationalDurableRecoverySource,
    },
    identity::EntityId,
    runtime::RelationalRuntime,
    transactions::{
        EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

pub struct ColdRecoveryFixture {
    pub runtime: RelationalRuntime,
    pub medium: Arc<TestDurableMedium>,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub entity: EntityId,
}

pub fn update_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    value: &str,
) {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(value).push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: field_patch("name", "name", value),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

pub fn recovered_with_history(
    request: &ExecutionRequest<'_, '_>,
    updates: usize,
) -> ColdRecoveryFixture {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let created = create_entity_outcome(request, &producer, "member-before");
    let entity = crate::aspect_storage::publications::changed_entities(&created)[0];
    release_test_commit_snapshot(&producer, &created);
    for update in 0..updates {
        update_entity(
            request,
            &producer,
            entity,
            &format!("member-update-{update}"),
        );
    }
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let artifacts = medium.offered();
    let source = Arc::new(ImmutableCheckpointMemberSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { .. } => panic!("first capture owns members"),
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|artifact| (artifact.position(), artifact.as_bytes().to_vec()))
            .collect(),
        canonical_members: artifacts
            .iter()
            .map(|artifact| {
                (
                    RelationalDurableMemberLocator {
                        sha256: Sha256::digest(artifact.as_bytes()).into(),
                        length: artifact.as_bytes().len() as u64,
                    },
                    artifact.position(),
                )
            })
            .collect(),
        damaged: AtomicBool::new(false),
        reads: Default::default(),
        canonical_member_reads: Default::default(),
        journal_reads: Default::default(),
        member_reads: Default::default(),
        commit_reads: Default::default(),
        commits_damaged: Default::default(),
        failed_member: Default::default(),
        failures: Default::default(),
    });
    let checkpoint = RelationalDurableCheckpointSource::new(
        native.covered_position().unwrap(),
        native.covered_branch_transitions().unwrap(),
        native,
    );
    let input =
        RelationalDurableRecoverySource::new(Some(checkpoint), vec![], vec![], source.clone());
    drop(producer);
    let mut runtime = durable_runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    ColdRecoveryFixture {
        runtime,
        medium,
        source,
        entity,
    }
}
