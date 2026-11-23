//! Proofs that `LineageSubsystem` persists relation lineages across commits
//! and self-derives authoritative live/retired persistent-name sets for
//! commit finalization without trusting caller-supplied maps.

use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::VersionId;
use crate::facade::lineage::{
    LineageEventKind, LineageNameDenial, LineagePostureClaim, LineageRecordTarget,
    PersistentNameHolder,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::lineage::data::LineageCommitPostureInput;
use crate::runtime::LineageIdentityAllocator;
use crate::tests::support::runtime_with_test_schema;

use super::posture_fixtures::{
    cad_scheme_registry, face_birth, push_entity, push_relation, receipt_at, retire_entity,
    seam_birth, seed_two_faces_and_parallel_seams, staged_entity_lineage,
};

#[test]
fn lineage_subsystem_persists_relation_lineages_and_self_derives_authoritative_basis() {
    let runtime = runtime_with_test_schema();
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();

    let (mut staged, f1, f2, s1, s2, c1_changed, c1_input) = seed_two_faces_and_parallel_seams();
    let c1 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &c1_changed,
        &c1_input,
    )
    .expect("initial birth commit succeeds");
    let f1_lin = staged_entity_lineage(&staged, f1).unwrap();
    let s1_lin = c1.relation_lineages()[&s1];
    let s2_lin = c1.relation_lineages()[&s2];
    let name_for = |lin| {
        c1.artifact()
            .event_batch()
            .events()
            .iter()
            .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [lin])
            .unwrap()
            .name_birth()
            .unwrap()
            .identity()
    };
    let f1_name = name_for(f1_lin);
    let s1_name = name_for(s1_lin);

    // Install published lineage including persisted relation lineages.
    let (c1_artifact, c1_nodes, c1_rels) = c1.into_parts();
    runtime.lineage_authority().install_published_lineage(
        crate::runtime::ValidatedLineageEventBatch::from_reserved(
            c1_artifact.event_batch().events().to_vec(),
        ),
        CommitId(1),
        c1_nodes,
        c1_rels,
    );
    assert_eq!(runtime.relation_lineage(s1), Some(s1_lin));
    assert_eq!(runtime.relation_lineage(s2), Some(s2_lin));

    // Self-derived basis must contain both entity and relation live holders.
    let main_branch = BranchId("main".to_owned());
    let prep = runtime.preparation_runtime_snapshot();
    let basis = prep.lineage_naming_basis.authoritative_basis(&main_branch);
    assert_eq!(
        basis.live_holders.get(&f1_name),
        Some(&PersistentNameHolder::Entity(f1))
    );
    assert_eq!(
        basis.live_holders.get(&s1_name),
        Some(&PersistentNameHolder::Relation(s1))
    );
    assert_eq!(
        basis.live_lineages.get(&PersistentNameHolder::Entity(f1)),
        Some(&f1_lin)
    );
    assert_eq!(
        basis.live_lineages.get(&PersistentNameHolder::Relation(s1)),
        Some(&s1_lin)
    );

    // Duplicate entity birth checked against self-derived basis refuses
    // naming both holders with zero allocator drift.
    let before = allocator.frontiers();
    let dup_entity = push_entity(&mut staged, VersionId(2));
    let spoof_entity_err = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[RecordRef::Entity(dup_entity)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: basis.live_holders.clone(),
            live_lineages: basis.live_lineages.clone(),
            retired_lineages: basis.retired_lineages.clone(),
            postures: vec![LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(dup_entity),
                birth: face_birth("occ.rt.1", "authored", vec![], "f1", dup_entity),
            }],
            ..Default::default()
        },
    )
    .err()
    .expect("self-derived basis must refuse duplicate entity birth");
    assert!(matches!(
        spoof_entity_err.lineage_name_denial_ref(),
        Some(LineageNameDenial::DuplicateBirth {
            identity,
            first,
            second,
        }) if *identity == f1_name
            && *first == PersistentNameHolder::Entity(f1)
            && *second == PersistentNameHolder::Entity(dup_entity)
    ));
    assert_eq!(allocator.frontiers(), before);

    // Duplicate relation birth checked against self-derived basis refuses
    // naming both relation holders with zero allocator drift.
    let dup_rel = push_relation(&mut staged, VersionId(2), f1, f2);
    let spoof_rel_err = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[RecordRef::Relation(dup_rel)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: basis.live_holders.clone(),
            live_lineages: basis.live_lineages.clone(),
            retired_lineages: basis.retired_lineages.clone(),
            postures: vec![LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(dup_rel),
                birth: seam_birth("occ.rt.1", "seam", vec![], "s1", dup_rel),
            }],
            ..Default::default()
        },
    )
    .err()
    .expect("self-derived basis must refuse duplicate relation birth");
    assert!(matches!(
        spoof_rel_err.lineage_name_denial_ref(),
        Some(LineageNameDenial::DuplicateBirth {
            identity,
            first,
            second,
        }) if *identity == s1_name
            && *first == PersistentNameHolder::Relation(s1)
            && *second == PersistentNameHolder::Relation(dup_rel)
    ));
    assert_eq!(allocator.frontiers(), before);

    // Retiring f1 moves f1_name from live to retired in the self-derived basis.
    retire_entity(&mut staged, f1, VersionId(2));
    let c2 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[RecordRef::Entity(f1)],
        &LineageCommitPostureInput {
            scheme_registry: registry,
            live_holders: basis.live_holders,
            live_lineages: basis.live_lineages,
            retired_lineages: basis.retired_lineages,
            postures: vec![LineagePostureClaim::Retire {
                target: LineageRecordTarget::Entity(f1),
                name: f1_name,
            }],
            ..Default::default()
        },
    )
    .expect("retiring f1 succeeds");
    let (c2_artifact, c2_nodes, c2_rels) = c2.into_parts();
    runtime.lineage_authority().install_published_lineage(
        crate::runtime::ValidatedLineageEventBatch::from_reserved(
            c2_artifact.event_batch().events().to_vec(),
        ),
        CommitId(2),
        c2_nodes,
        c2_rels,
    );
    let after_retire = prep.lineage_naming_basis.authoritative_basis(&main_branch);
    assert!(!after_retire.live_holders.contains_key(&f1_name));
    assert!(!after_retire
        .live_lineages
        .contains_key(&PersistentNameHolder::Entity(f1)));
    assert!(after_retire.retired_lineages.contains_key(&f1_name));
    assert_eq!(
        after_retire.live_holders.get(&s1_name),
        Some(&PersistentNameHolder::Relation(s1))
    );
}
