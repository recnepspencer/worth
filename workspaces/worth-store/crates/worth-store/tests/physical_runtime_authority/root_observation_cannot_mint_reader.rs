use worth_store::physical_runtime::{PhysicalProtectedRootObservation, PhysicalRecordReader};

fn counterfeit(root: PhysicalProtectedRootObservation) -> PhysicalRecordReader {
    root.into()
}

fn main() {}
