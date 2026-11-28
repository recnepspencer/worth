//! Public transaction and pin setup for the automatic aggregate court.
use super::super::bounded_naming_fixture::observe_branch;
use super::schema::{self, CASES, COUPLING, FACE, FACE_NAME, INCIDENCE, INCIDENCE_NAME, VERTEX};
use worth_execution::ExecutionRequest;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use worth_relational::facade::lineage::{CommitNamingPlan, PendingBirthTarget, PendingCommitBirth};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::{
    PinnedAggregateRead, PinnedAggregateReadScope, RelationalRuntime,
};
use worth_relational::facade::schema::{NativeMemberKey, ProjectedMemberCoordinate};
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::transactions::{
    CommitResult, CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityReference, EntitySpec,
    MutationIntent, RelationSpec, WorkerIntentBatch,
};

pub(super) fn created(kind_id: KindId, key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id,
        client_key: key.into(),
    }
}

pub(super) fn create(reference: &CreatedEntityRef) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: reference.partition_id,
        kind_id: reference.kind_id,
        client_key: reference.client_key.clone(),
        fields: Default::default(),
    }))
}

pub(super) fn relation(
    kind_id: KindId,
    key: &str,
    source: EntityReference,
    target: EntityReference,
) -> CreatedRelationRef {
    CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id,
        client_key: key.into(),
        source,
        target,
    }
}

pub(super) fn connect(reference: &CreatedRelationRef) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: reference.partition_id,
        kind_id: reference.kind_id,
        client_key: reference.client_key.clone(),
        source: reference.source.clone(),
        target: reference.target.clone(),
        fields: Default::default(),
    }))
}

fn birth(target: PendingBirthTarget, scheme: &str, occurrence: &str) -> PendingCommitBirth {
    PendingCommitBirth {
        target,
        scheme_id: scheme.into(),
        occurrence_key: occurrence.into(),
        role_path: "authored".into(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
    }
}

pub(super) fn commit(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    batch: WorkerIntentBatch,
    names: Vec<PendingCommitBirth>,
) -> CommitResult {
    let basis = observe_branch(runtime, runtime.main_branch_identity().branch_id());
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(batch, request.allocation_policy()).unwrap();
    if !names.is_empty() {
        tx.attach_naming_plan(CommitNamingPlan {
            pending_births: names,
            ..Default::default()
        });
    }
    tx.commit(request, runtime).unwrap()
}

pub(super) struct Born {
    pub(super) commit: CommitResult,
    pub(super) face: EntityId,
    pub(super) vertex: EntityId,
    pub(super) coupling: RelationId,
    pub(super) incidence: RelationId,
}

pub(super) fn start(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    projected: bool,
    named: bool,
) -> Born {
    let face = created(FACE, "face");
    let vertex = created(VERTEX, "vertex");
    let face_ref = EntityReference::Created(face.clone());
    let vertex_ref = EntityReference::Created(vertex.clone());
    let coupling = relation(
        COUPLING,
        "coupling",
        face_ref.clone(),
        if projected {
            vertex_ref.clone()
        } else {
            face_ref.clone()
        },
    );
    let incidence = relation(INCIDENCE, "incidence", face_ref, vertex_ref);
    let batch = WorkerIntentBatch::new("birth")
        .push(create(&face))
        .push(create(&vertex))
        .push(connect(&coupling))
        .push(connect(&incidence));
    let names = if named {
        vec![
            birth(
                PendingBirthTarget::CreatedEntity(face.clone()),
                FACE_NAME,
                "face.0",
            ),
            birth(
                PendingBirthTarget::CreatedRelation(incidence.clone()),
                INCIDENCE_NAME,
                "incidence.0",
            ),
        ]
    } else {
        Vec::new()
    };
    let commit = commit(request, runtime, batch, names);
    Born {
        face: commit.created_entity(&face).unwrap(),
        vertex: commit.created_entity(&vertex).unwrap(),
        coupling: commit.created_relation(&coupling).unwrap(),
        incidence: commit.created_relation(&incidence).unwrap(),
        commit,
    }
}

pub(super) fn scope(
    face: EntityId,
    side: usize,
) -> PinnedAggregateReadScope<'static, NativeMemberKey> {
    PinnedAggregateReadScope::for_projected_definition(
        schema::descriptor(true).definition,
        ProjectedMemberCoordinate {
            domain: schema::domain(),
            carrier: face,
            case: CASES[side].into(),
        },
    )
}

pub(super) fn read(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    face: EntityId,
    side: usize,
) -> PinnedAggregateRead<NativeMemberKey> {
    runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], request)
        .unwrap()
        .read_projected_aggregate(request, scope(face, side))
        .unwrap()
}
