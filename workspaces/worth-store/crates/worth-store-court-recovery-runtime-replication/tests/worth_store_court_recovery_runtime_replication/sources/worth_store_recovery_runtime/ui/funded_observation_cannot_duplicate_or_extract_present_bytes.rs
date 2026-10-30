use worth_store::physical_runtime::{FundedRecoveryObservation, ObservedRecoveryArtifact};

fn clone_funded(observation: FundedRecoveryObservation) {
    let _ = observation.clone();
}

fn clone_raw(observation: &FundedRecoveryObservation) -> ObservedRecoveryArtifact {
    observation.observed().clone()
}

fn extract_bytes(observation: &FundedRecoveryObservation) {
    let _ = observation.observed().into_bytes();
}

fn main() {}
