use worth_store::physical_runtime::{AdmittedPhysicalRuntime, PhysicalStore, RuntimeIdentity};

fn duplicate_runtime(runtime: AdmittedPhysicalRuntime) {
    let _duplicate = runtime.clone();
}

fn reconstruct_from_identity(identity: RuntimeIdentity) {
    let _runtime = PhysicalStore::admit(identity);
}

fn main() {}
