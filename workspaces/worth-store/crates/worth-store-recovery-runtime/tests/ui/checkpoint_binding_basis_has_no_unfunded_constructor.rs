use worth_store::physical_runtime::{
    PhysicalRecoveryReadAllocation, SharedRecoveryCheckpoint,
    StoreRecoveryCheckpointBindingRebuilder,
};

fn bypass(window: &mut PhysicalRecoveryReadAllocation<'_>, checkpoint: &SharedRecoveryCheckpoint) {
    let _ = StoreRecoveryCheckpointBindingRebuilder::begin;
    let _ = StoreRecoveryCheckpointBindingRebuilder::prepare(window, checkpoint, 1);
}

fn main() {}
