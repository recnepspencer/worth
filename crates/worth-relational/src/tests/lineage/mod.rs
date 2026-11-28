mod birth_fixtures;
mod branch_locality;
mod fork_scale;
mod graph_queries;
mod name_epoch_restore;
mod name_event_fixtures;
mod name_group_world;
mod name_history_continuity;
mod name_history_proptest;
mod name_history_proptest_harness;
mod name_history_traversal;
mod name_index_fixtures;
mod name_index_rebuild;
mod name_merge_birth_restore;
mod name_resolution_cost;
mod name_resolution_entities;
mod name_resolution_oracle;
mod name_resolution_proptest;
mod name_resolution_relations_groups;
mod name_split_birth_restore;
mod name_world;
mod name_world_history;
mod posture_conservation_merges;
mod posture_conservation_ops;
mod posture_conservation_proptest;
mod posture_corruption;
mod posture_corruption_fixtures;
mod posture_fixtures {
    //! Shared staged-record fixtures for persistent-name posture tests.

    use worth_relational_contracts::internal_port::lineage::data::naming::InstalledPersistentNameSchemeOwnerPort as _;
    use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::RecordArenaFixturePort as _;
    use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
    use std::collections::BTreeMap;
    use worth_relational_contracts::internal_port as contracts;
    use worth_relational_storage::internal_port as storage;

    use contracts::config::data::policies::{AdjacencyBackend, AdjacencyPolicy};
    use contracts::history::data::{
        commit_receipt::RelationalCommitReceipt,
        coordinates::{BranchId, CommitId},
    };
    use contracts::identity::data::{
        primitives::{KindId, LineageId, PartitionId, VersionId},
        record_ids::{EntityId, RelationId},
    };
    use contracts::lineage::data::{
        InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
        PersistentNameBirthRequest, PersistentNameHolder, PersistentNameIdentity,
        PersistentNameSubjectKind,
    };
    use storage::storage::overlay::{CurrentPartitionAccess, WorkingState};
    use storage::storage::substrate::{
        EntityRecordKind, RelationEndpoints, RelationExtra, RelationRecordKind, SlotInit,
    };

    pub const FACE_SCHEME_ID: &str = "worth.cad.face.v1";
    pub const SEAM_SCHEME_ID: &str = "worth.cad.seam.v1";

    pub fn cad_scheme_registry() -> InstalledPersistentNameSchemeRegistry {
        let mut registry = InstalledPersistentNameSchemeRegistry::new();
        registry
            .register(
                InstalledPersistentNameScheme::name_scheme_from_ordered_roles_for_test(
                    FACE_SCHEME_ID,
                    PersistentNameSubjectKind::Entity,
                    ["authored", "cut_piece", "merged_face"],
                    true,
                    true,
                ),
            )
            .unwrap();
        registry
            .register(
                InstalledPersistentNameScheme::name_scheme_from_ordered_roles_for_test(
                    SEAM_SCHEME_ID,
                    PersistentNameSubjectKind::Relation,
                    ["seam", "seam_piece", "merged_seam"],
                    true,
                    true,
                ),
            )
            .unwrap();
        registry
    }

