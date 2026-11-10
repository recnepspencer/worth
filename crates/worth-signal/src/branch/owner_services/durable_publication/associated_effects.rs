//! Recovery discovers every admitted associated original from the protected owner cut.
use super::SignalDurablePublicationState;
use crate::durability::{SignalBranchAdvanceArtifact, SignalOwnerReadmissionDenial as D};
use std::{num::NonZeroU64, ops::ControlFlow};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::SignalDurableJournal;

impl<Dm, I, T> SignalDurablePublicationState<Dm, I, T>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn inspect_associated_advances<R>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        mut inspect: impl FnMut(
            &SignalBranchAdvanceArtifact,
            &crate::durability::SignalRecordedSourcePoint,
        ) -> ControlFlow<R>,
    ) -> Result<ControlFlow<R>, D> {
        match self.medium.publication_task_posture() {
            crate::durability::SignalPublicationTaskPosture::Terminal => {}
            posture @ (crate::durability::SignalPublicationTaskPosture::Busy
            | crate::durability::SignalPublicationTaskPosture::Unsettled) => {
                return Err(D::PublicationNotQuiescent(posture));
            }
        }
        let source = self
            .medium
            .capture_recovery_source(execution)
            .map_err(D::Medium)?;
        if source.owner() != self.binding().owner() {
            return Err(D::ForeignOwner);
        }
        let end = source
            .frontier(SignalDurableJournal::Advances, execution)
            .map_err(D::Medium)?;
        for position in 1..=end {
            let bytes = source
                .read(SignalDurableJournal::Advances, position, execution)
                .map_err(D::Medium)?;
            let artifact = SignalBranchAdvanceArtifact::readmit_bytes(execution, bytes.bytes())
                .map_err(D::Frame)?;
            let point = crate::durability::SignalRecordedSourcePoint::original(
                &artifact,
                NonZeroU64::new(position).expect("source position"),
            );
            let original = self
                .source
                .lock()
                .map_err(|_| D::PublicationStatePoisoned)?
                .advance_record(artifact.identity())?;
            if !original.matches_point(&point) {
                return Err(D::SourcePointMismatch(artifact.reference()));
            }
            if artifact.association().is_some() {
                if let ControlFlow::Break(value) = inspect(&artifact, &point) {
                    return Ok(ControlFlow::Break(value));
                }
            }
        }
        Ok(ControlFlow::Continue(()))
    }
}
