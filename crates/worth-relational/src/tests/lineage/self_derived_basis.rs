//! Proofs that `LineageSubsystem` persists relation lineages across commits,
//! preserves exact basis equality across `rebuild_derived_indexes` and fork
//! ancestry (including historical fork cutoffs), and fails closed with
//! `MissingBirth` when a schemed kind is created without a birth or posture.

use std::collections::BTreeMap;

use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::{KindId, VersionId};
use crate::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, LineageEventKind,
    LineageNameDenial, LineageRecordTarget, PersistentNameHolder, PersistentNameSubjectKind,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;
use crate::schema::data::RelationalSchemaRegistry;
use crate::tests::support::{runtime_with_test_schema, test_schema_registry};

use super::posture_fixtures::{
    push_entity, receipt_at, retire_entity, seed_two_faces_and_parallel_seams,
    staged_entity_lineage,
};

#[test]
fn lineage_basis_is_exact_across_rebuild_derived_indexes_and_fork_ancestry_cutoffs() {
    let runtime = runtime_with_test_schema();
    let allocator = LineageIdentityAllocator::new();
    let main_branch = BranchId("main".to_owned());
    let fork_branch = BranchId("fork-at-c1".to_owned());

    let (mut staged, f1, f2, s1, s2, c1_changed, c1_input) = seed_two_faces_and_parallel_seams();
    let c1 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &c1_changed,
        &c1_input,
    )
    .expect("c1 succeeds");
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

    // Advance main at commit 2 by retiring f1 BEFORE registering a historical
    // fork rooted at commit 1.
    let prep = runtime.preparation_runtime_snapshot();
    let c1_basis = prep.lineage_naming_basis.authoritative_basis(&main_branch);
    retire_entity(&mut staged, f1, VersionId(2));
    let mut c2_input = c1_input;
    c2_input.live_holders = c1_basis.live_holders.clone();
    c2_input.live_lineages = c1_basis.live_lineages.clone();
    c2_input.retired_lineages = c1_basis.retired_lineages.clone();
    c2_input.postures = vec![crate::facade::lineage::LineagePostureClaim::Retire {
        target: LineageRecordTarget::Entity(f1),
        name: f1_name,
    }];
    let c2 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[RecordRef::Entity(f1)],
        &c2_input,
    )
    .expect("c2 retire succeeds");
    let (c2_artifact, c2_nodes, c2_rels) = c2.into_parts();
    runtime.lineage_authority().install_published_lineage(
        crate::runtime::ValidatedLineageEventBatch::from_reserved(
            c2_artifact.event_batch().events().to_vec(),
        ),
        CommitId(2),
        c2_nodes,
        c2_rels,
    );

    // Register fork rooted at CommitId(1): must inherit f1 + s1 + s2 LIVE even
    // though main retired f1 at CommitId(2).
    prep.lineage_naming_basis
        .register_fork_ancestry(fork_branch.clone(), main_branch.clone(), CommitId(1));

    let main_before = prep.lineage_naming_basis.authoritative_basis(&main_branch);
    let fork_before = prep.lineage_naming_basis.authoritative_basis(&fork_branch);
    assert!(!main_before.live_holders.contains_key(&f1_name));
    assert!(main_before.retired_lineages.contains_key(&f1_name));
    assert_eq!(
        fork_before.live_holders.get(&f1_name),
        Some(&PersistentNameHolder::Entity(f1))
    );
    assert_eq!(
        fork_before.live_holders.get(&s1_name),
        Some(&PersistentNameHolder::Relation(s1))
    );
    assert_eq!(fork_before, c1_basis);

    // Full rebuild_derived_indexes must reproduce byte-identical bases for both
    // main and fork (entities + relations + fork cutoff).
    let snapshot = runtime.rebuild_lineage_derived_indexes_for_test();
    assert_eq!(
        snapshot.authoritative_branch_naming_basis(&main_branch),
        main_before
    );
    assert_eq!(
        snapshot.authoritative_branch_naming_basis(&fork_branch),
        fork_before
    );

    // Silence unused f2 in helper binding.
    let _ = f2;
}

