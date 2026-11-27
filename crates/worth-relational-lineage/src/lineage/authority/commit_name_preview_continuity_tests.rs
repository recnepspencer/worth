//! Differential continuity: a birth whose identity is a retired name of the
//! branch restores that name (Create, Split child, Merge `MintNew`), and a
//! record the commit retires without a posture ends its name, in the preview
//! exactly as finalization records it.

use super::commit_name_preview::PendingRecordName;
use super::commit_name_preview_tests::preview;
use super::finalize_postures_in_working_state_for_test;
use super::posture_fixture::records::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    retire_entity, retire_relation, staged_entity_lineage, FACE_SCHEME_ID,
};
use super::posture_fixture::LineageCommitPostureInput;
use crate::contracts::identity::data::primitives::{LineageId, VersionId};
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::{
    LineageEventKind, LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget,
    LineageSplitChild, LineageSubject, PersistentNameBirthRequest, PersistentNameHolder,
    PersistentNameIdentity,
};
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::runtime::state::subsystems::lineage_identity::LineageIdentityAllocator;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

fn identity(birth: &PersistentNameBirthRequest) -> PersistentNameIdentity {
    PersistentNameIdentity::of_pending_birth(
        FACE_SCHEME_ID,
        &birth.occurrence_key,
        &birth.role_path,
        &birth.parent_identities,
        &birth.canonical_discriminator,
        false,
    )
}

fn entity(id: EntityId) -> LineageRecordTarget {
    LineageRecordTarget::Entity(id)
}

/// Faces born and finalized at version 1, live on the basis.
fn seeded(
    staged: &mut WorkingState,
    allocator: &LineageIdentityAllocator,
    labels: &[&str],
) -> (
    Vec<(EntityId, PersistentNameIdentity)>,
    LineageCommitPostureInput,
) {
    let faces: Vec<_> = labels
        .iter()
        .map(|label| {
            let face = push_entity(staged, VersionId(1));
            (face, face_birth("seed", "authored", vec![], label, face))
        })
        .collect();
    let input = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        postures: faces
            .iter()
            .map(|(face, birth)| LineagePostureClaim::Create {
                target: entity(*face),
                birth: birth.clone(),
            })
            .collect(),
        ..LineageCommitPostureInput::default()
    };
    let changed: Vec<_> = faces
        .iter()
        .map(|(face, _)| RecordRef::Entity(*face))
        .collect();
    finalize_postures_in_working_state_for_test(
        allocator,
        staged,
        &receipt_at(1, 1),
        &[],
        &changed,
        &input,
    )
    .expect("the seed births finalize");
    let mut basis = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        ..LineageCommitPostureInput::default()
    };
    let named = faces
        .into_iter()
        .map(|(face, birth)| {
            let name = identity(&birth);
            let holder = PersistentNameHolder::Entity(face);
            let lineage = staged_entity_lineage(staged, face).expect("a seeded lineage");
            basis.live_holders.insert(name, holder.clone());
            basis.live_lineages.insert(holder, lineage);
            (face, name)
        })
        .collect();
    (named, basis)
}

/// Previews and finalizes one commit at version 2; returns the preview's
/// name for each of `targets` and the names finalization restored.
fn restored_at_version_2(
    staged: &mut WorkingState,
    allocator: &LineageIdentityAllocator,
    changed: &[RecordRef],
    input: &LineageCommitPostureInput,
    targets: &[LineageRecordTarget],
) -> (Vec<Option<PendingRecordName>>, Vec<PersistentNameIdentity>) {
    let names = preview(staged, VersionId(2), changed, input).expect("the commit validates");
    let answers = targets
        .iter()
        .map(|target| names.record_name(*target))
        .collect();
    let finalized = finalize_postures_in_working_state_for_test(
        allocator,
        staged,
        &receipt_at(2, 2),
        &[],
        changed,
        input,
    )
    .expect("finalization accepts the same commit");
    let restored = finalized
        .artifact()
        .event_batch()
        .events()
        .iter()
        .filter(|event| event.kind() == LineageEventKind::Restore)
        .filter_map(|event| event.kept_name())
        .collect();
    (answers, restored)
}

fn named(name: PersistentNameIdentity) -> Option<PendingRecordName> {
    Some(PendingRecordName::Named(PersistentNameKey::Entity(name)))
}

fn retired_basis(
    mut basis: LineageCommitPostureInput,
    name: PersistentNameIdentity,
) -> LineageCommitPostureInput {
    basis
        .retired_lineages
        .insert(name, (LineageSubject::Entity, LineageId(900)));
    basis
}

