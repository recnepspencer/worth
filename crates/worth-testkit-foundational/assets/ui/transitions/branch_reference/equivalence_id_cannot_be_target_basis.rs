use worth_foundational::{
    EquivalenceBasisId, FoundationalBranchId, FoundationalBranchReferenceGeneration,
    FoundationalBranchReferenceObservation, FoundationalBranchTarget,
};

fn main() {
    let _observation = FoundationalBranchReferenceObservation::new(
        FoundationalBranchId::new("main").expect("valid branch"),
        FoundationalBranchTarget::basis(EquivalenceBasisId::new(7)),
        FoundationalBranchReferenceGeneration::initial(),
    );
}
