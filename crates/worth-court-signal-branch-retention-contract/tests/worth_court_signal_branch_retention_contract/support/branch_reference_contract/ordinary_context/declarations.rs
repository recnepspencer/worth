pub(crate) use {
    worth_foundational::FoundationalBranchReferenceGeneration,
    worth_foundational::FoundationalBranchReferenceMismatchAxis,
    worth_foundational::FoundationalBranchTarget,
    worth_foundational::FoundationalBranchTargetBasis,
};
pub(crate) use {
    worth_signal::facade::branch::signal_branch_observation,
    worth_signal::facade::branch::SignalBranchTarget,
};

pub(crate) fn hex_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("hex is utf8"), 16)
                .expect("valid hex")
        })
        .collect()
}
