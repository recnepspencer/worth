#[allow(unused_imports)]
use crate::layout_corruption_rebuild::*;
use {
    worth_store_layout_indexes::layout_parity_verification,
    worth_store_layout_indexes::layout_rebuild_candidate_readmission,
};

#[test]
fn root_manifest_rebuild_reaches_exact_parity_through_production_owners() {
    let execution = execute_root_rebuild(11);
    let candidate = execution.candidate_declaration();
    let readmitted = layout_rebuild_candidate_readmission().readmit(execution, candidate);
    let parity = layout_parity_verification().verify(readmitted);

    assert_eq!(parity.case_id().as_str(), "verified");
}

#[test]
fn copied_rebuild_shape_with_changed_value_is_denied_at_parity() {
    let execution = execute_root_rebuild(11);
    let exact = execution.rebuilt_basis();
    let hostile = worth_store_layout_indexes::DerivedIndexParityBasis::new(
        vec![worth_store_layout_indexes::DerivedIndexParityRow::new(
            exact.ordered_rows()[0].key().clone(),
            "sha256:hostile-rebuild-value",
        )],
        exact.coverage().clone(),
        exact.cost_envelope_compliant(),
        exact.counter_shape().to_vec(),
    )
    .expect("hostile candidate remains canonically shaped");
    let readmitted = layout_rebuild_candidate_readmission().readmit(
        execution,
        worth_store_layout_indexes::DerivedIndexCandidateDeclaration::from_canonical_basis(hostile),
    );
    let parity = layout_parity_verification().verify(readmitted);

    assert_eq!(parity.case_id().as_str(), "denied.value_identity");
}
