use worth_foundational::{
    FoundationalBranchCandidateForkBasis, FoundationalBranchForkBasis, FoundationalBranchId,
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
    let candidate = FoundationalBranchCandidateForkBasis::new(
        FoundationalBranchId::new("main").expect("valid branch"),
        worth_foundational::BoundaryEpoch::new(4),
    );
    let _exact: FoundationalBranchForkBasis<Target> = FoundationalBranchForkBasis::new(candidate);
}
