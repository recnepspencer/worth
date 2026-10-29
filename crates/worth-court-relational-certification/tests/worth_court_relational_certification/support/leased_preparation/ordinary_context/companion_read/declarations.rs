// The prepared native owner retains the read image through actual cutover.
pub(crate) use {worth_execution::ExecutionAllocationPolicy as AllocationPolicy, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::identity::KindId, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::identity::PartitionId, worth_relational::facade::runtime::RelationalRuntimeApi, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_relational::facade::transactions::WorkerIntentBatch, crate::leased_preparation::ordinary_context::serial, crate::leased_preparation::ordinary_context::support};
pub(crate) use {std::sync::atomic::AtomicBool, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};
pub(crate) use {worth_relational::facade::mvcc::CompanionBranchCell, worth_relational::facade::mvcc::CompanionCellEditStop, worth_relational::facade::mvcc::CompanionPreflightStop, worth_relational::facade::mvcc::PreparedPublicationCompanionEffect, worth_relational::facade::mvcc::PublicationCompanionPreflight, worth_relational::facade::mvcc::RelationalPublicationCompanion, worth_relational::facade::mvcc::RelationalPublicationDeferred, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::runtime::RelationalRuntime};

pub(crate) struct Publisher {
    pub(crate) cell: CompanionBranchCell<u64>,
    pub(crate) edit_under_reservation: AtomicBool,
    pub(crate) refused_edits: AtomicUsize,
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
        if self.edit_under_reservation.load(std::sync::atomic::Ordering::Acquire) {
            let result = self
                .cell
                .replace_derived_root_at_same_position(self.cell.read_image(), std::sync::Arc::new(99));
            assert!(matches!(
                result,
                Err(worth_relational::facade::mvcc::CompanionCellEditStop::PreflightPending)
            ));
            self.refused_edits.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        }
        let payload = std::sync::Arc::clone(reserved.current().payload());
        context.seal_replacement(reserved, payload)
    }
}

pub(crate) fn install(
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
            std::sync::Arc::new(1),
            |cell| cell,
        )
        .unwrap();
    let other = pending
        .with_branch_cell_at_head(
            runtime,
            &runtime.main_branch_identity(),
            std::sync::Arc::new(1),
            |cell| cell,
        )
        .unwrap();
    let publisher = std::sync::Arc::new(Publisher {
        cell,
        edit_under_reservation: std::sync::atomic::AtomicBool::new(false),
        refused_edits: std::sync::atomic::AtomicUsize::new(0),
    });
    let registration = pending.activate(publisher.clone()).unwrap();
    (registration, publisher, other)
}

pub(crate) fn candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new(key).push(worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Entity(
                EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(key),
                    fields: worth_relational::facade::transactions::AspectFieldPatch::default(),
                },
            ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}

pub(crate) fn assert_image_refusal(
    outcome: RelationalPublicationOutcome,
    expected: CompanionPreflightStop,
) {
    assert!(
        matches!(outcome, worth_relational::facade::mvcc::RelationalPublicationOutcome::Deferred(
        worth_relational::facade::mvcc::RelationalPublicationDeferred::CompanionPreflight(actual)) if actual == expected),
        "{outcome:?}"
    );
}