use super::redo_intent::{
    WorthQueryProvedUndo, WorthQueryProvedUndoAxisProbe, WorthQueryRedoIntent,
};

#[test]
fn fanout_does_not_change_identity() {
    let proved = WorthQueryProvedUndo::axis_probe(WorthQueryProvedUndoAxisProbe {
        original_operation: [3; 32],
        principal_scope_digest: [4; 32],
        compatibility_generation: 1,
        runtime_instance: 9,
    });
    let mut digests = Vec::new();
    for (postings, lineage) in [(10usize, 1usize), (1000, 100)] {
        let _discarded = (postings, lineage);
        let intent = WorthQueryRedoIntent::derive(
            &proved,
            proved.undo_product_publication().composite_commit().clone(),
            proved.undo_commit().clone(),
        )
        .expect("derive");
        digests.push(*intent.identity().digest());
    }
    assert_eq!(digests[0], digests[1]);
}

#[test]
fn intent_does_not_decide_divergence() {
    let proved = WorthQueryProvedUndo::axis_probe(WorthQueryProvedUndoAxisProbe {
        original_operation: [1; 32],
        principal_scope_digest: [2; 32],
        compatibility_generation: 1,
        runtime_instance: 7,
    });
    let bound = proved.undo_commit().clone();
    let intent = WorthQueryRedoIntent::derive(
        &proved,
        proved.undo_product_publication().composite_commit().clone(),
        bound.clone(),
    )
    .expect("derive");
    assert_eq!(intent.bound_relational_head(), &bound);
    assert_eq!(intent.work().basis_preparations(), 1);
    assert_eq!(intent.work().digest_derivations(), 1);
    assert_eq!(intent.work().digest_text_materializations(), 0);
    let _ = intent.original_operation();
    let _ = intent.undo_product_publication();
    let _ = intent.compatibility_generation();
}