#[test]
fn installed_schema_registry_validates_bindings_and_enforces_missing_birth() {
    let base = test_schema_registry();
    let entity_kind_name = base.entity_kinds[&KindId(1)].kind_name.clone();
    let rel_kind_name = base.relation_kinds[&KindId(2)].kind_name.clone();
    let mut reg = InstalledPersistentNameSchemeRegistry::new();
    reg.register(
        InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
            "worth.cad.entity.v1",
            PersistentNameSubjectKind::Entity,
            entity_kind_name,
            ["authored"],
            true,
            true,
        )
        .unwrap(),
    )
    .unwrap();
    reg.register(
        InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
            "worth.cad.rel.v1",
            PersistentNameSubjectKind::Relation,
            rel_kind_name,
            ["seam"],
            true,
            true,
        )
        .unwrap(),
    )
    .unwrap();

    // Mismatched kind name binding must refuse at schema installation.
    let bad = test_schema_registry().install_persistent_name_schemes(
        reg.clone(),
        BTreeMap::from([(
            (PersistentNameSubjectKind::Entity, KindId(1)),
            "worth.cad.rel.v1".to_owned(),
        )]),
    );
    assert!(bad.is_err());

    let schema: RelationalSchemaRegistry = test_schema_registry()
        .install_persistent_name_schemes(
            reg,
            BTreeMap::from([
                (
                    (PersistentNameSubjectKind::Entity, KindId(1)),
                    "worth.cad.entity.v1".to_owned(),
                ),
                (
                    (PersistentNameSubjectKind::Relation, KindId(2)),
                    "worth.cad.rel.v1".to_owned(),
                ),
            ]),
        )
        .expect("valid bindings install");
    assert_eq!(
        schema.scheme_id_for_kind(PersistentNameSubjectKind::Entity, KindId(1)),
        Some("worth.cad.entity.v1")
    );
    assert_eq!(
        schema.scheme_id_for_kind(PersistentNameSubjectKind::Relation, KindId(2)),
        Some("worth.cad.rel.v1")
    );

    // Verify finalize_postures_with_installer refuses MissingBirth on both
    // schemed entities and schemed relations when postures are empty.
    let allocator = LineageIdentityAllocator::new();
    let (mut staged, f1, f2, _, _, _, _) = seed_two_faces_and_parallel_seams();
    let before = allocator.frontiers();
    let fresh_e = push_entity(&mut staged, VersionId(2));
    let err = crate::lineage::authority::test_finalize_with_schema(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[RecordRef::Entity(fresh_e)],
        &schema,
    )
    .err()
    .expect("missing entity birth must refuse");
    assert!(matches!(
        err.lineage_name_denial_ref(),
        Some(LineageNameDenial::MissingBirth {
            target,
            scheme_id,
        }) if *target == LineageRecordTarget::Entity(fresh_e)
            && scheme_id == "worth.cad.entity.v1"
    ));
    assert_eq!(allocator.frontiers(), before);

    let fresh_r = {
        use crate::storage::substrate::{RelationEndpoints, RelationExtra, RelationRecordKind, SlotInit};
        let part = staged.get_partition_mut(crate::facade::identity::PartitionId::main());
        let (slot, gen_id, _) = part.relation_arena.push_slot(SlotInit::<RelationRecordKind> {
            partition_id: crate::facade::identity::PartitionId::main(),
            kind_id: KindId(2),
            version_id: VersionId(2),
            extra: RelationExtra::from_funded_parts(
                Some(RelationEndpoints { source: f1, target: f2 }),
                None,
                None,
            ),
        });
        crate::facade::identity::RelationId::new(crate::facade::identity::PartitionId::main(), slot as u64, gen_id)
    };
    let err_r = crate::lineage::authority::test_finalize_with_schema(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[RecordRef::Relation(fresh_r)],
        &schema,
    )
    .err()
    .expect("missing relation birth must refuse");
    assert!(matches!(
        err_r.lineage_name_denial_ref(),
        Some(LineageNameDenial::MissingBirth {
            target,
            scheme_id,
        }) if *target == LineageRecordTarget::Relation(fresh_r)
            && scheme_id == "worth.cad.rel.v1"
    ));
    assert_eq!(allocator.frontiers(), before);
}
