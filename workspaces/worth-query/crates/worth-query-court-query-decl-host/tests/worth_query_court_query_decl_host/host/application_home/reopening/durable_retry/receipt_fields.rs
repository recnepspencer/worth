//! Only the existing public durable axes are compared; live execution is excluded.
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;

pub(super) fn projection(receipt: &WorthQueryApplicationCommitReceipt) -> String {
    let publication = receipt.committed_product_publication();
    let basis = receipt.basis_descriptor();
    // The live descriptor also carries process runtime/root allocation identities.
    // Compare its semantic axes explicitly, without rebinding or fabricating a basis.
    format!(
        "{:#?}",
        (
            receipt.commit_reference(),
            receipt.outcome_identity(),
            receipt.changed_record_count(),
            receipt.emitted_effect_count(),
            receipt.idempotency_binding(),
            receipt.aftermath_causality(),
            (
                (
                    publication
                        .product_branch()
                        .owner_identity()
                        .durable()
                        .expect("directory Home identity"),
                    publication.product_branch().name(),
                ),
                publication.product_incarnation().ordinal(),
                publication.product_generation(),
                (
                    publication
                        .composite_commit()
                        .owner_identity()
                        .durable()
                        .expect("directory commit owner"),
                    publication.composite_commit().ordinal(),
                ),
                (
                    publication
                        .publication_attempt()
                        .owner_identity()
                        .durable()
                        .expect("directory attempt owner"),
                    publication.publication_attempt().ordinal(),
                ),
                publication.relational_commit(),
                publication.relational_posture(),
                publication.signal_posture(),
                publication.signal_publication(),
            ),
            (
                basis.descriptor_version(),
                basis.branch_id(),
                basis.reference().generation(),
                basis.truth_version(),
                basis.schema_commitment(),
                basis.visibility_commitment(),
                basis.posture(),
            ),
        )
    )
}
