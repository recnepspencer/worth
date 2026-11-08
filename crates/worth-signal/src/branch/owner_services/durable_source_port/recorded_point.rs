//! Historical observation admission verifies both originals before issuing a lease.
use super::SignalDurableBranchSourcePort;
use crate::branch::owner_services::SignalDurableSourceDenial as S;
use crate::durability::{
    SignalAuthoredBranchChange, SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact,
    SignalReadmittedSourcePoint, SignalRecordedSourcePoint,
    SignalSourcePointReadmissionDenial as D,
};
use worth_execution::ExecutionRequest;

impl<Dm, I, T> SignalDurableBranchSourcePort<Dm, I, T>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    /// Admit the exact protected original through this owner's source index.
    /// The weak service cannot prolong or resurrect the durable owner.
    pub fn readmit_source_point(
        &self,
        point: &SignalRecordedSourcePoint,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalReadmittedSourcePoint, D> {
        self.owner().map_err(D::Source)?;
        let publication = self
            .publication
            .upgrade()
            .ok_or_else(|| D::Source(S::SourceNotInstalled))?;
        let (original, definition) = publication
            .read_source_point(point, execution)
            .map_err(D::Original)?;
        self.readmit_original_point(point, original, definition, execution)
    }

    pub(crate) fn readmit_original_point(
        &self,
        point: &SignalRecordedSourcePoint,
        original: SignalBranchAdvanceArtifact,
        definition: SignalDefinitionTransitionArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalReadmittedSourcePoint, D> {
        let restored = match original.change() {
            SignalAuthoredBranchChange::Created { .. }
            | SignalAuthoredBranchChange::Advanced
            | SignalAuthoredBranchChange::SnapshotCaptured => false,
            SignalAuthoredBranchChange::Restored => true,
            SignalAuthoredBranchChange::Retired => return Err(D::RetiredOriginal),
        };
        let owner = self.owner().map_err(D::Source)?;
        let admission = owner
            .admit()
            .map_err(super::super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(|denial| D::Source(S::Admission(denial)))?;
        let id = crate::state::SignalBranchId(point.descriptor().branch().ordinal());
        let cell = owner
            .lookup_cell(&admission, id)
            .map_err(|denial| super::super::owner::basis::map_basis_registry_denial(denial, id))
            .map_err(|denial| D::Source(S::Observation(denial)))?;
        let (bindings, observation) = cell
            .recorded_source_bindings(&admission, point.descriptor(), restored)
            .map_err(D::Source)?;
        // Replay the exact authored body through the installed program. Merely
        // possessing valid coordinates or a newer derived value is insufficient.
        let (graph, _) = bindings
            .readmit(execution, &definition.authored_image())
            .map_err(D::Image)?
            .into_parts();
        crate::logic::transaction::SignalRuntimeConfig::readmit_authored_image(
            execution,
            &definition.authored_runtime_image(),
            &graph,
            &self.vocabulary,
        )
        .map_err(D::Image)?;
        let basis = owner
            .admit_canonical_basis_with_retention(
                &admission,
                observation,
                id,
                cell.incarnation().get(),
                || owner.acquire_admitted_retention(&admission, id),
            )
            .map_err(|denial| D::Source(S::Retention(denial)))?;
        Ok(SignalReadmittedSourcePoint::from_owner(
            point.clone(),
            original,
            basis,
        ))
    }
}
