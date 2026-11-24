//! Reconstruction restores an actual original cell, never a relabelled basis.
use super::*;
use crate::durability::data::recovery_errors::DurabilityError;
use crate::durability::data::recovery_errors::RecoveryFailureClass;

pub(crate) struct WorldSourceSelection<'a> {
    pub(crate) history: &'a crate::runtime::state::subsystems::history::HistorySubsystem,
    pub(crate) visibility: crate::branch::head_visibility_inputs::BranchHeadVisibilityInputs<'a>,
}

impl RelationalBranchReferenceCell {
    pub(crate) fn select_recovered_world_source<'a>(
        &self,
        inputs: impl Into<WorldSourceSelection<'a>>,
        original: &crate::branch::basis::RelationalBranchBasisDescriptor,
        root: Arc<crate::branch::root::RelationalBranchRoot>,
    ) -> Result<Arc<crate::branch::root::RelationalBranchRoot>, DurabilityError> {
        let inputs = inputs.into();
        let corrupt = |message| DurabilityError::new(RecoveryFailureClass::ReplayFailure, message);
        if original.runtime_instance_id() != self.identity.runtime_instance_id()
            || original.branch_id() != self.identity.branch_id()
            || original.posture() != crate::branch::basis::RelationalBranchBasisPosture::Live
        {
            return Err(corrupt(
                "World recovery source belongs to another branch or posture",
            ));
        }
        super::super::basis_axis_validation::require_current_descriptor_axes(
            original,
            original.reference(),
            original.truth_version(),
            &root,
        )
        .map_err(|_| corrupt("World recovery source does not describe its actual root"))?;
        let previous = self
            .root()
            .ok_or_else(|| corrupt("World recovery head has no root"))?;
        let retirement = inputs
            .history
            .retention_binding()
            .reserve_head_retirement_capacity(&self.identity, &previous, &self.head_retention)
            .map_err(|_| corrupt("World recovery head retirement could not be reserved"))?;
        let mut prepared = self.state_snapshot();
        if prepared.lifecycle != crate::branch::lifecycle::RelationalBranchLifecyclePosture::Live {
            return Err(corrupt(
                "World recovery cannot restore an inactive Native branch",
            ));
        }
        let _coordination = self.coordination.enter();
        let publication = self.publication_cell();
        let mut state = publication.enter_state();
        if state.lifecycle_posture() != prepared.lifecycle
            || state.observation() != &prepared.observation
            || state.truth_version() != prepared.truth_version
            || !state
                .selected_root()
                .is_some_and(|head| Arc::ptr_eq(&head, &previous))
        {
            return Err(corrupt("World recovery head moved before source selection"));
        }
        let retirement = retirement
            .bind_current_head()
            .map_err(|_| corrupt("World recovery head custody changed before selection"))?;
        prepared.observation = original.reference().clone();
        prepared.truth_version = original.truth_version();
        prepared.root = Some(root.clone());
        let displaced = state.replace_with(prepared);
        retirement.transfer_head(&root).replace_head();
        inputs
            .history
            .move_branch_head_basis(Some(&previous), Some(&root));
        drop(state);
        drop(_coordination);
        inputs.visibility.move_residency(
            self.identity.branch_id(),
            previous
                .axes()
                .map(|axes| crate::identity::data::VersionId(axes.storage_version)),
            root.axes()
                .map(|axes| crate::identity::data::VersionId(axes.storage_version)),
            None,
        );
        Ok(displaced)
    }
}
