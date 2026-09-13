use crate::branch::{ProductBranchCreationIntent, RuntimeWorldBootstrapIntent};
use crate::budget::RuntimeWorldBudgets;
use crate::lifecycle::{RuntimeWorldClock, RuntimeWorldOwnerInputs};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::{
    PerformedRelationalCommit, PreparedRelationalCommitCandidate, RelationalPublicationOutcome,
    RelationalTransactionIntent,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

use super::RealReferenceFixture;

impl RealReferenceFixture {
    pub(crate) fn reserve_relational_fork_target(
        &self,
        target: &str,
    ) -> Result<
        worth_relational::facade::branch::RelationalForkTargetReservation,
        worth_relational::facade::branch::RelationalForkDenial,
    > {
        self._relational_runtime
            .fork_port()
            .reserve_fork_target(BranchId(target.to_owned()))
    }

    pub(crate) fn perform_relational_owner_change(&self) -> PerformedRelationalCommit {
        let candidate =
            self.prepare_relational_owner_candidate("runtime-world-cas-loss-owner-effect");
        match self
            ._relational_runtime
            .publication_port()
            .compare_and_publish(candidate)
        {
            RelationalPublicationOutcome::Performed(performed) => performed,
            outcome => {
                panic!("production Relational owner must perform the test change: {outcome:?}")
            }
        }
    }

    pub(crate) fn prepare_relational_owner_candidate(
        &self,
        operation_name: &str,
    ) -> PreparedRelationalCommitCandidate {
        let identity = self._relational_runtime.main_branch_identity();
        let (_, basis) = self
            ._relational_runtime
            .observe_branch(&identity)
            .expect("real Relational owner observes its current basis");
        let services = self._relational_runtime.owner_component_services();
        let mut transaction = services
            .transaction_admission_port()
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .expect("current owner basis admits a transaction");
        transaction
            .push_batch(WorkerIntentBatch::new(operation_name))
            .expect("bounded empty batch stages through the production transaction path");
        services
            .preparation_port()
            .prepare_branch_transaction(transaction)
            .expect("production Relational owner prepares the change")
    }

    /// The Signal owner's own view of the branch it is currently on. A fork
    /// creates a destination without advancing this source, so a creation proof
    /// can hold the two apart.
    pub(crate) fn observe_signal_current_basis(
        &self,
    ) -> worth_signal::facade::branch::AdmittedSignalBranchBasis {
        self._signal_runtime
            .observe_signal_branch_basis(self._signal_runtime.current_branch())
            .expect("real Signal owner observes its current basis")
    }

    pub(crate) fn perform_signal_owner_change(
        &mut self,
    ) -> worth_signal::facade::branch::SignalBranchAdvanceOutcome {
        let services = self
            ._signal_runtime
            .owner_component_services()
            .expect("real Signal owner issues its mutation service");
        let expected = self
            ._signal_runtime
            .observe_signal_branch_basis(self._signal_runtime.current_branch())
            .expect("real Signal owner observes its current basis");
        let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
        services
            .mutation_port()
            .advance_exact(&expected, &mut (), &cancellation.token(), |_| Ok(()))
            .expect("real Signal owner performs the empty bounded transaction")
    }

    #[cfg(feature = "test-operation-control")]
    pub(crate) fn signal_operation_control(
        &self,
    ) -> worth_signal::facade::branch::SignalOwnerOperationControl {
        self._signal_runtime
            .owner_operation_control()
            .expect("real Signal owner exposes operation control")
    }

    #[cfg(feature = "test-operation-control")]
    pub(crate) fn inject_signal_retention_panic(&self) {
        use worth_signal::facade::branch::SignalOwnerOperationBoundary;

        self._signal_runtime
            .owner_operation_control()
            .expect("real Signal owner exposes operation control")
            .inject_panic_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    }

    pub(crate) fn owner_inputs(
        &mut self,
        budgets: RuntimeWorldBudgets,
        clock: RuntimeWorldClock,
    ) -> RuntimeWorldOwnerInputs<(), (), (), (), ()> {
        let relational = self._relational_runtime.owner_component_services();
        let signal = self
            ._signal_runtime
            .owner_component_services()
            .expect("real Signal owner issues its sealed services again");
        let signal_definition_publication = self
            ._signal_runtime
            .runtime_world_definition_publication_port()
            .expect("real Signal owner dedicates publication to its World");
        RuntimeWorldOwnerInputs::new(
            relational,
            signal,
            signal_definition_publication,
            self._correspondence_port.clone(),
            budgets,
            clock,
        )
    }

    pub(crate) fn bootstrap_intent(&self) -> RuntimeWorldBootstrapIntent {
        RuntimeWorldBootstrapIntent::new(
            ProductBranchCreationIntent::named("root").expect("valid root branch name"),
            self.basis.relational_basis().clone(),
            self.basis.signal_basis().clone(),
            self.basis.correspondence_basis().clone(),
        )
    }
}
