//! Differential: a commit's name preview answers exactly what its
//! finalization records, and refuses exactly what finalization refuses.
use std::collections::BTreeMap;

use super::commit_name_preview::{preview_commit_names, CommitNamePreviewInput, PendingRecordName};
use super::finalize_postures_in_working_state_for_test;
use super::posture_fixture::records::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    seam_birth,
};
use super::posture_fixture::LineageCommitPostureInput;
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::{CommitNamingPlan, LineagePostureClaim, LineageRecordTarget};
use crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::runtime::state::subsystems::lineage_identity::LineageIdentityAllocator;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub(super) fn preview(
    staged: &WorkingState,
    version_id: VersionId,
    changed: &[RecordRef],
    input: &LineageCommitPostureInput,
) -> Result<super::PendingCommitNames, crate::contracts::lineage::data::LineageNameDenial> {
    let plan = CommitNamingPlan {
        postures: input.postures.clone(),
        ..CommitNamingPlan::default()
    };
    // The postures name their records directly: nothing is created by reference.
    let symbols = crate::contracts::symbols::data::string_interner::StringInterner::default();
    preview_commit_names(CommitNamePreviewInput {
        staged,
        version_id,
        changed_records: changed,
        created_entities: &CommitCreatedEntityBindings::from_owner_map(BTreeMap::new(), &symbols),
        created_relations: &CommitCreatedRelationBindings::from_owner_map(
            BTreeMap::new(),
            &symbols,
        ),
        naming_plan: Some(&plan),
        scheme_registry: &input.scheme_registry,
        schema_registry: None,
        basis: input,
        graph_selection: None,
    })
}

#[test]
fn a_preview_names_each_record_with_the_birth_finalization_records() {
    let mut staged = empty_working_state();
    let face = push_entity(&mut staged, VersionId(1));
    let other = push_entity(&mut staged, VersionId(1));
    let seam = push_relation(&mut staged, VersionId(1), face, other);
    let input = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        postures: vec![
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(face),
                birth: face_birth("occ", "authored", vec![], "a", face),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(other),
                birth: face_birth("occ", "authored", vec![], "b", other),
            },
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(seam),
                birth: seam_birth("occ", "seam", vec![], "s", seam),
            },
        ],
        ..LineageCommitPostureInput::default()
    };
    let changed = [
        RecordRef::Entity(face),
        RecordRef::Entity(other),
        RecordRef::Relation(seam),
    ];
    let names = preview(&staged, VersionId(1), &changed, &input).expect("the births validate");
    let finalized = finalize_postures_in_working_state_for_test(
        &LineageIdentityAllocator::new(),
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &changed,
        &input,
    )
    .expect("finalization records the same births");
    let recorded: BTreeMap<_, _> = finalized
        .artifact()
        .event_batch()
        .events()
        .iter()
        .filter_map(|event| event.name_birth())
        .map(|birth| (birth.identity(), birth.clone()))
        .collect();
    assert_eq!(recorded.len(), 3);
    for target in [
        LineageRecordTarget::Entity(face),
        LineageRecordTarget::Entity(other),
        LineageRecordTarget::Relation(seam),
    ] {
        let Some(PendingRecordName::Named(key)) = names.record_name(target) else {
            panic!("{target:?} is named by the commit");
        };
        let (PersistentNameKey::Entity(identity) | PersistentNameKey::Relation(identity)) = key
        else {
            panic!("a record name is never a group");
        };
        assert_eq!(key.subject_kind(), target.subject_kind());
        assert_eq!(names.birth_of(key), recorded.get(&identity));
        assert_eq!(
            names.recorded_name(key.subject_kind(), key.digest()),
            Some(key)
        );
    }
    // A name never answers under another subject class.
    let Some(PendingRecordName::Named(face_key)) =
        names.record_name(LineageRecordTarget::Entity(face))
    else {
        unreachable!()
    };
    let as_relation = crate::contracts::lineage::data::PersistentNameSubjectKind::Relation;
    assert_eq!(names.recorded_name(as_relation, face_key.digest()), None);
}

#[test]
fn a_preview_refuses_exactly_what_finalization_refuses() {
    let mut staged = empty_working_state();
    let first = push_entity(&mut staged, VersionId(1));
    let second = push_entity(&mut staged, VersionId(1));
    let input = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        postures: [first, second]
            .map(|face| LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(face),
                birth: face_birth("occ", "authored", vec![], "same", face),
            })
            .to_vec(),
        ..LineageCommitPostureInput::default()
    };
    let changed = [RecordRef::Entity(first), RecordRef::Entity(second)];
    let refused = preview(&staged, VersionId(1), &changed, &input).expect_err("the births collide");
    let Err(finalized) = finalize_postures_in_working_state_for_test(
        &LineageIdentityAllocator::new(),
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &changed,
        &input,
    ) else {
        panic!("finalization refuses the collision");
    };
    assert_eq!(finalized.lineage_name_denial_ref(), Some(&refused));
    assert!(matches!(
        refused,
        crate::contracts::lineage::data::LineageNameDenial::DuplicateBirth { .. }
    ));
}
