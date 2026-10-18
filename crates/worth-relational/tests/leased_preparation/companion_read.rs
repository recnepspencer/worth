//! The prepared native owner retains the read image through actual cutover.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_relational::facade::{
    mvcc::{
        CompanionBranchCell, CompanionCellEditStop, CompanionPreflightBudget,
        CompanionPreflightStop, PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
        RelationalPublicationCompanion, RelationalPublicationDeferred,
        RelationalPublicationOutcome,
    },
    runtime::RelationalRuntime,
};

struct Publisher {
    cell: CompanionBranchCell<u64>,
    edit_under_reservation: AtomicBool,
    refused_edits: AtomicUsize,
}

impl std::fmt::Debug for Publisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeImagePublisher")
    }
}

impl RelationalPublicationCompanion for Publisher {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let reserved = self.cell.reserve_preflight(context)?;
        if self.edit_under_reservation.load(Ordering::Acquire) {
            let result = self
                .cell
                .replace_derived_root_at_same_position(self.cell.read_image(), Arc::new(99));
            assert!(matches!(
                result,
                Err(CompanionCellEditStop::PreflightPending)
            ));
            self.refused_edits.fetch_add(1, Ordering::AcqRel);
        }
        let payload = Arc::clone(reserved.current().payload());
        context.seal_replacement(reserved, payload)
    }
}

fn install(
    runtime: &RelationalRuntime,
) -> (
    worth_relational::facade::mvcc::PublicationCompanionRegistration,
    Arc<Publisher>,
    CompanionBranchCell<u64>,
) {
    let pending = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap();
    let cell = pending
        .with_branch_cell_at_head(
            runtime,
            &runtime.main_branch_identity(),
            Arc::new(1),
            |cell| cell,
        )
        .unwrap();
    let other = pending
        .with_branch_cell_at_head(
            runtime,
            &runtime.main_branch_identity(),
            Arc::new(1),
            |cell| cell,
        )
        .unwrap();
    let publisher = Arc::new(Publisher {
        cell,
        edit_under_reservation: AtomicBool::new(false),
        refused_edits: AtomicUsize::new(0),
    });
    let registration = pending
        .activate(
            publisher.clone(),
            CompanionPreflightBudget {
                maximum_work_visits: 1_000_000,
                maximum_preparation_bytes: 32 * 1024 * 1024,
            },
        )
        .unwrap();
    (registration, publisher, other)
}

fn candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: AspectFieldPatch::default(),
                },
            ))),
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}

fn assert_image_refusal(outcome: RelationalPublicationOutcome, expected: CompanionPreflightStop) {
    assert!(
        matches!(outcome, RelationalPublicationOutcome::Deferred(
        RelationalPublicationDeferred::CompanionPreflight(actual)) if actual == expected),
        "{outcome:?}"
    );
}

#[test]
fn native_companion_read_image_rejects_replacement_and_admits_original_owner_retry() {
    let _serial = serial();
    let host = crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();
    let (_registration, publisher, other) = install(&runtime);
    for (ordinal, replacement) in [(0, 2), (1, 2)] {
        let observed = publisher.cell.read_image();
        publisher
            .cell
            .replace_derived_root_at_same_position(
                publisher.cell.read_image(),
                Arc::new(replacement),
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
            CompanionPreflightStop::SelectedSourceMismatch,
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
        CompanionPreflightStop::ForeignCell,
    );
    let foreign = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();
    let (_foreign_registration, foreign_publisher, _) = install(&foreign);
    let mut prepared = candidate(execution, &runtime, "original-owner-positive");
    assert_eq!(
        prepared
            .require_companion_image(&foreign_publisher.cell, foreign_publisher.cell.read_image()),
        Err(CompanionPreflightStop::ForeignCell)
    );
    // A failed foreign condition does not consume or replace this original owner.
    prepared
        .require_companion_image(&publisher.cell, publisher.cell.read_image())
        .unwrap();
    publisher
        .edit_under_reservation
        .store(true, Ordering::Release);
    let outcome = runtime
        .publication_port()
        .compare_and_publish(execution, prepared);
    let RelationalPublicationOutcome::Performed(performed) = outcome else {
        panic!("{outcome:?}");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    assert_eq!(publisher.refused_edits.load(Ordering::Acquire), 1);
    assert_eq!(**publisher.cell.read_image().payload(), 2);
    // After cutover released the reservation, the original cell can advance.
    publisher
        .cell
        .replace_derived_root_at_same_position(publisher.cell.read_image(), Arc::new(3))
        .unwrap();
    assert_eq!(**publisher.cell.read_image().payload(), 3);
}
