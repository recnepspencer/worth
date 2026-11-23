use crate::facade::identity::VersionId;
use crate::facade::lineage::{
    LineageMergeKeeper, LineageNameDenial, LineagePostureClaim,
    LineagePostureKind, LineageRecordTarget, LineageSplitChild, LineageSubject,
};
use crate::lineage::data::LineageCommitPostureInput;
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;

use super::posture_corruption_fixtures::setup_corruption_base;
use super::posture_fixtures::{
    face_birth, push_entity, push_relation, receipt_at, retire_entity, retire_relation, seam_birth,
    staged_entity_lineage,
};

fn run_refused(
    allocator: &LineageIdentityAllocator,
    staged: &mut crate::storage::overlay::WorkingState,
    changed: &[RecordRef],
    input: &LineageCommitPostureInput,
) -> LineageNameDenial {
    let before = allocator.frontiers();
    let err = finalize_postures_in_working_state(
        allocator,
        staged,
        &receipt_at(2, 2),
        &[],
        changed,
        input,
    )
    .err()
    .expect("corrupted posture claim must refuse");
    assert_eq!(
        allocator.frontiers(),
        before,
        "refused posture must leave allocator frontiers untouched"
    );
    err.lineage_name_denial_ref()
        .expect("refusal must carry typed LineageNameDenial")
        .clone()
}

#[test]
fn corruption_court_1_split_without_retiring_parent_refuses_with_zero_effects() {
    let allocator = LineageIdentityAllocator::new();
    let mut base = setup_corruption_base(&allocator);

    let child_e = push_entity(&mut base.staged, VersionId(2));
    let input_e = base.input_with(vec![LineagePostureClaim::Split {
        parent_target: LineageRecordTarget::Entity(base.f1),
        parent_name: base.f1_name,
        children: vec![LineageSplitChild {
            target: LineageRecordTarget::Entity(child_e),
            birth: face_birth("occ.bad", "cut_piece", vec![base.f1_name], "p1", child_e),
        }],
    }]);
    let denial_e = run_refused(
        &allocator,
        &mut base.staged,
        &[RecordRef::Entity(child_e)],
        &input_e,
    );
    assert!(matches!(
        denial_e,
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Split,
            subject: LineageSubject::Entity,
            ..
        }
    ));
    assert_eq!(staged_entity_lineage(&base.staged, child_e), None);

    let child_r = push_relation(&mut base.staged, VersionId(2), base.f1, base.f2);
    let input_r = base.input_with(vec![LineagePostureClaim::Split {
        parent_target: LineageRecordTarget::Relation(base.r1),
        parent_name: base.r1_name,
        children: vec![LineageSplitChild {
            target: LineageRecordTarget::Relation(child_r),
            birth: seam_birth("occ.bad", "seam_piece", vec![base.r1_name], "s1", child_r),
        }],
    }]);
    let denial_r = run_refused(
        &allocator,
        &mut base.staged,
        &[RecordRef::Relation(child_r)],
        &input_r,
    );
    assert!(matches!(
        denial_r,
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Split,
            subject: LineageSubject::Relation,
            ..
        }
    ));
}

