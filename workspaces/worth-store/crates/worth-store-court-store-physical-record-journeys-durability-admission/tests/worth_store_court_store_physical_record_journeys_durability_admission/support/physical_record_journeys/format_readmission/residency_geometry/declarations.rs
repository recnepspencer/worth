//! Policy geometry must reject before constructing ordinary Store owners.

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalPageSizeClass, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::RecordBootstrapDenial};

pub(crate) fn foreign_format_policy() -> AdmittedPhysicalRecordResidencyPolicy {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .page_size(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    );
    worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy::canonical(format)
}