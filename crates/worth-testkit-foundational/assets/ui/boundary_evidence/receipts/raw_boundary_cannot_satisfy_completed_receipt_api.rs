use worth_foundational::{
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalCommitId, FoundationalCommitParentBasis,
    FoundationalCommitParentageLocator, FoundationalTransitionLocator,
};

fn main() {
    let boundary = FoundationalBoundaryEvidenceReceiptBoundary::transition(
        FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
            FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(1)),
            FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(1)),
        )),
    );

    let _ = boundary.completed_boundary();
}
