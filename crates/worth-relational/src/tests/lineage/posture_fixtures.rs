use std::collections::BTreeMap;

use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::facade::history::{BranchId, CommitId, RelationalCommitReceipt};
use crate::facade::identity::{EntityId, KindId, LineageId, PartitionId, RelationId, VersionId};
use crate::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameBirthRequest, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameSubjectKind,
};
use crate::storage::overlay::{CurrentPartitionAccess, WorkingState};
use crate::storage::substrate::{
    EntityRecordKind, RelationEndpoints, RelationExtra, RelationRecordKind, SlotInit,
};

pub(super) const FACE_SCHEME_ID: &str = "worth.cad.face.v1";
pub(super) const SEAM_SCHEME_ID: &str = "worth.cad.seam.v1";

pub(super) fn cad_scheme_registry() -> InstalledPersistentNameSchemeRegistry {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    registry
        .register(InstalledPersistentNameScheme::from_ordered_roles(
            FACE_SCHEME_ID,
            PersistentNameSubjectKind::Entity,
            ["authored", "cut_piece", "merged_face"],
            true,
            true,
        ))
        .unwrap();
    registry
        .register(InstalledPersistentNameScheme::from_ordered_roles(
            SEAM_SCHEME_ID,
            PersistentNameSubjectKind::Relation,
            ["seam", "seam_piece", "merged_seam"],
            true,
            true,
        ))
        .unwrap();
    registry
}

pub(super) fn empty_working_state() -> WorkingState {
    WorkingState::new(
        BTreeMap::new(),
        AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
    )
}

pub(super) fn push_entity(staged: &mut WorkingState, version_id: VersionId) -> EntityId {
    let part = staged.get_partition_mut(PartitionId::main());
    let (slot, generation, _) = part.entity_arena.push_slot(SlotInit::<EntityRecordKind> {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        version_id,
        extra: Default::default(),
    });
    EntityId::new(PartitionId::main(), slot as u64, generation)
}

pub(super) fn push_relation(
    staged: &mut WorkingState,
    version_id: VersionId,
    source: EntityId,
    target: EntityId,
) -> RelationId {
    let part = staged.get_partition_mut(PartitionId::main());
    let (slot, generation, _) = part
        .relation_arena
        .push_slot(SlotInit::<RelationRecordKind> {
            partition_id: PartitionId::main(),
            kind_id: KindId(10),
            version_id,
            extra: RelationExtra::from_funded_parts(
                Some(RelationEndpoints { source, target }),
                None,
                None,
            ),
        });
    RelationId::new(PartitionId::main(), slot as u64, generation)
}

pub(super) fn retire_entity(staged: &mut WorkingState, entity: EntityId, version_id: VersionId) {
    let part = staged.get_partition_mut(entity.partition_id);
    part.entity_arena
        .retire(entity.slot_index(), version_id)
        .expect("entity slot exists to retire");
}

pub(super) fn retire_relation(
    staged: &mut WorkingState,
    relation: RelationId,
    version_id: VersionId,
) {
    let part = staged.get_partition_mut(relation.partition_id);
    part.relation_arena
        .retire(relation.slot_index(), version_id)
        .expect("relation slot exists to retire");
}

pub(super) fn receipt_at(commit: u64, version: u64) -> RelationalCommitReceipt {
    RelationalCommitReceipt {
        commit_id: CommitId(commit),
        version_id: VersionId(version),
        branch_id: BranchId("main".to_owned()),
        parents: if commit > 1 {
            vec![CommitId(commit - 1)]
        } else {
            Vec::new()
        },
    }
}

pub(super) fn canonical_label_discriminator(label: &str) -> Vec<u8> {
    if label.is_empty() {
        return Vec::new();
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.query.name-discriminator.v1\0");
    let disc_id = b"worth.test.posture.label.v1";
    bytes.extend_from_slice(&(disc_id.len() as u64).to_be_bytes());
    bytes.extend_from_slice(disc_id);
    bytes.push(0x19); // TAG_TEXT
    bytes.extend_from_slice(&(label.len() as u64).to_be_bytes());
    bytes.extend_from_slice(label.as_bytes());
    bytes
}

pub(super) fn face_birth(
    occurrence_key: &str,
    role_path: &str,
    parents: Vec<PersistentNameIdentity>,
    label: &str,
    entity: EntityId,
) -> PersistentNameBirthRequest {
    PersistentNameBirthRequest {
        scheme_id: FACE_SCHEME_ID.to_owned(),
        occurrence_key: occurrence_key.to_owned(),
        role_path: role_path.to_owned(),
        parent_identities: parents,
        referenced_names: Vec::new(),
        canonical_discriminator: canonical_label_discriminator(label),
        holder: PersistentNameHolder::Entity(entity),
    }
}

pub(super) fn seam_birth(
    occurrence_key: &str,
    role_path: &str,
    parents: Vec<PersistentNameIdentity>,
    label: &str,
    relation: RelationId,
) -> PersistentNameBirthRequest {
    PersistentNameBirthRequest {
        scheme_id: SEAM_SCHEME_ID.to_owned(),
        occurrence_key: occurrence_key.to_owned(),
        role_path: role_path.to_owned(),
        parent_identities: parents,
        referenced_names: Vec::new(),
        canonical_discriminator: canonical_label_discriminator(label),
        holder: PersistentNameHolder::Relation(relation),
    }
}

pub(super) fn staged_entity_lineage(state: &WorkingState, entity: EntityId) -> Option<LineageId> {
    state
        .get_current_partition(entity.partition_id)
        .unwrap()
        .entity_arena()
        .get(&entity)
        .unwrap()
        .extra()
        .lineage_id
}

pub(super) fn seed_two_faces_and_parallel_seams() -> (
    WorkingState,
    EntityId,
    EntityId,
    RelationId,
    RelationId,
    Vec<crate::facade::transactions::RecordRef>,
    crate::lineage::data::LineageCommitPostureInput,
) {
    use crate::facade::lineage::{LineagePostureClaim, LineageRecordTarget};
    use crate::lineage::data::LineageCommitPostureInput;
    use crate::facade::transactions::RecordRef;

    let mut staged = empty_working_state();
    let f1 = push_entity(&mut staged, VersionId(1));
    let f2 = push_entity(&mut staged, VersionId(1));
    let s1 = push_relation(&mut staged, VersionId(1), f1, f2);
    let s2 = push_relation(&mut staged, VersionId(1), f1, f2);
    let changed = vec![
        RecordRef::Entity(f1),
        RecordRef::Entity(f2),
        RecordRef::Relation(s1),
        RecordRef::Relation(s2),
    ];
    let input = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        live_holders: BTreeMap::new(),
        live_lineages: BTreeMap::new(),
        retired_lineages: BTreeMap::new(),
        postures: vec![
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(f1),
                birth: face_birth("occ.rt.1", "authored", vec![], "f1", f1),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(f2),
                birth: face_birth("occ.rt.1", "authored", vec![], "f2", f2),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(s1),
                birth: seam_birth("occ.rt.1", "seam", vec![], "s1", s1),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(s2),
                birth: seam_birth("occ.rt.1", "seam", vec![], "s2", s2),
            },
        ],
    };
    (staged, f1, f2, s1, s2, changed, input)
}
