use std::collections::BTreeMap;

use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::facade::history::{BranchId, CommitId, RelationalCommitReceipt};
use crate::facade::identity::{EntityId, KindId, LineageId, PartitionId, VersionId};
use crate::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameBirthRequest, PersistentNameHolder, PersistentNameSubjectKind,
};
use crate::storage::overlay::{CurrentPartitionAccess, WorkingState};
use crate::storage::substrate::{EntityRecordKind, SlotInit};

pub(super) fn face_scheme_registry() -> InstalledPersistentNameSchemeRegistry {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    registry
        .register(InstalledPersistentNameScheme::from_ordered_roles(
            "worth.cad.face.v1",
            PersistentNameSubjectKind::Entity,
            ["authored", "cut_piece"],
            true,
            true,
        ))
        .unwrap();
    registry
}

pub(super) fn stage_two_entities() -> (WorkingState, EntityId, EntityId) {
    let mut staged = WorkingState::new(
        BTreeMap::new(),
        AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
    );
    let part = staged.get_partition_mut(PartitionId::main());
    for _ in 0..2 {
        part.entity_arena.push_slot(SlotInit::<EntityRecordKind> {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: Default::default(),
        });
    }
    (
        staged,
        EntityId::new(PartitionId::main(), 0, 1),
        EntityId::new(PartitionId::main(), 1, 1),
    )
}

pub(super) fn face_birth_request(
    occurrence_key: &str,
    role_path: &str,
    discriminator_label: &str,
    entity: EntityId,
) -> PersistentNameBirthRequest {
    let canonical_discriminator = if discriminator_label.is_empty() {
        Vec::new()
    } else {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"worth.query.name-discriminator.v1\0");
        let disc_id = b"worth.test.label.v1";
        bytes.extend_from_slice(&(disc_id.len() as u64).to_be_bytes());
        bytes.extend_from_slice(disc_id);
        bytes.push(0x19); // TAG_TEXT
        bytes.extend_from_slice(&(discriminator_label.len() as u64).to_be_bytes());
        bytes.extend_from_slice(discriminator_label.as_bytes());
        bytes
    };
    PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: occurrence_key.to_owned(),
        role_path: role_path.to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator,
        holder: PersistentNameHolder::Entity(entity),
    }
}

pub(super) fn commit_receipt() -> RelationalCommitReceipt {
    RelationalCommitReceipt {
        commit_id: CommitId(1),
        version_id: VersionId(1),
        branch_id: BranchId("main".to_owned()),
        parents: Vec::new(),
    }
}

pub(super) fn staged_entity_lineage_id(
    state: &WorkingState,
    entity: EntityId,
) -> Option<LineageId> {
    state
        .get_current_partition(PartitionId::main())
        .unwrap()
        .entity_arena()
        .get(&entity)
        .unwrap()
        .extra()
        .lineage_id
}

pub(super) fn create_postures_input(
    scheme_registry: InstalledPersistentNameSchemeRegistry,
    births: Vec<(EntityId, PersistentNameBirthRequest)>,
) -> crate::facade::lineage::LineageCommitPostureInput {
    use crate::facade::lineage::{
        LineageCommitPostureInput, LineagePostureClaim, LineageRecordTarget,
    };

    LineageCommitPostureInput {
        scheme_registry,
        live_holders: BTreeMap::new(),
        live_lineages: BTreeMap::new(),
        retired_lineages: BTreeMap::new(),
        postures: births
            .into_iter()
            .map(|(entity, birth)| LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(entity),
                birth,
            })
            .collect(),
    }
}
