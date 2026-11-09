//! Inspect admitted originals without carrying the source-index lock across I/O.
use super::SignalDurableBranchSourcePort;
use crate::durability::{SignalBranchAdvanceArtifact, SignalOwnerReadmissionDenial as D};
use std::ops::ControlFlow;
use worth_execution::ExecutionRequest;

impl<Dm, I, T> SignalDurableBranchSourcePort<Dm, I, T>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    /// Visit only exact owner-admitted associated effects, including originals
    /// absent from the composer's journal. This grants no effect or recovery fate.
    pub fn inspect_associated_advances<R>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        inspect: impl FnMut(&SignalBranchAdvanceArtifact) -> ControlFlow<R>,
    ) -> Result<ControlFlow<R>, D> {
        let owner = self.owner().map_err(D::Source)?;
        let _admission = owner
            .admit()
            .map_err(super::super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(|cause| D::Source(super::SignalDurableSourceDenial::Admission(cause)))?;
        let publication = self.publication.upgrade().ok_or(D::Source(
            super::SignalDurableSourceDenial::SourceNotInstalled,
        ))?;
        publication.inspect_associated_advances(execution, inspect)
    }
}
