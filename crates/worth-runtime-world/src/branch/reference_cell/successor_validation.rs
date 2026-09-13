use super::{
    ProductBranchHeadProtection, ProductBranchReferenceCellDenial, ProductBranchReferenceSnapshot,
};

pub(super) fn validate_successor(
    current: &ProductBranchReferenceSnapshot,
    successor: &ProductBranchHeadProtection,
) -> Result<(), ProductBranchReferenceCellDenial> {
    let successor_snapshot = successor.snapshot();
    if successor_snapshot.owner() != current.owner() {
        return Err(ProductBranchReferenceCellDenial::SuccessorOwnerMismatch);
    }
    if successor_snapshot.branch() != current.branch() {
        return Err(ProductBranchReferenceCellDenial::SuccessorBranchMismatch);
    }
    if successor_snapshot.lifecycle() != current.lifecycle() {
        return Err(ProductBranchReferenceCellDenial::SuccessorLifecycleMismatch);
    }
    let expected_generation = current
        .generation()
        .advance()
        .map_err(|_| ProductBranchReferenceCellDenial::GenerationExhausted)?;
    if successor_snapshot.generation() != expected_generation {
        return Err(
            ProductBranchReferenceCellDenial::SuccessorGenerationMismatch {
                expected: expected_generation,
                actual: successor_snapshot.generation(),
            },
        );
    }
    if successor.product_head().owner_identity() != successor_snapshot.owner()
        || !successor
            .product_head()
            .matches_basis(successor_snapshot.commit().basis())
        || successor.product_head_history().owner_identity() != successor_snapshot.owner()
        || !successor
            .product_head_history()
            .matches_commit(successor_snapshot.commit())
        || successor.transfer_receipt().is_none()
    {
        return Err(ProductBranchReferenceCellDenial::SuccessorProtectionMismatch);
    }
    Ok(())
}
