//! Clone original membership under the index lock, then perform source I/O without it.
use super::SignalDurablePublicationState;
use crate::durability::{
    SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor,
    SignalDurableHomeBinding, SignalOwnerReadmissionDenial as Dn,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::SignalBranchAdvanceIdentity;

impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn binding(&self) -> &SignalDurableHomeBinding {
        self.medium.binding()
    }
    pub(crate) fn read_definition_source(
        &self,
        descriptor: &SignalDurableBranchDescriptor,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDefinitionTransitionArtifact, Dn> {
        if descriptor.owner() != self.binding().owner() {
            return Err(Dn::ForeignOwner);
        }
        let original = self
            .source
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
            .definition_record(descriptor)?;
        original.read(self.medium.as_ref(), execution)
    }
    pub(crate) fn read_original_advance(
        &self,
        identity: SignalBranchAdvanceIdentity,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalBranchAdvanceArtifact, Dn> {
        if identity.owner() != self.binding().owner() {
            return Err(Dn::ForeignOwner);
        }
        let original = self
            .source
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
            .advance_record(identity)?;
        original.read(self.medium.as_ref(), execution)
    }
    pub(crate) fn read_source_point(
        &self,
        point: &crate::durability::SignalRecordedSourcePoint,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            SignalBranchAdvanceArtifact,
            SignalDefinitionTransitionArtifact,
        ),
        Dn,
    > {
        if point.descriptor().owner() != self.binding().owner() {
            return Err(Dn::ForeignOwner);
        }
        let (advance, definition) = {
            let source = self
                .source
                .lock()
                .map_err(|_| Dn::PublicationStatePoisoned)?;
            let advance = source.advance_record(point.reference().identity())?;
            if !advance.matches_point(point) {
                return Err(Dn::SourcePointMismatch(point.reference()));
            }
            (advance, source.definition_record(point.descriptor())?)
        };
        let artifact = advance.read(self.medium.as_ref(), execution)?;
        if artifact.reference() != point.reference() || artifact.descriptor() != point.descriptor()
        {
            return Err(Dn::SourcePointMismatch(point.reference()));
        }
        let definition = definition.read(self.medium.as_ref(), execution)?;
        Ok((artifact, definition))
    }
}