#[test]
fn corruption_court_2_merge_with_keep_parent_outside_parents_refuses_with_zero_effects() {
    let allocator = LineageIdentityAllocator::new();
    let mut base = setup_corruption_base(&allocator);

    retire_entity(&mut base.staged, base.f2, VersionId(2));
    retire_relation(&mut base.staged, base.r2, VersionId(2));

    let input_e = base.input_with(vec![LineagePostureClaim::Merge {
        parents: vec![
            (LineageRecordTarget::Entity(base.f1), base.f1_name),
            (LineageRecordTarget::Entity(base.f2), base.f2_name),
        ],
        target: LineageRecordTarget::Entity(base.f1),
        keeper: LineageMergeKeeper::KeepParent(base.f3_name),
    }]);
    let denial_e = run_refused(
        &allocator,
        &mut base.staged,
        &[RecordRef::Entity(base.f2)],
        &input_e,
    );
    assert!(matches!(
        denial_e,
        LineageNameDenial::InvalidMergeKeeper { keeper, .. } if keeper == base.f3_name
    ));

    let input_r = base.input_with(vec![LineagePostureClaim::Merge {
        parents: vec![
            (LineageRecordTarget::Relation(base.r1), base.r1_name),
            (LineageRecordTarget::Relation(base.r2), base.r2_name),
        ],
        target: LineageRecordTarget::Relation(base.r1),
        keeper: LineageMergeKeeper::KeepParent(base.r3_name),
    }]);
    let denial_r = run_refused(
        &allocator,
        &mut base.staged,
        &[RecordRef::Relation(base.r2)],
        &input_r,
    );
    assert!(matches!(
        denial_r,
        LineageNameDenial::InvalidMergeKeeper { keeper, .. } if keeper == base.r3_name
    ));
}

#[test]
fn corruption_court_3_merge_with_live_unretired_non_keeper_refuses_with_zero_effects() {
    let allocator = LineageIdentityAllocator::new();
    let mut base = setup_corruption_base(&allocator);

    let input_e = base.input_with(vec![LineagePostureClaim::Merge {
        parents: vec![
            (LineageRecordTarget::Entity(base.f1), base.f1_name),
            (LineageRecordTarget::Entity(base.f2), base.f2_name),
        ],
        target: LineageRecordTarget::Entity(base.f1),
        keeper: LineageMergeKeeper::KeepParent(base.f1_name),
    }]);
    let denial_e = run_refused(&allocator, &mut base.staged, &[], &input_e);
    assert!(matches!(
        denial_e,
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Merge,
            subject: LineageSubject::Entity,
            ..
        }
    ));

    let input_r = base.input_with(vec![LineagePostureClaim::Merge {
        parents: vec![
            (LineageRecordTarget::Relation(base.r1), base.r1_name),
            (LineageRecordTarget::Relation(base.r2), base.r2_name),
        ],
        target: LineageRecordTarget::Relation(base.r1),
        keeper: LineageMergeKeeper::KeepParent(base.r1_name),
    }]);
    let denial_r = run_refused(&allocator, &mut base.staged, &[], &input_r);
    assert!(matches!(
        denial_r,
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Merge,
            subject: LineageSubject::Relation,
            ..
        }
    ));
}

#[test]
fn corruption_court_4_preserve_on_fresh_create_refuses_with_zero_effects() {
    let allocator = LineageIdentityAllocator::new();
    let mut base = setup_corruption_base(&allocator);

    let fresh_e = push_entity(&mut base.staged, VersionId(2));
    let fresh_r = push_relation(&mut base.staged, VersionId(2), base.f1, base.f2);

    let input_e = base.input_with(vec![LineagePostureClaim::Preserve {
        target: LineageRecordTarget::Entity(fresh_e),
        name: base.f1_name,
    }]);
    let denial_e = run_refused(
        &allocator,
        &mut base.staged,
        &[RecordRef::Entity(fresh_e)],
        &input_e,
    );
    assert!(matches!(
        denial_e,
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Preserve,
            subject: LineageSubject::Entity,
            ..
        }
    ));
    assert_eq!(staged_entity_lineage(&base.staged, fresh_e), None);

    let input_r = base.input_with(vec![LineagePostureClaim::Preserve {
        target: LineageRecordTarget::Relation(fresh_r),
        name: base.r1_name,
    }]);
    let denial_r = run_refused(
        &allocator,
        &mut base.staged,
        &[RecordRef::Relation(fresh_r)],
        &input_r,
    );
    assert!(matches!(
        denial_r,
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Preserve,
            subject: LineageSubject::Relation,
            ..
        }
    ));
}

