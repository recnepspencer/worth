use worth_store::physical_runtime::{FundedRecoveryWalObservations, ObservedWalArtifact};

fn escape(observed: &FundedRecoveryWalObservations) -> ObservedWalArtifact {
    <ObservedWalArtifact as Clone>::clone(&observed.artifacts()[0])
}

fn main() {}