#[test]
fn a_created_birth_of_a_retired_name_restores_it() {
    let (mut staged, allocator) = (empty_working_state(), LineageIdentityAllocator::new());
    let (_, basis) = seeded(&mut staged, &allocator, &["other"]);
    let face = push_entity(&mut staged, VersionId(2));
    let birth = face_birth("again", "authored", vec![], "a", face);
    let mut input = retired_basis(basis, identity(&birth));
    input.postures = vec![LineagePostureClaim::Create {
        target: entity(face),
        birth: birth.clone(),
    }];
    let changed = [RecordRef::Entity(face)];
    let (answers, restored) =
        restored_at_version_2(&mut staged, &allocator, &changed, &input, &[entity(face)]);
    assert_eq!(restored, [identity(&birth)]);
    assert_eq!(answers, [named(identity(&birth))]);
}

#[test]
fn a_split_child_and_a_minted_merge_of_a_retired_name_restore_it() {
    let (mut staged, allocator) = (empty_working_state(), LineageIdentityAllocator::new());
    let (seeded_faces, basis) = seeded(&mut staged, &allocator, &["p", "q1", "q2"]);
    let [(p, p_name), (q1, q1_name), (q2, q2_name)] = seeded_faces[..] else {
        unreachable!()
    };
    for face in [p, q1, q2] {
        retire_entity(&mut staged, face, VersionId(2));
    }
    let (left, right, merged) = (
        push_entity(&mut staged, VersionId(2)),
        push_entity(&mut staged, VersionId(2)),
        push_entity(&mut staged, VersionId(2)),
    );
    let left_birth = face_birth("cut", "cut_piece", vec![p_name], "left", left);
    let right_birth = face_birth("cut", "cut_piece", vec![p_name], "right", right);
    let mut parents = vec![q1_name, q2_name];
    parents.sort_unstable();
    let merged_birth = face_birth("join", "merged_face", parents, "m", merged);
    let mut input = retired_basis(basis, identity(&left_birth));
    input = retired_basis(input, identity(&merged_birth));
    input.postures = vec![
        LineagePostureClaim::Split {
            parent_target: entity(p),
            parent_name: p_name,
            children: vec![
                LineageSplitChild {
                    target: entity(left),
                    birth: left_birth.clone(),
                },
                LineageSplitChild {
                    target: entity(right),
                    birth: right_birth.clone(),
                },
            ],
        },
        LineagePostureClaim::Merge {
            parents: vec![(entity(q1), q1_name), (entity(q2), q2_name)],
            target: entity(merged),
            keeper: LineageMergeKeeper::MintNew {
                birth: merged_birth.clone(),
            },
        },
    ];
    let changed = [p, q1, q2, left, right, merged].map(RecordRef::Entity);
    let targets = [left, right, merged, p, q1].map(entity);
    let (answers, mut restored) =
        restored_at_version_2(&mut staged, &allocator, &changed, &input, &targets);
    let mut expected = vec![identity(&left_birth), identity(&merged_birth)];
    expected.sort_unstable();
    restored.sort_unstable();
    assert_eq!(restored, expected, "finalization restores both names");
    let ended = Some(PendingRecordName::Ended);
    assert_eq!(
        answers,
        [
            named(identity(&left_birth)),
            named(identity(&right_birth)),
            named(identity(&merged_birth)),
            ended,
            ended,
        ]
    );
}

#[test]
fn a_record_retired_without_a_posture_ends_its_name() {
    let (mut staged, allocator) = (empty_working_state(), LineageIdentityAllocator::new());
    let (seeded_faces, basis) = seeded(&mut staged, &allocator, &["kept", "gone", "end"]);
    let [(kept, _), (gone, _), (end, _)] = seeded_faces[..] else {
        unreachable!()
    };
    // An unpostured delete, and a relation retired with its endpoint (as a
    // cascade or a replaced source leaves it): neither carries a posture.
    let edge = push_relation(&mut staged, VersionId(1), gone, end);
    retire_entity(&mut staged, gone, VersionId(2));
    retire_relation(&mut staged, edge, VersionId(2));
    let changed = [RecordRef::Entity(gone), RecordRef::Relation(edge)];
    let names = preview(&staged, VersionId(2), &changed, &basis).expect("nothing is claimed");
    let ended = Some(PendingRecordName::Ended);
    assert_eq!(names.record_name(entity(gone)), ended);
    assert_eq!(
        names.record_name(LineageRecordTarget::Relation(edge)),
        ended
    );
    assert_eq!(
        names.record_name(entity(kept)),
        None,
        "untouched keeps its name"
    );
    assert_eq!(
        names.record_name(entity(end)),
        None,
        "a live endpoint is untouched"
    );
}
