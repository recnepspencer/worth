use worth_store::physical_runtime::PhysicalRecoveryCoordination;
use worth_store_physical_integrity::VerifiedCheckpointStream;

fn duplicate(stream: VerifiedCheckpointStream) {
    let _ = stream.clone();
}

fn adopt_unfunded(
    coordination: &mut PhysicalRecoveryCoordination,
    stream: VerifiedCheckpointStream,
) {
    let _ = coordination.admit_shared_checkpoint(stream);
}

fn main() {}
