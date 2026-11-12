//! Prepare one exact captured destination and release its source custody.
use super::*;

impl<'a, D, I, T> SignalOwnerForkCellBuilder<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn source_matches(
        &self,
        cell: &SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>,
    ) -> bool {
        self.source_custody
            .as_ref()
            .is_some_and(|custody| custody.matches(cell))
    }

    pub(crate) fn destination(&self) -> &SignalBranchHandle {
        &self.handle
    }

    pub(crate) fn prepare_cell(
        &mut self,
        state: BranchState<D, I, T>,
    ) -> Result<
        SignalPreparedBranchCell<SignalBranchCellState<D, I, T>>,
        SignalBranchForkOperationDenial,
    > {
        if state.branch_id() != self.handle.id {
            return Err(owner_fork_invariant(
                "fork destination state does not match its owner-issued reservation",
            ));
        }
        let destination_branch_id = self.handle.id;
        let destination_state = SignalBranchCellState::new(
            self.handle.clone(),
            self.owner_runtime_instance_id,
            self.definition_basis,
            state,
            0,
            None,
        );
        let destination_observation = destination_state
            .observation()
            .map_err(|error| SignalBranchForkOperationDenial::OwnerDeniedNoMovement { error })?;
        let prepared = self
            .reservation
            .prepare_fork_destination_cell(destination_state)
            .map_err(|denial| map_fork_registry_denial(denial, destination_branch_id))?;
        self.destination_observation = Some(destination_observation);
        Ok(prepared)
    }

    pub(crate) fn validate_prepared_cell(
        &self,
        prepared: &SignalPreparedBranchCell<SignalBranchCellState<D, I, T>>,
    ) -> Result<(), SignalBranchForkOperationDenial> {
        self.reservation
            .matches_prepared_fork_destination(prepared)
            .then_some(())
            .ok_or_else(|| {
                owner_fork_invariant(
                    "prepared fork destination does not match its exact owner reservation",
                )
            })
    }

    pub(crate) fn bind_prepared_cell(
        mut self,
        prepared: SignalPreparedBranchCell<SignalBranchCellState<D, I, T>>,
    ) -> SignalPreparedOwnerFork<'a, D, I, T> {
        drop(
            self.source_custody
                .take()
                .expect("prepared fork capture retains source custody until capture completes"),
        );
        let admission = self.reservation.admission();
        SignalPreparedOwnerFork {
            handle: self.handle,
            admission,
            installation: self.reservation.bind_prepared_fork_destination(prepared),
            lineage: self.lineage,
            destination_observation: self
                .destination_observation
                .expect("prepared fork destination must carry its sealed observation"),
        }
    }
}
