//! Compare recorded correction meaning against the independently held original owner.
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};

pub fn assert_agrees(
    data: &WorthQueryRecoveredApplicationCommitReceipt,
    live: &WorthQueryApplicationCommitReceipt,
) {
    assert_eq!(data.commit_reference(), live.commit_reference());
    assert_eq!(Some(data.outcome_identity()), live.outcome_identity());
    assert_eq!(
        data.idempotency_key_identity(),
        live.idempotency_binding().key_identity()
    );
    assert_eq!(data.changed_record_count(), live.changed_record_count());
    assert_eq!(data.emitted_effect_count(), live.emitted_effect_count());
    assert_eq!(
        data.exact_output_settlement(),
        live.recorded_output_settlement()
    );
    let publication = live.committed_product_publication();
    let head = data
        .new_product_head()
        .memory()
        .expect("the fixture has a real memory owner");
    assert_eq!(
        head.owner(),
        publication.product_branch().owner_identity().get()
    );
    assert_eq!(head.branch(), publication.product_branch().name().as_str());
    assert_eq!(
        head.incarnation(),
        publication.product_incarnation().ordinal()
    );
    assert_eq!(head.generation(), publication.product_generation().get());
    assert_eq!(
        head.selected_commit(),
        publication.composite_commit().ordinal()
    );
    assert_eq!(
        data.attempt_identity().ordinal(),
        publication.publication_attempt().ordinal()
    );
    assert_eq!(
        data.attempt_identity().owner(),
        worth_query_host::facade::primary_graph::WorldOwnerCoordinate::MemoryOnly {
            observation: publication.publication_attempt().owner_identity().get()
        }
    );
    let descriptor = |basis: &worth_relational::facade::branch::RelationalBranchBasisDescriptor| {
        (
            basis.descriptor_version(),
            basis.branch_id().clone(),
            basis.reference().generation(),
            basis.truth_version(),
            basis.schema_commitment().clone(),
            basis.visibility_commitment().clone(),
            basis.posture(),
        )
    };
    assert_eq!(
        descriptor(data.basis_descriptor()),
        descriptor(live.basis_descriptor())
    );
}
