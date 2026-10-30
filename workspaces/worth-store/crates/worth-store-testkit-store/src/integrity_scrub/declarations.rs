pub use {std::path::Path, std::time::Duration};
pub use {
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress as Progress,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest as Request,
    worth_store::physical_runtime::PhysicalIntegrityScrubRequestDenial as Denial,
    worth_store::physical_runtime::PhysicalIntegrityScrubTarget as Target,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalWorkCounterStage,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkPressureClass,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub use {
    worth_store_physical_format::PhysicalArtifactReadTarget,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::RecordArtifactFile,
};
pub use {
    worth_store_physical_integrity::PhysicalArtifactScope,
    worth_store_physical_integrity::PhysicalByteRange,
    worth_store_physical_integrity::PhysicalDamageCause,
    worth_store_physical_integrity::PhysicalIntegrityObservationOutcome,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};

pub fn target(serving: &ServingPhysicalRuntime, root: &Path) -> Target {
    let length = std::fs::metadata(root.join("families/records/root-current.selector"))
        .unwrap()
        .len();
    worth_store::physical_runtime::PhysicalIntegrityScrubTarget::new(
        worth_store_physical_format::PhysicalArtifactReadTarget::Record(
            worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
        ),
        worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(
            serving.store_identity(),
            worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
                .admit()
                .unwrap(),
            worth_store_physical_integrity::PhysicalByteRange::new(0, length).unwrap(),
        ),
    )
    .unwrap()
}
pub fn request(serving: &ServingPhysicalRuntime, target: Target) -> Request {
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        std::time::Duration::from_secs(30),
    )
    .unwrap()
}
pub fn assert_released(serving: &ServingPhysicalRuntime) {
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(
                worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub
            ),
        0
    );
    assert_eq!(
        serving.physical_scheduler_capacity().active_reservations(),
        0
    );
}
