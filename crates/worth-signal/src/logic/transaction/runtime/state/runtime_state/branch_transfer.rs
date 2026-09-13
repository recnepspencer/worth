use super::{
    AuthorityTransferPacket, BranchLifecycleTransfer, RestoreTransferPacket, SignalRuntime,
};

#[cfg(test)]
mod tests;

pub(in crate::logic::transaction::runtime::state) struct ReadyBranchLifecycleTransfer<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    transfer: BranchLifecycleTransfer<D, I, T>,
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn commit_branch_state(
        &mut self,
        packet: AuthorityTransferPacket<D, I, T>,
        count_temporal_restore: bool,
    ) {
        let preserved_transaction = self.telemetry_snapshot().transaction;
        let state = packet.into_state();
        let count_temporal_restore = count_temporal_restore
            && self.graph.captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            );
        self.branches.restore_active_state(
            state,
            &mut self.graph,
            &mut self.config,
            &mut self.checkpoint,
            &mut self.resource,
            &mut self.temporal,
            &mut self.telemetry,
            count_temporal_restore,
        );
        Self::merge_global_transaction_telemetry(
            preserved_transaction,
            &mut self.telemetry.transaction,
        );
    }

    fn commit_restored_branch_state(&mut self, packet: RestoreTransferPacket<D, I, T>) {
        self.with_telemetry(|telemetry| telemetry.transaction.restore_transfer_count += 1);
        self.commit_branch_state(
            AuthorityTransferPacket::new(packet.branch_id(), packet.into_state()),
            true,
        );
    }

    pub(in crate::logic::transaction::runtime::state) fn prepare_branch_lifecycle_transfer(
        &self,
        transfer: BranchLifecycleTransfer<D, I, T>,
    ) -> Result<ReadyBranchLifecycleTransfer<D, I, T>, crate::data::error::SignalError> {
        let (branch_id, state) = match &transfer {
            BranchLifecycleTransfer::Move(packet) => (packet.branch_id(), packet.state()),
            BranchLifecycleTransfer::Restore(packet) => (packet.branch_id(), packet.state()),
        };
        if branch_id != state.ancestry().branch_id() {
            return Err(crate::data::error::SignalError::internal(format!(
                "branch lifecycle transfer mismatch: packet branch {} does not match state branch {}",
                branch_id.0,
                state.ancestry().branch_id().0
            )));
        }
        Self::ensure_managed_queue_branch_transfer_allowed(&self.resource)?;
        Self::ensure_managed_queue_branch_transfer_allowed(state.resource())?;
        Ok(ReadyBranchLifecycleTransfer { transfer })
    }

    pub(in crate::logic::transaction::runtime::state) fn commit_branch_lifecycle_transfer(
        &mut self,
        ready: ReadyBranchLifecycleTransfer<D, I, T>,
    ) {
        match ready.transfer {
            BranchLifecycleTransfer::Move(packet) => self.commit_branch_state(packet, false),
            BranchLifecycleTransfer::Restore(packet) => self.commit_restored_branch_state(packet),
        }
    }

    pub(in crate::logic::transaction::runtime::state) fn apply_branch_lifecycle_transfer(
        &mut self,
        transfer: BranchLifecycleTransfer<D, I, T>,
    ) -> Result<(), crate::data::error::SignalError> {
        let ready = self.prepare_branch_lifecycle_transfer(transfer)?;
        self.commit_branch_lifecycle_transfer(ready);
        Ok(())
    }

    pub(in crate::logic::transaction::runtime::state) fn project_branch_catalog(&mut self) {
        let active_branch = self.graph.current_branch().id;
        self.branches
            .project_catalog(active_branch, &mut self.graph);
    }
}
