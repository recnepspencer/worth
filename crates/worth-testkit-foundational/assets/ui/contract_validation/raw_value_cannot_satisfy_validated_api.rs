use worth_foundational::{AspectValue, ContractValidatedAspectArtifact};

fn requires_validated(_artifact: ContractValidatedAspectArtifact) {}

fn main() {
    requires_validated(AspectValue::Bool(true));
}
