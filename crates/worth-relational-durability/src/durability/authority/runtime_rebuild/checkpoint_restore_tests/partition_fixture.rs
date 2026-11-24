use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::RecordArenaFixturePort as _;
use super::super::{branch_root_images::RestoredBranchRootImages, partition_images::*};
use super::CheckpointTestRequest;
use crate::durability::checkpoints::{
    aspect_state_images::CheckpointAspectContractCatalog, images::partition_to_image_with_contracts,
};
use crate::durability::data::CheckpointRestoreWork;
use worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::PartitionCheckpointImage;
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::identity::data::primitives::{KindId, PartitionId, VersionId};
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::residency::checkpoint_kind_images::CheckpointRegionKindImages;
use worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::{EntityArena, RelationArena, SlotInit};
use std::collections::BTreeMap;

use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;

pub(super) struct ReadmittedMirrorFixture {
    pub(super) roots: Vec<(CommitId, Vec<PartitionCheckpointImage>)>,
    pub(super) mirror_images: Vec<PartitionCheckpointImage>,
    pub(super) restored: RestoredBranchRootImages,
    kinds: CheckpointRegionKindImages,
}

impl ReadmittedMirrorFixture {
    pub(super) fn restore_mirror(
        &self,
        lane: &ReconstructionLane<'_>,
        request: &CheckpointTestRequest<'_, '_>,
    ) -> Result<RestoredPartitions, RelationalRecoveryStop> {
        let schema = self.restored.schema_authorities.values().next().unwrap();
        let contracts =
            CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())
                .unwrap();
        self.restore_mirror_with_contracts(lane, request, &contracts)
    }

    pub(super) fn restore_mirror_with_contracts(
        &self,
        lane: &ReconstructionLane<'_>,
        request: &CheckpointTestRequest<'_, '_>,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<RestoredPartitions, RelationalRecoveryStop> {
        let schema = self.restored.schema_authorities.values().next().unwrap();
        restore_partition_mirror(
            lane,
            request,
            CheckpointMirrorImages {
                partitions: &self.mirror_images,
                kinds: &self.kinds,
            },
            self.roots
                .iter()
                .map(|(commit_id, partitions)| CheckpointRootPartitions {
                    commit_id: *commit_id,
                    partitions,
                }),
            &self.restored,
            schema.aspect_plans(),
            contracts,
            &mut CheckpointRestoreWork::default(),
        )
    }
}

pub(super) fn readmitted_mirror_fixture(
    lane: &ReconstructionLane<'_>,
    request: &CheckpointTestRequest<'_, '_>,
    partitions: &[PartitionState],
) -> ReadmittedMirrorFixture {
    readmitted_mirror_fixture_with_schema(lane, request, partitions, partition_schema())
}

pub(super) fn readmitted_mirror_fixture_with_schema(
    lane: &ReconstructionLane<'_>,
    request: &CheckpointTestRequest<'_, '_>,
    partitions: &[PartitionState],
    schema: std::sync::Arc<RelationalBranchRootSchemaAuthority>,
) -> ReadmittedMirrorFixture {
    let mut fixture = ReadmittedMirrorFixture {
        roots: Vec::new(),
        mirror_images: Vec::new(),
        restored: RestoredBranchRootImages {
            partitions: BTreeMap::new(),
            schema_authorities: BTreeMap::new(),
        },
        kinds: CheckpointRegionKindImages::capture(lane, request, partitions).unwrap(),
    };
    fixture.readmit_root_with_schema(lane, request, CommitId(1), partitions, schema);
    fixture.mirror_images = fixture.roots[0].1.clone();
    fixture
}

impl ReadmittedMirrorFixture {
    pub(super) fn readmit_root(
        &mut self,
        lane: &ReconstructionLane<'_>,
        request: &CheckpointTestRequest<'_, '_>,
        commit_id: CommitId,
        partitions: &[PartitionState],
    ) {
        self.readmit_root_with_schema(lane, request, commit_id, partitions, partition_schema());
    }

    fn readmit_root_with_schema(
        &mut self,
        lane: &ReconstructionLane<'_>,
        request: &CheckpointTestRequest<'_, '_>,
        commit_id: CommitId,
        partitions: &[PartitionState],
        schema: std::sync::Arc<RelationalBranchRootSchemaAuthority>,
    ) {
        let contracts =
            CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())
                .unwrap();
        let images = partitions
            .iter()
            .cloned()
            .map(|partition| {
                partition_to_image_with_contracts(
                    lane,
                    partition,
                    schema.aspect_plans(),
                    &contracts,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let mut decoded_root = restore_unique_partition_images_with_schema(
            lane,
            &images,
            schema.aspect_plans(),
            &contracts,
            "checkpoint root",
        )
        .unwrap();
        CheckpointRegionKindImages::capture(lane, request, partitions)
            .unwrap()
            .restore(lane, request, &mut decoded_root)
            .unwrap();
        self.roots.push((commit_id, images));
        self.restored.partitions.insert(commit_id, decoded_root);
        self.restored.schema_authorities.insert(commit_id, schema);
    }
}

fn partition_schema(
) -> std::sync::Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>{
    use worth_relational_contracts::internal_port::config::data::{
        CascadeDeletePolicy, CrossContextPolicy,
    };
    use worth_relational_contracts::internal_port::schema::data::{
        EntityKindRegistration, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    };
    let schema_id = SchemaId("checkpoint-fixture".into());
    let schema_version_id = SchemaVersionId(1);
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "entity".into(),
            schema_id: schema_id.clone(),
            schema_version_id,
            aspect_contract_declarations: Default::default(),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "relation".into(),
            schema_id,
            schema_version_id,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: Default::default(),
            relation_integrity: Default::default(),
        })
        .unwrap();
    worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority::capture(
        1,
        &registry,
        &registry.authority_snapshot(),
        worth_relational_contracts::internal_port::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion::default(),
        None,
    )
    .unwrap()
}

pub(super) fn empty_partition(partition_id: PartitionId) -> PartitionState {
    PartitionState {
        partition_id,
        adjacency_policy: worth_relational_contracts::internal_port::config::data::policies::AdjacencyPolicy {
            backend: worth_relational_contracts::internal_port::config::data::policies::AdjacencyBackend::CompressedFanoutAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    }
}

pub(super) fn partition_with_entity(partition_id: PartitionId) -> PartitionState {
    let mut partition = empty_partition(partition_id);
    partition.entity_arena.push_slot_for_test(SlotInit {
        partition_id,
        kind_id: KindId(1),
        version_id: VersionId(1),
        extra: Default::default(),
    });
    partition
}
