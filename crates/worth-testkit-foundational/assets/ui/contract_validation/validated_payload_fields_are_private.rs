use worth_foundational::ContractValidatedAspectValue;

// Probe today's actual carrier field privacy, not obsolete representation names.
fn reconstruct(value: ContractValidatedAspectValue) -> ContractValidatedAspectValue {
    ContractValidatedAspectValue { ..value }
}

fn main() {}
