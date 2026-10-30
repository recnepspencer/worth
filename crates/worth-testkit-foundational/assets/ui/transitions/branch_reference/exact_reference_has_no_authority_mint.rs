use worth_foundational::{
    FoundationalBranchId, FoundationalBranchReferenceGeneration,
    FoundationalBranchReferenceObservation, FoundationalBranchTarget,
    FoundationalBranchTargetBasis, FoundationalBranchTargetEncoding,
};

#[derive(Clone, PartialEq, Eq, serde::Serialize)]
struct Target;

impl FoundationalBranchTargetBasis for Target {
    fn canonical_encoding(&self) -> FoundationalBranchTargetEncoding {
        FoundationalBranchTargetEncoding::new("test", 1, Vec::new()).expect("valid")
    }
}

fn main() {
    let observation = FoundationalBranchReferenceObservation::new(
        FoundationalBranchId::new("main").expect("valid branch"),
        FoundationalBranchTarget::basis(Target),
        FoundationalBranchReferenceGeneration::initial(),
    );
    let _authority = observation.mint_authority();
}
