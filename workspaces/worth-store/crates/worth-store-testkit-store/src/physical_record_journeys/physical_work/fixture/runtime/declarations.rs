pub use {std::path::Path};

pub use {worth_store::physical_runtime::PhysicalWorkProfileDeclaration, worth_store::physical_runtime::ServingPhysicalRuntime};

pub fn serving_from_initialization_with_work_profile(
    root: &Path,
    profile: PhysicalWorkProfileDeclaration,
) -> ServingPhysicalRuntime {
    let (format, placement, access) = crate::physical_record_journeys::configuration();
    crate::physical_record_journeys::success(initialize_record_store!(
        crate::physical_record_journeys::media(root),
        |durability| {
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format, placement, access, durability,
            )
            .with_physical_work_profile(profile)
        },
    ))
}

pub fn serving_from_open_with_work_profile(
    root: &Path,
    profile: PhysicalWorkProfileDeclaration,
) -> ServingPhysicalRuntime {
    let (format, _, access) = crate::physical_record_journeys::configuration();
    crate::physical_record_journeys::success(open_record_store!(
        crate::physical_record_journeys::media(root),
        |durability| worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability
        )
        .with_physical_work_profile(profile),
    ))
}