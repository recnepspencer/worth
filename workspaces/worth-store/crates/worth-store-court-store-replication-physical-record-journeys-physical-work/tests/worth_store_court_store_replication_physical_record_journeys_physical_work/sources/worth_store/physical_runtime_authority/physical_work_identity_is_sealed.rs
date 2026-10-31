use worth_store::physical_runtime::{
    PhysicalOperationIdentity, PhysicalWorkGeneration, PhysicalWorkIdentity, RuntimeIdentity,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

fn forge_identity(
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    generation: PhysicalWorkGeneration,
    operation: PhysicalOperationIdentity,
) -> PhysicalWorkIdentity {
    PhysicalWorkIdentity {
        store,
        runtime,
        generation,
        operation,
    }
}

fn main() {}
