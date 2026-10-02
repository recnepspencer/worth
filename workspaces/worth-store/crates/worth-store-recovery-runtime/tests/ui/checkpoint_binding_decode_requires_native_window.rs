use worth_store::physical_runtime::StoreRecoveryCheckpointBindingRebuilder;
use worth_store_physical_integrity::{
    IntegrityValidatedCheckpointBinding, UntrustedPhysicalArtifact,
};

fn decode_without_owner(
    rebuilder: &mut StoreRecoveryCheckpointBindingRebuilder,
    admitted: &IntegrityValidatedCheckpointBinding<'_>,
    record: UntrustedPhysicalArtifact<'_>,
) {
    rebuilder.consume(admitted, record);
}

fn main() {}
