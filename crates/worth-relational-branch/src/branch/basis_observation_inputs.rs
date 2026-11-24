//! Observation binds one exact branch directory, schema fallback, and counter.
use super::basis::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use super::basis_admission::{
    descriptor_for_cell, issue_admitted_relational_branch_basis_with_retention,
};
use super::basis_denial::RelationalBranchBasisDenial;
use super::basis_identity_validation::require_local_branch_identity;
use super::basis_retention::map_retention_acquisition_denial;
use super::identity::RelationalBranchIdentity;
use super::root::RelationalBranchRoot;
use crate::runtime::operation_control::interruption_event::{
    RelationalInterruptionEvent, RelationalOperationInterruption,
};
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::RelationalSchemaRegistry;

#[derive(Clone, Copy)]
pub struct BranchObservationInputs<'a> {
    pub history: &'a HistorySubsystem,
    pub runtime_instance_id: u64,
    pub empty_schema: &'a RelationalSchemaRegistry,
    pub instrumentation: &'a RuntimeInstrumentation,
}

impl BranchObservationInputs<'_> {
    pub fn branch_identity(
        self,
        branch: &crate::history::data::BranchId,
    ) -> Result<RelationalBranchIdentity, super::identity::RelationalBranchIdentityDenial> {
        use super::identity::RelationalBranchIdentityDenial as Denial;
        let cell = self
            .history
            .branch_cell(branch)
            .ok_or_else(|| Denial::UnknownBranch(branch.clone()))?;
        if cell.identity().runtime_instance_id() != self.runtime_instance_id {
            return Err(Denial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_instance_id,
                actual_runtime_instance_id: cell.identity().runtime_instance_id(),
            });
        }
        Ok(cell.identity().clone())
    }

    pub fn capture_current_visibility_basis(
        self,
        branch: &crate::history::data::BranchId,
        version: crate::identity::data::VersionId,
    ) -> Result<
        Option<crate::history::retention::VisibilitySnapshotBasis>,
        RelationalBranchBasisDenial,
    > {
        let identity = self
            .branch_identity(branch)
            .map_err(super::basis_identity_validation::identity_denial)?;
        let (_, basis) = self.observe(&identity, || None)?;
        let observation = basis.observation();
        Ok((observation.version_id() == version).then(|| {
            crate::history::retention::VisibilitySnapshotBasis::from_observation(&observation)
        }))
    }

    pub fn observe(
        self,
        identity: &RelationalBranchIdentity,
        interrupted: impl Fn() -> Option<RelationalInterruptionEvent>,
    ) -> Result<
        (
            RelationalBranchBasisDescriptor,
            AdmittedRelationalBranchBasis,
        ),
        RelationalBranchBasisDenial,
    > {
        require_local_branch_identity(self.runtime_instance_id, self.history, identity)?;
        let (descriptor, root, publication_cell, retention_binding) = {
            let cell = self
                .history
                .branch_cell(identity.branch_id())
                .ok_or_else(|| {
                    RelationalBranchBasisDenial::UnknownBranch(identity.branch_id().clone())
                })?;
            let snapshot = cell.atomic_snapshot();
            match snapshot.lifecycle_posture() {
                crate::branch::lifecycle::RelationalBranchLifecyclePosture::Live => {}
                crate::branch::lifecycle::RelationalBranchLifecyclePosture::Archived => {
                    return Err(RelationalBranchBasisDenial::ArchivedBranch(
                        identity.branch_id().clone(),
                    ));
                }
                crate::branch::lifecycle::RelationalBranchLifecyclePosture::Deleting => {
                    return Err(RelationalBranchBasisDenial::DeletingBranch(
                        identity.branch_id().clone(),
                    ));
                }
            }
            let root = snapshot.root().unwrap_or_else(|| {
                RelationalBranchRoot::empty_with_schema(
                    self.empty_schema,
                    crate::schema::data::runtime_descriptor_semantics_policy()
                        .current_write_version(),
                )
            });
            let descriptor = descriptor_for_cell(&snapshot, &root)?;
            let retention_binding = cell
                .head_retention()
                .binding()
                .map_err(map_retention_acquisition_denial)?;
            (descriptor, root, cell.publication_cell(), retention_binding)
        };
        require_not_interrupted(interrupted(), &retention_binding)?;
        let basis = issue_admitted_relational_branch_basis_with_retention(
            descriptor.clone(),
            identity.clone(),
            root,
            publication_cell,
            &retention_binding,
        )?;
        require_not_interrupted(interrupted(), &retention_binding)?;
        let basis = self
            .history
            .branch_cell(identity.branch_id())
            .ok_or_else(|| {
                RelationalBranchBasisDenial::UnknownBranch(identity.branch_id().clone())
            })?
            .register_basis(basis)?;
        self.instrumentation.count_basis(|counters| {
            counters.basis_observations = counters.basis_observations.saturating_add(1);
        });
        Ok((descriptor, basis))
    }
}

fn require_not_interrupted(
    event: Option<RelationalInterruptionEvent>,
    retention_binding: &crate::history::retention::RelationalBranchRetentionBinding,
) -> Result<(), RelationalBranchBasisDenial> {
    match event {
        None => Ok(()),
        Some(event) if event.interruption() == RelationalOperationInterruption::Cancelled => {
            retention_binding.record_interruption(event);
            Err(RelationalBranchBasisDenial::Cancelled)
        }
        Some(event) => {
            retention_binding.record_interruption(event);
            Err(RelationalBranchBasisDenial::TimedOut)
        }
    }
}