#[test]
fn corruption_court_5_retire_on_unretired_record_and_empty_split_refuse_with_zero_effects() {
    let allocator = LineageIdentityAllocator::new();
    let mut base = setup_corruption_base(&allocator);

    let input_e = base.input_with(vec![LineagePostureClaim::Retire {
        target: LineageRecordTarget::Entity(base.f1),
        name: base.f1_name,
    }]);
    assert!(matches!(
        run_refused(&allocator, &mut base.staged, &[], &input_e),
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Retire,
            subject: LineageSubject::Entity,
            ..
        }
    ));

    let input_r = base.input_with(vec![LineagePostureClaim::Retire {
        target: LineageRecordTarget::Relation(base.r1),
        name: base.r1_name,
    }]);
    assert!(matches!(
        run_refused(&allocator, &mut base.staged, &[], &input_r),
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Retire,
            subject: LineageSubject::Relation,
            ..
        }
    ));

    retire_entity(&mut base.staged, base.f1, VersionId(2));
    let input_empty = base.input_with(vec![LineagePostureClaim::Split {
        parent_target: LineageRecordTarget::Entity(base.f1),
        parent_name: base.f1_name,
        children: Vec::new(),
    }]);
    assert!(matches!(
        run_refused(
            &allocator,
            &mut base.staged,
            &[RecordRef::Entity(base.f1)],
            &input_empty
        ),
        LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Split,
            ..
        }
    ));
}

#[test]
fn corruption_court_6_unpostured_replace_blocked_by_posture_claimed_target_refuses_with_zero_effects(
) {
    use crate::facade::identity::{KindId, PartitionId};
    use crate::facade::lineage::PersistentNameHolder;
    use crate::facade::transactions::{
        AspectFieldPatch, EntityMutationIntent, EntitySpec, MutationIntent, ReplaceEntityIntent,
    };
    use crate::symbols::data::ClientKey;

    let allocator = LineageIdentityAllocator::new();
    let mut base = setup_corruption_base(&allocator);

    // Retire f1 and create ONLY one entity of the same kind, claimed by an
    // explicit Create posture, while merged_plan asks for an unpostured Replace of f1.
    retire_entity(&mut base.staged, base.f1, VersionId(2));
    let e_claimed = push_entity(&mut base.staged, VersionId(2));
    let merged_plan = vec![MutationIntent::Entity(EntityMutationIntent::Replace(
        ReplaceEntityIntent {
            entity_id: base.f1,
            replacement: EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("claimed"),
                fields: AspectFieldPatch::default(),
            },
        },
    ))];
    let input = base.input_with(vec![LineagePostureClaim::Create {
        target: LineageRecordTarget::Entity(e_claimed),
        birth: face_birth("occ.r3.refuse", "authored", vec![], "claimed", e_claimed),
    }]);

    let before = allocator.frontiers();
    let err = finalize_postures_in_working_state(
        &allocator,
        &mut base.staged,
        &receipt_at(2, 2),
        &merged_plan,
        &[RecordRef::Entity(base.f1), RecordRef::Entity(e_claimed)],
        &input,
    )
    .err()
    .expect("unpostured Replace whose only candidate is posture-claimed must refuse");

    assert_eq!(
        allocator.frontiers(),
        before,
        "refused Replace-vs-posture conflict must not reserve allocator IDs"
    );
    assert_eq!(
        staged_entity_lineage(&base.staged, e_claimed),
        None,
        "refused Replace-vs-posture conflict must install zero staged lineage"
    );
    let denial = err
        .lineage_name_denial_ref()
        .expect("refusal must carry typed LineageNameDenial");
    assert!(
        matches!(
            denial,
            LineageNameDenial::LineagePostureMismatch {
                posture: LineagePostureKind::Create,
                subject: LineageSubject::Entity,
                holder,
                ..
            } if *holder == PersistentNameHolder::Entity(base.f1)
        ),
        "unexpected denial: {denial:?}"
    );
}