    pub fn empty_working_state() -> WorkingState {
        WorkingState::working_state_for_test(
            BTreeMap::new(),
            AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 4,
            },
        )
    }

    pub fn push_entity(staged: &mut WorkingState, version_id: VersionId) -> EntityId {
        let part = staged.get_partition_mut(PartitionId::main());
        let (slot, generation, _) =
            part.entity_arena
                .push_slot_for_test(SlotInit::<EntityRecordKind> {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    version_id,
                    extra: Default::default(),
                });
        EntityId::new(PartitionId::main(), slot as u64, generation)
    }

    pub fn push_relation(
        staged: &mut WorkingState,
        version_id: VersionId,
        source: EntityId,
        target: EntityId,
    ) -> RelationId {
        let part = staged.get_partition_mut(PartitionId::main());
        let (slot, generation, _) = part.relation_arena.push_slot_for_test(SlotInit::<
            RelationRecordKind,
        > {
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

    pub fn retire_entity(staged: &mut WorkingState, entity: EntityId, version_id: VersionId) {
        let part = staged.get_partition_mut(entity.partition_id);
        part.entity_arena
            .retire_record_for_test(entity.slot_index(), version_id)
            .expect("entity slot exists to retire");
    }

    pub fn retire_relation(staged: &mut WorkingState, relation: RelationId, version_id: VersionId) {
        let part = staged.get_partition_mut(relation.partition_id);
        part.relation_arena
            .retire_record_for_test(relation.slot_index(), version_id)
            .expect("relation slot exists to retire");
    }

    pub fn receipt_at(commit: u64, version: u64) -> RelationalCommitReceipt {
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

    pub fn canonical_label_discriminator(label: &str) -> Vec<u8> {
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

    pub fn face_birth(
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

    pub fn seam_birth(
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

    pub fn staged_entity_lineage(state: &WorkingState, entity: EntityId) -> Option<LineageId> {
        state
            .get_current_partition(entity.partition_id)
            .unwrap()
            .entity_arena()
            .get(&entity)
            .unwrap()
            .extra()
            .lineage_id
    }

    pub fn seed_two_faces_and_parallel_seams() -> (
        WorkingState,
        EntityId,
        EntityId,
        RelationId,
        RelationId,
        Vec<contracts::transactions::data::primitives::RecordRef>,
        crate::lineage::authority::posture_fixture::LineageCommitPostureInput,
    ) {
        use contracts::lineage::data::{LineagePostureClaim, LineageRecordTarget};
        use contracts::transactions::data::primitives::RecordRef;
        use crate::lineage::authority::posture_fixture::LineageCommitPostureInput;

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

    pub fn merge_and_split_postures(
        scheme_registry: contracts::lineage::data::InstalledPersistentNameSchemeRegistry,
        faces: [(
            contracts::identity::data::record_ids::EntityId,
            contracts::lineage::data::PersistentNameIdentity,
        ); 2],
        seam: (
            contracts::identity::data::record_ids::RelationId,
            contracts::lineage::data::PersistentNameIdentity,
        ),
        children: [contracts::identity::data::record_ids::RelationId; 2],
    ) -> crate::lineage::authority::posture_fixture::LineageCommitPostureInput {
        use contracts::lineage::data::{
            LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget, LineageSplitChild,
        };
        use crate::lineage::authority::posture_fixture::LineageCommitPostureInput;

        let [(f1, f1_name), (f2, f2_name)] = faces;
        let (s1, s1_name) = seam;
        let [s1_a, s1_b] = children;
        LineageCommitPostureInput {
            scheme_registry,
            postures: vec![
                LineagePostureClaim::Merge {
                    parents: vec![
                        (LineageRecordTarget::Entity(f1), f1_name),
                        (LineageRecordTarget::Entity(f2), f2_name),
                    ],
                    target: LineageRecordTarget::Entity(f1),
                    keeper: LineageMergeKeeper::KeepParent(f1_name),
                },
                LineagePostureClaim::Split {
                    parent_target: LineageRecordTarget::Relation(s1),
                    parent_name: s1_name,
                    children: vec![
                        LineageSplitChild {
                            target: LineageRecordTarget::Relation(s1_a),
                            birth: seam_birth("occ.rt.2", "seam_piece", vec![s1_name], "a", s1_a),
                        },
                        LineageSplitChild {
                            target: LineageRecordTarget::Relation(s1_b),
                            birth: seam_birth("occ.rt.2", "seam_piece", vec![s1_name], "b", s1_b),
                        },
                    ],
                },
            ],
            ..Default::default()
        }
    }
}
mod posture_replay;
mod resolution;
mod scheme_composition;
mod self_derived_basis;

mod posture_lineage_tests;
mod commit_name_preview_continuity_tests;
mod untouched_keeper_merge;
mod commit_name_preview_tests;
