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
    discriminator: &[u8],
    entity: EntityId,
) -> PersistentNameBirthRequest {
    PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: occurrence_key.to_owned(),
        role_path: role_path.to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: discriminator.to_vec(),
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
