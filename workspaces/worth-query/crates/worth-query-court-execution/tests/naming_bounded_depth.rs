//! Slice 1B court proving O(1) bounded `WorthQueryName<K>` size (32 bytes) and
//! O(1) birth-part size across a 64-deep committed split chain through
//! `worth-query-execution-naming` -> `worth-relational`.

use std::collections::BTreeSet;
use std::mem::size_of;

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionPendingEntityBirth,
};
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref, fresh_cad_runtime,
    lowered_cad_catalog, release_commit_snapshot, Face, FaceSplitDisc, Seam, FACE_SCHEME_ID,
};

#[test]
fn bounded_o1_name_size_and_birth_part_size_across_64_deep_split_chain() {
    assert_eq!(size_of::<WorthQueryName<Face>>(), 32);
    assert_eq!(size_of::<WorthQueryName<Seam>>(), 32);

    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);

    // Commit root face.
    let root_ref = face_ref("root");
    let tx0 = begin_staged_tx(&runtime, "root", vec![create_face_intent("root")]);
    let mut b0 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b0.add_entity_birth(ExecutionPendingEntityBirth::new(
        root_ref.clone(),
        FACE_SCHEME_ID,
        "occ.root",
        "authored",
    ))
    .unwrap();
    let c0 = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx0,
        b0.build(),
    )
    .expect("root commit succeeds");
    let (res0, names0) = c0.into_parts();
    let (mut cur_entity, mut cur_name) = names0.created_entity_name::<Face>(&res0, &root_ref).unwrap();
    let mut cur_rel_id = names0.entity_relational_identity(cur_entity).unwrap();
    release_commit_snapshot(&runtime, &res0);

    let mut seen_names = BTreeSet::from([*cur_name.as_bytes()]);
    let mut len_d1 = None;
    let mut len_d16 = None;
    let mut len_d64 = None;

    for depth in 1..=64u8 {
        let key = format!("split-{depth:02}");
        let occ = format!("occ.cut.{depth:02}");
        let next_ref = face_ref(&key);
        let tx = begin_staged_tx(
            &runtime,
            &occ,
            vec![
                MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: cur_entity,
                })),
                create_face_intent(&key),
            ],
        );
        let mut b = ExecutionNameBirthPlanBuilder::new(&catalog, [cur_rel_id]);
        b.add_retire_entity(cur_entity, cur_rel_id);
        b.add_entity_birth(
            ExecutionPendingEntityBirth::new(next_ref.clone(), FACE_SCHEME_ID, &occ, "cut_piece")
                .with_parent(cur_name)
                .with_discriminator(&FaceSplitDisc::Side { side: depth }),
        )
        .unwrap();
        let committed = ExecutionNamingCommitAuthority::commit_with_naming_plan(
            &runtime,
            &execution,
            tx,
            b.build(),
        )
        .expect("split step commit succeeds");
        let (res, names) = committed.into_parts();
        let (next_entity, next_name) = names.created_entity_name::<Face>(&res, &next_ref).unwrap();
        let birth = names.entity_birth(next_entity).unwrap();
        assert_eq!(next_name.as_bytes().len(), 32);
        assert_eq!(birth.parts().parent_identities().len(), 1);
        assert!(seen_names.insert(*next_name.as_bytes()));
        let part_len = birth.parts().logical_byte_len();
        match depth {
            1 => len_d1 = Some(part_len),
            16 => len_d16 = Some(part_len),
            64 => len_d64 = Some(part_len),
            _ => {}
        }
        cur_rel_id = birth.identity();
        cur_entity = next_entity;
        cur_name = next_name;
        release_commit_snapshot(&runtime, &res);
    }

    assert_eq!(seen_names.len(), 65);
    assert_eq!(len_d1, len_d16, "birth part size must stay O(1) at depth 16");
    assert_eq!(len_d16, len_d64, "birth part size must stay O(1) at depth 64");
}
