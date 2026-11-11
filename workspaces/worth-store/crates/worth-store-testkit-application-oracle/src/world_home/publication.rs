//! The actual ordinary owner doors; no substitute publication or recovery port.
use super::WorldHomeFixture;
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;

impl WorldHomeFixture {
    /// This fixture installs only the initial programme; transition steps keep A
    /// until activation. Admission still comes from this actual Home owner.
    pub fn admit_initial_program_source(
        &self,
        expected: &ProductBranchObservation,
    ) -> worth_proof::AdmittedBranchProgramSource {
        let revision = *self.home.binding().selected().revision().bytes();
        let commitment = world_branch_program_source_commitment(expected, revision)
            .expect("the actual fixture World head admits its programme source");
        self.home
            .branch_program_source_owner()
            .admit_source(revision, commitment)
    }

    pub fn prepare_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> WorldPreparedPublicationWithoutSignal {
        let native = self.home.relational().runtime();
        let port = self.home.world().publication_port();
        let change = worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary();
        let mut outcome = port.open_publication_without_signal(
            execution,
            expected.clone(),
            self.admit_initial_program_source(&expected),
            change.clone(),
            cancellation,
            None,
        );
        let backstop = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let opened = loop {
            match outcome {
                WorldOpenAttemptOutcome::Opened(opened) => break opened,
                WorldOpenAttemptOutcome::Pending(pending) => {
                    assert!(
                        std::time::Instant::now() < backstop,
                        "actual World open did not settle"
                    );
                    execution
                        .consult()
                        .expect("original opening request remains live");
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    outcome = pending.advance(execution);
                }
                other => panic!("actual World open refused: {other:?}"),
            }
        };
        let mut transaction = native
            .begin_branch_transaction(expected.basis().relational_basis(), change)
            .expect("the original product predecessor admits its transaction");
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        let proposal = transaction
            .validate(execution, native)
            .expect("one actual handler proposal validates");
        opened
            .prepare_native(native, execution, proposal)
            .expect("one actual Native preparation binds the original World open")
    }

    pub fn publish(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prepared: WorldPreparedPublicationWithoutSignal,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> PerformedCompositePublication {
        let mut outcome = self
            .home
            .world()
            .publication_port()
            .execute_durable_without_signal(*execution, prepared, cancellation);
        let backstop = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match outcome {
                WorldDurablePublicationOutcome::Performed(performed) => return performed,
                WorldDurablePublicationOutcome::Pending(pending) => {
                    assert!(
                        std::time::Instant::now() < backstop,
                        "actual World occurrence did not settle"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    outcome = pending.advance(execution);
                }
                other => panic!("actual World mutation refused: {other:?}"),
            }
        }
    }
}
