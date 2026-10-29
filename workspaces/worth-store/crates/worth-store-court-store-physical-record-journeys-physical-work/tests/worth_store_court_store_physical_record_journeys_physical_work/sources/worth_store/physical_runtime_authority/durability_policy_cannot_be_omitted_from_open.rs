use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, PhysicalRecordOpen,
};

fn omit(
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
) -> PhysicalRecordOpen {
    PhysicalRecordOpen::new(format, access)
}

fn main() {}
