//! Actual Home custody reaches the persisted no-pending admission boundary.
mod journal_absence;
mod unresolved;

use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, ManifestEntryCapacity,
    OrderedJournalName, OrderedJournalPublication, OrderedJournalPublicationFailure,
    OrderedJournalPublished, PhysicalMutationDeadline, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, ServingPhysicalRuntime,
};
use worth_store_application::{ApplicationHomeCloseOutcome, WorldReadmittedApplicationHome};

fn open(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixture {
    worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    )
    .expect("actual filesystem Native/World owner admission")
}

fn placement() -> AdmittedRecordPlacementPolicy {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap()
}

fn publish_entry(
    serving: &ServingPhysicalRuntime,
    name: &'static str,
) -> Result<OrderedJournalPublished, OrderedJournalPublicationFailure> {
    serving
        .ordered_journal(OrderedJournalName::new(name).unwrap())
        .expect("the reserved journal belongs to this actual Home")
        .publish(OrderedJournalPublication::new(
            1,
            b"unsupported-owner-artifact".to_vec(),
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
}

fn close(
    home: WorldReadmittedApplicationHome<(), (), (), (), ()>,
    execution: &ExecutionRequest<'_, '_>,
) {
    let sealed = home
        .seal_world(execution)
        .unwrap_or_else(|refused| panic!("quiescent original World seals: {:?}", refused.reason()));
    assert!(matches!(
        sealed.finish_close(execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

fn assert_contended(location: &std::path::Path, execution: &ExecutionRequest<'_, '_>) {
    match crate::durable_run::open(location, execution) {
        Err(worth_store_application::ApplicationHomeOpenRefusal::Physical(refused)) => {
            assert_eq!(
                refused.kind(),
                worth_store_application::ApplicationHomePhysicalRefusalKind::MutationOwnerContended
            );
        }
        Err(cause) => panic!("the retained original lease must refuse first: {cause:?}"),
        Ok(_) => panic!("admission custody must retain its actual mutation lease"),
    }
}
