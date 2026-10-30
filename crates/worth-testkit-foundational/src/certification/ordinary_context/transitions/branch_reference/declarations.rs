pub use {serde::Deserialize, serde::Serialize};
pub use {
    worth_foundational::FoundationalBranchComparisonBasis,
    worth_foundational::FoundationalBranchForkBasis, worth_foundational::FoundationalBranchId,
    worth_foundational::FoundationalBranchReferenceGeneration,
    worth_foundational::FoundationalBranchReferenceGenerationAdvanceDenial,
    worth_foundational::FoundationalBranchReferenceMismatchAxis,
    worth_foundational::FoundationalBranchReferenceMovement,
    worth_foundational::FoundationalBranchReferenceMovementKind,
    worth_foundational::FoundationalBranchReferenceObservation,
    worth_foundational::FoundationalBranchTarget,
    worth_foundational::FoundationalBranchTargetBasis,
    worth_foundational::FoundationalBranchTargetEncoding,
    worth_foundational::FoundationalBranchTargetEncodingConstructionDenial,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalTarget {
    pub runtime: String,
    pub commit: u64,
    pub truth_root: u64,
}

impl FoundationalBranchTargetBasis for RelationalTarget {
    fn canonical_encoding(&self) -> FoundationalBranchTargetEncoding {
        worth_foundational::FoundationalBranchTargetEncoding::new(
            "worth.relational.commit-root",
            1,
            format!("{}:{}:{}", self.runtime, self.commit, self.truth_root).into_bytes(),
        )
        .expect("test target encoding is valid")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalTarget {
    pub graph: String,
    pub definition: u64,
    pub snapshot: u64,
}

impl FoundationalBranchTargetBasis for SignalTarget {
    fn canonical_encoding(&self) -> FoundationalBranchTargetEncoding {
        worth_foundational::FoundationalBranchTargetEncoding::new(
            "worth.signal.definition-snapshot",
            1,
            format!("{}:{}:{}", self.graph, self.definition, self.snapshot).into_bytes(),
        )
        .expect("test target encoding is valid")
    }
}

pub fn branch(name: &str) -> FoundationalBranchId {
    worth_foundational::FoundationalBranchId::new(name).expect("test branch id is valid")
}

pub fn relational_target(runtime: &str, commit: u64, truth_root: u64) -> RelationalTarget {
    RelationalTarget {
        runtime: runtime.to_owned(),
        commit,
        truth_root,
    }
}

pub fn relational_observation(
    branch_name: &str,
    target: FoundationalBranchTarget<RelationalTarget>,
    generation: u64,
) -> FoundationalBranchReferenceObservation<RelationalTarget> {
    worth_foundational::FoundationalBranchReferenceObservation::new(
        branch(branch_name),
        target,
        worth_foundational::FoundationalBranchReferenceGeneration::new(generation),
    )
}

pub fn hex_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("hex is utf8"), 16)
                .expect("valid hex")
        })
        .collect()
}
