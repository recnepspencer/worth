use crate::state::SignalBranchId;

use super::super::{
    SignalBranchCellState, SignalBranchRegistryDenial, SignalOwnerOperationAdmission,
};
use super::SignalOwner;

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn replace_branch_incarnation_for_test(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<(), SignalBranchRegistryDenial> {
        let original = self.registry.lookup(admission, branch_id)?;
        let (handle, state, generation, restore_snapshot_id) = original
            .with_state(admission, |state, _| {
                (
                    state.handle().clone(),
                    state.state().clone(),
                    state.head_generation(),
                    state.restore_snapshot_id(),
                )
            })
            .map_err(SignalBranchRegistryDenial::TargetCellDenied)?;
        self.registry
            .begin_retirement(admission, branch_id)?
            .execute(|_, _| Ok::<(), ()>(()))?
            .expect("test-only incarnation retirement is infallible");
        self.registry
            .reserve_named(admission, branch_id, handle.name.clone())?
            .install(SignalBranchCellState::new(
                handle,
                self.runtime_instance_id,
                self.definition_basis,
                state,
                generation,
                restore_snapshot_id,
            ))?;
        Ok(())
    }
}
