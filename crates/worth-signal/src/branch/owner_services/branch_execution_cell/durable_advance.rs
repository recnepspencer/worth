//! Same-cell callback execution is isolated from every other branch's callback.
use super::super::{SignalBranchCellState, SignalOwner, SignalOwnerOperationAdmission};
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};
use crate::branch::owner_services::durable_publication::prepared::SignalPreparedDurableAdvance;
use crate::branch::owner_services::durable_publication::SignalDurableAdvanceDenial as D;
use crate::branch::AdmittedSignalBranchBasis;
use crate::data::error::SignalError;
use crate::logic::transaction::SignalTransaction;
use worth_execution::ExecutionRequest;

impl<Dm, I, T> SignalBranchExecutionCell<SignalBranchCellState<Dm, I, T>>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn prepare_durable_advance<E, Ctx, F>(
        &self,
        owner: &SignalOwner<Dm, I, T>,
        admission: &SignalOwnerOperationAdmission<'_>,
        execution: ExecutionRequest<'_, '_>,
        expected: &AdmittedSignalBranchBasis,
        runtime_ctx: &mut Ctx,
        recorded_nodes: worth_execution::ExecutionOrderedMap<u32, u32>,
        apply: F,
    ) -> Result<SignalPreparedDurableAdvance<Dm, I, T>, D>
    where
        F: FnOnce(&mut SignalTransaction<'_, Dm, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let map = |denial| {
            D::Mutation(super::advance::map_advance_cell_denial(
                denial,
                self.branch_id,
            ))
        };
        self.validate_admission(admission).map_err(map)?;
        let _hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(map)?;
        self.counters.record_target_cell_contact();
        self.contacts
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut state = self
            .lock_state_after_contention_observation()
            .map_err(map)?;
        self.require_live_posture().map_err(map)?;
        let live = state.observation().map_err(|error| {
            D::Mutation(
                crate::branch::SignalBranchAdvanceDenial::MutationFailedNoMovement { error },
            )
        })?;
        if !owner.is_current_canonical_basis(
            expected,
            self.branch_id,
            self.incarnation().get(),
            &live,
        ) {
            return Err(D::Source(
                super::super::durable_source_port::SignalDurableSourceDenial::ForeignBasis,
            ));
        }
        let scope =
            super::super::conditional_execution::SignalConditionalOperationScopeBinding::issue(
                expected,
                self.incarnation(),
                state.state().installed_definition().cloned(),
            );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.prepare_durable_advance(execution, runtime_ctx, scope, recorded_nodes, apply)
        }));
        // Only the candidate was mutable. Release the actual canonical mutex
        // before propagating a caller unwind, so it is not falsely quarantined.
        drop(state);
        match result {
            Ok(Ok(result)) => result,
            Ok(Err(unwind)) => unwind.resume(),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
}

impl<Dm, I, T> SignalBranchExecutionCell<SignalBranchCellState<Dm, I, T>>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(in crate::branch::owner_services) fn commit_durable_advance(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        proof: worth_proof::DurablyCommitted<
            super::super::durable_publication::medium_binding::SignalCommittedBranchAdvance<
                Dm,
                I,
                T,
            >,
        >,
    ) -> (
        Box<crate::logic::transaction::BranchState<Dm, I, T>>,
        Box<crate::logic::transaction::TransactionResult>,
        crate::branch::SignalBranchObservation,
        crate::durability::publication_binding::SignalCommittedAdvanceRecord,
    ) {
        self.validate_admission(admission)
            .expect("the original owner admission remains held through committed cutover");
        let _hold = admission
            .hold_branch_cell()
            .expect("committed cutover contacts exactly its original branch");
        let mut state = self
            .lock_state_after_contention_observation()
            .expect("the reserved canonical cell remains available");
        self.require_live_posture()
            .expect("the original reservation excludes branch retirement");
        let result = state.commit_durable_advance(proof);
        self.counters
            .record_diagnostic_publication(result.1.diagnostic_publication_work);
        self.counters.record_canonical_movement();
        self.movements
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        drop(state);
        result
    }
}
