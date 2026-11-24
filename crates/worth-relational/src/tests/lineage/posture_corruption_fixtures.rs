use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalizationFixturePort as _;

use crate::facade::identity::{EntityId, LineageId, RelationId, VersionId};
use crate::facade::lineage::{
    LineageEventKind, LineagePostureClaim, LineageRecordTarget, PersistentNameHolder,
    PersistentNameIdentity,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state_for_test;
use crate::lineage::authority::posture_fixture::LineageCommitPostureInput;
use crate::runtime::LineageIdentityAllocator;
use crate::storage::overlay::WorkingState;

use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    seam_birth, staged_entity_lineage,
};

pub(super) struct CorruptionBaseModel {
    pub(super) staged: WorkingState,
    pub(super) f1: EntityId,
    pub(super) f2: EntityId,
    pub(super) r1: RelationId,
    pub(super) r2: RelationId,
    pub(super) f1_name: PersistentNameIdentity,
    pub(super) f2_name: PersistentNameIdentity,
    pub(super) f3_name: PersistentNameIdentity,
    pub(super) r1_name: PersistentNameIdentity,
    pub(super) r2_name: PersistentNameIdentity,
    pub(super) r3_name: PersistentNameIdentity,
    pub(super) live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub(super) live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
}

impl CorruptionBaseModel {
    pub(super) fn input_with(
        &self,
        postures: Vec<LineagePostureClaim>,
    ) -> LineageCommitPostureInput {
        LineageCommitPostureInput {
            scheme_registry: cad_scheme_registry(),
            live_holders: self.live_holders.clone(),
            live_lineages: self.live_lineages.clone(),
            retired_lineages: BTreeMap::new(),
            postures,
        }
    }
}

pub(super) fn setup_corruption_base(allocator: &LineageIdentityAllocator) -> CorruptionBaseModel {
    let mut staged = empty_working_state();
    let f1 = push_entity(&mut staged, VersionId(1));
    let f2 = push_entity(&mut staged, VersionId(1));
    let f3 = push_entity(&mut staged, VersionId(1));
    let r1 = push_relation(&mut staged, VersionId(1), f1, f2);
    let r2 = push_relation(&mut staged, VersionId(1), f1, f2);
    let r3 = push_relation(&mut staged, VersionId(1), f2, f3);

    let input = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        live_holders: BTreeMap::new(),
        live_lineages: BTreeMap::new(),
        retired_lineages: BTreeMap::new(),
        postures: vec![
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(f1),
                birth: face_birth("occ.base", "authored", vec![], "f1", f1),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(f2),
                birth: face_birth("occ.base", "authored", vec![], "f2", f2),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(f3),
                birth: face_birth("occ.base", "authored", vec![], "f3", f3),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(r1),
                birth: seam_birth("occ.base", "seam", vec![], "r1", r1),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(r2),
                birth: seam_birth("occ.base", "seam", vec![], "r2", r2),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(r3),
                birth: seam_birth("occ.base", "seam", vec![], "r3", r3),
            },
        ],
    };
    let prepared = finalize_postures_in_working_state_for_test(
        allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[
            RecordRef::Entity(f1),
            RecordRef::Entity(f2),
            RecordRef::Entity(f3),
            RecordRef::Relation(r1),
            RecordRef::Relation(r2),
            RecordRef::Relation(r3),
        ],
        &input,
    )
    .expect("base model setup succeeds");

    let events = prepared.artifact().event_batch().events();
    let f1_lin = staged_entity_lineage(&staged, f1).unwrap();
    let f2_lin = staged_entity_lineage(&staged, f2).unwrap();
    let f3_lin = staged_entity_lineage(&staged, f3).unwrap();
    let r1_lin = prepared.relation_lineages_for_test()[&r1];
    let r2_lin = prepared.relation_lineages_for_test()[&r2];
    let r3_lin = prepared.relation_lineages_for_test()[&r3];
    let name_of = |lin: LineageId| {
        events
            .iter()
            .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [lin])
            .and_then(|e| e.name_birth())
            .unwrap()
            .identity()
    };
    let (f1_name, f2_name, f3_name) = (name_of(f1_lin), name_of(f2_lin), name_of(f3_lin));
    let (r1_name, r2_name, r3_name) = (name_of(r1_lin), name_of(r2_lin), name_of(r3_lin));

    let live_holders = BTreeMap::from([
        (f1_name, PersistentNameHolder::Entity(f1)),
        (f2_name, PersistentNameHolder::Entity(f2)),
        (f3_name, PersistentNameHolder::Entity(f3)),
        (r1_name, PersistentNameHolder::Relation(r1)),
        (r2_name, PersistentNameHolder::Relation(r2)),
        (r3_name, PersistentNameHolder::Relation(r3)),
    ]);
    let live_lineages = BTreeMap::from([
        (PersistentNameHolder::Entity(f1), f1_lin),
        (PersistentNameHolder::Entity(f2), f2_lin),
        (PersistentNameHolder::Entity(f3), f3_lin),
        (PersistentNameHolder::Relation(r1), r1_lin),
        (PersistentNameHolder::Relation(r2), r2_lin),
        (PersistentNameHolder::Relation(r3), r3_lin),
    ]);
    CorruptionBaseModel {
        staged,
        f1,
        f2,
        r1,
        r2,
        f1_name,
        f2_name,
        f3_name,
        r1_name,
        r2_name,
        r3_name,
        live_holders,
        live_lineages,
    }
}
