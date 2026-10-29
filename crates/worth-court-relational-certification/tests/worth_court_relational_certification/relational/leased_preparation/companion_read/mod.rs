// The prepared native owner retains the read image through actual cutover.
#[allow(unused_imports)] use crate::leased_preparation::ordinary_context::companion_read::*;
pub(crate) use {worth_execution::ExecutionAllocationPolicy as AllocationPolicy, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::identity::KindId, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::identity::PartitionId, worth_relational::facade::runtime::RelationalRuntimeApi, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_relational::facade::transactions::WorkerIntentBatch, crate::leased_preparation::ordinary_context::serial, crate::leased_preparation::ordinary_context::support};
pub(crate) use {std::sync::atomic::AtomicBool, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};
pub(crate) use {worth_relational::facade::mvcc::CompanionBranchCell, worth_relational::facade::mvcc::CompanionCellEditStop, worth_relational::facade::mvcc::CompanionPreflightStop, worth_relational::facade::mvcc::PreparedPublicationCompanionEffect, worth_relational::facade::mvcc::PublicationCompanionPreflight, worth_relational::facade::mvcc::RelationalPublicationCompanion, worth_relational::facade::mvcc::RelationalPublicationDeferred, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::runtime::RelationalRuntime};

#[test]
fn native_companion_read_image_rejects_replacement_and_admits_original_owner_retry() {
    let _serial = serial();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(crate::leased_preparation::ordinary_context::support::demo_schema_registry())
        .build()
        .expect("valid native runtime configuration");
    let (_registration, publisher, other) = install(&runtime);
    for (ordinal, replacement) in [(0, 2), (1, 2)] {
        let observed = publisher.cell.read_image();
        publisher
            .cell
            .replace_derived_root_at_same_position(
                publisher.cell.read_image(),
                std::sync::Arc::new(replacement),
            )
            .unwrap();
        let before = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0;
        let mut prepared = candidate(execution, &runtime, &format!("image-refused-{ordinal}"));
        prepared
            .require_companion_image(&publisher.cell, observed)
            .unwrap();
        assert_image_refusal(
            runtime
                .publication_port()
                .compare_and_publish(execution, prepared),
            worth_relational::facade::mvcc::CompanionPreflightStop::SelectedSourceMismatch,
        );
        assert_eq!(
            runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0,
            before
        );
    }
    let mut wrong = candidate(execution, &runtime, "wrong-native-cell");
    wrong
        .require_companion_image(&other, other.read_image())
        .unwrap();
    assert_image_refusal(
        runtime
            .publication_port()
            .compare_and_publish(execution, wrong),
        worth_relational::facade::mvcc::CompanionPreflightStop::ForeignCell,
    );
    let foreign = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(crate::leased_preparation::ordinary_context::support::demo_schema_registry())
        .build()
        .expect("valid native runtime configuration");
    let (_foreign_registration, foreign_publisher, _) = install(&foreign);
    let mut prepared = candidate(execution, &runtime, "original-owner-positive");
    assert_eq!(
        prepared
            .require_companion_image(&foreign_publisher.cell, foreign_publisher.cell.read_image()),
        Err(worth_relational::facade::mvcc::CompanionPreflightStop::ForeignCell)
    );
    // A failed foreign condition does not consume or replace this original owner.
    prepared
        .require_companion_image(&publisher.cell, publisher.cell.read_image())
        .unwrap();
    publisher
        .edit_under_reservation
        .store(true, std::sync::atomic::Ordering::Release);
    let outcome = runtime
        .publication_port()
        .compare_and_publish(execution, prepared);
    let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) = outcome else {
        panic!("{outcome:?}");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    assert_eq!(publisher.refused_edits.load(std::sync::atomic::Ordering::Acquire), 1);
    assert_eq!(**publisher.cell.read_image().payload(), 2);
    // After cutover released the reservation, the original cell can advance.
    publisher
        .cell
        .replace_derived_root_at_same_position(publisher.cell.read_image(), std::sync::Arc::new(3))
        .unwrap();
    assert_eq!(**publisher.cell.read_image().payload(), 3);
}
