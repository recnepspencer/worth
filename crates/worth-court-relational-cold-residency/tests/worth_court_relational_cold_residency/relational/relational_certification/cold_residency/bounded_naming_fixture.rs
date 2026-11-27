//! Shared persistent-naming and persistent-group fixture for `P0-A` / `#440`
//! cold-residency courts.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::config::{
    CascadeDeletePolicy, RelationalRuntimeProfile, ResidencyBudget,
};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableMemberLocator, RelationalDurableRecoverySource,
    RelationalNativeCheckpoint,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    LineageRecordTarget, PendingBirthTarget, PendingCommitBirth, PersistentNameIdentity,
    PersistentNameKey, PersistentNameOfRecord, PersistentNameSubjectKind,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::transactions::{
    CommitResult, CreateIntent, CreatedEntityRef, DeleteEntityIntent, EntityMutationIntent,
    EntitySpec, MutationIntent, RecordRef, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
use worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

pub(super) const ENTITY_SCHEME: &str = "worth.m43f.entity.v1";
pub(super) const GROUP_SCHEME: &str = "worth.m43f.group.v1";

pub(super) fn named_schema() -> RelationalSchemaRegistry {
    naming_schema(false)
}

/// The named schema, its entity scheme minting structural identities when
/// `structural_identity`.
pub(super) fn naming_schema(structural_identity: bool) -> RelationalSchemaRegistry {
    let base = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_registry();
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    schemes
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                ENTITY_SCHEME,
                PersistentNameSubjectKind::Entity,
                "test.entity",
                ["authored"],
                false,
                false,
                structural_identity,
            )
            .unwrap(),
        )
        .unwrap();
    base.install_persistent_name_schemes(
        schemes,
        BTreeMap::from([(
            (PersistentNameSubjectKind::Entity, KindId(1)),
            ENTITY_SCHEME.into(),
        )]),
    )
    .unwrap()
}

pub(super) fn group_scheme() -> AggregateGroupScheme {
    AggregateGroupScheme::new(
        AggregateGroupSchemeId::new(GROUP_SCHEME).unwrap(),
        AggregateContinuityRule::Anchor,
    )
}

pub(super) fn durable_named_runtime(
    medium: &Arc<TestDurableMedium>,
    historical_budget: u64,
) -> RelationalRuntime {
    durable_runtime_with(medium, historical_budget, named_schema())
}

pub(super) fn durable_runtime_with(
    medium: &Arc<TestDurableMedium>,
    historical_budget: u64,
    schema: RelationalSchemaRegistry,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema)
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: u64::MAX / 4,
            historical_working_set_bytes: historical_budget,
            max_in_flight_fill_bytes: u64::MAX / 4,
            streaming_window_bytes: 64 * 1024,
        })
        .build_durable(medium.attachment())
        .unwrap()
}

pub(super) fn reopen_named(
    request: &ExecutionRequest<'_, '_>,
    checkpoint: RelationalNativeCheckpoint,
    source: Arc<ImmutableCheckpointMemberSource>,
    medium: &Arc<TestDurableMedium>,
    historical_budget: u64,
) -> RelationalRuntime {
    reopen_with(
        request,
        checkpoint,
        source,
        durable_named_runtime(medium, historical_budget),
    )
}

/// Recovers `checkpoint` cold into the fresh `runtime`.
pub(super) fn reopen_with(
    request: &ExecutionRequest<'_, '_>,
    checkpoint: RelationalNativeCheckpoint,
    source: Arc<ImmutableCheckpointMemberSource>,
    mut runtime: RelationalRuntime,
) -> RelationalRuntime {
    let input = RelationalDurableRecoverySource::new(
        Some(
            worth_relational::facade::durability::RelationalDurableCheckpointSource::new(
                checkpoint.covered_position().unwrap(),
                checkpoint.covered_branch_transitions().unwrap(),
                checkpoint,
            ),
        ),
        Vec::new(),
        Vec::new(),
        source,
    );
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
    runtime
}

pub(super) fn observe_branch(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> AdmittedRelationalBranchBasis {
    let branch_identity = if branch == runtime.main_branch_identity().branch_id() {
        runtime.main_branch_identity()
    } else {
        runtime.branch_identity(branch).unwrap()
    };
    runtime.observe_branch(&branch_identity).unwrap().1
}

pub(super) fn create_named_entities(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    keys: &[String],
) -> (CommitResult, Vec<EntityId>, Vec<PersistentNameIdentity>) {
    let basis = observe_branch(runtime, branch);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut batch = WorkerIntentBatch::new("create-named");
    let mut plan = CommitNamingPlan::default();
    for key in keys {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: key.as_str().into(),
            fields: field_patch("name", "name", key),
        })));
        plan.pending_births.push(PendingCommitBirth {
            target: PendingBirthTarget::CreatedEntity(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: key.as_str().into(),
            }),
            scheme_id: ENTITY_SCHEME.to_owned(),
            occurrence_key: format!("occ.{key}"),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
        });
    }
    tx.push_batch(batch, request.allocation_policy()).unwrap();
    tx.attach_naming_plan(plan);
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
    let entities: Vec<EntityId> = committed
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .collect();
    let after = observe_branch(runtime, branch);
    let names = runtime
        .with_persistent_names_at_observation(request, &after.observation(), |view| {
            entities
                .iter()
                .map(
                    |entity| match view.name_of_record(LineageRecordTarget::Entity(*entity)) {
                        PersistentNameOfRecord::Live {
                            name: PersistentNameKey::Entity(id),
                            ..
                        } => id,
                        other => panic!("expected live entity name, got {other:?}"),
                    },
                )
                .collect::<Vec<_>>()
        })
        .unwrap();
    (committed, entities, names)
}

pub(super) fn publish_group_partition(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    touch_entity: EntityId,
    label: &str,
    groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>>,
) -> CommitResult {
    let basis = observe_branch(runtime, branch);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new(label).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: touch_entity,
                fields: field_patch("name", "name", label),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(group_scheme(), Vec::new(), groups);
    tx.attach_naming_plan(plan);
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
    committed
}

pub(super) fn delete_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity_id: EntityId,
) -> CommitResult {
    let basis = observe_branch(runtime, branch);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
    committed
}

pub(super) fn naming_pages(
    source: &ImmutableCheckpointMemberSource,
) -> Vec<RelationalDurableMemberLocator> {
    source
        .members
        .iter()
        .filter(|(_, bytes)| {
            bytes.bytes().starts_with(b"WORTH-REL-CHUNK1\n") && bytes.bytes().get(17) == Some(&5)
        })
        .map(|(locator, _)| *locator)
        .collect()
}

pub(super) fn naming_page_reads(source: &ImmutableCheckpointMemberSource) -> usize {
    source.naming_page_reads.lock().unwrap().len()
}

pub(super) fn commit_page_reads(source: &ImmutableCheckpointMemberSource) -> usize {
    source.commit_page_reads.lock().unwrap().len()
}

pub(super) fn lineage_page_reads(
    source: &ImmutableCheckpointMemberSource,
    directory: &super::selected_directory::Directory,
) -> usize {
    let lineage_members: BTreeSet<_> = directory
        .lineage
        .iter()
        .map(|page| page.member)
        .chain(directory.lineage_events.iter().map(|page| page.member))
        .collect();
    source
        .member_reads
        .lock()
        .unwrap()
        .iter()
        .filter(|locator| lineage_members.contains(locator))
        .count()
}
