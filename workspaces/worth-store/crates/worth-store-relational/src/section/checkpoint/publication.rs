//! One framing and selection owner shared by ordinary and genuine Complete captures.
use super::selection_custody::SectionSelectionCustody;
use super::*;

pub(super) struct SectionPublication {
    initial_predecessor: Option<Option<[u8; 32]>>,
    pub(super) custody: SectionSelectionCustody,
    bytes: Vec<u8>,
    // Framed data and all original custody are destroyed before their bill.
    _memory: Option<ExecutionMemoryReservation>,
}

impl SectionPublication {
    pub(super) fn from_custody(
        custody: SectionSelectionCustody,
        initial_predecessor: Option<Option<[u8; 32]>>,
        bytes: Vec<u8>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            initial_predecessor,
            custody,
            bytes,
            _memory: memory,
        }
    }
    pub(super) fn prepare(
        source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
        checkpoint: &RelationalNativeCheckpoint,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalSectionCheckpointDenial> {
        Self::prepare_encoded(
            source,
            checkpoint,
            execution,
            None,
            26,
            checkpoint.bytes(),
            || {
                super::super::codec::encode(checkpoint)
                    .ok_or(RelationalSectionCheckpointDenial::MalformedSection)
            },
        )
    }
    pub(super) fn prepare_complete(
        source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
        complete: &worth_relational::facade::durability::CompleteRelationalNativeCheckpoint,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalSectionCheckpointDenial> {
        Self::prepare_encoded(
            source,
            complete.checkpoint(),
            execution,
            Some(complete.initial_predecessor()),
            super::super::complete_codec::HEADER,
            complete.source_artifact().as_bytes(),
            || super::super::complete_codec::encode(complete),
        )
    }
    fn prepare_encoded(
        source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
        checkpoint: &RelationalNativeCheckpoint,
        execution: &ExecutionRequest<'_, '_>,
        initial_predecessor: Option<Option<[u8; 32]>>,
        header: usize,
        payload: &[u8],
        encode: impl FnOnce() -> Result<Vec<u8>, RelationalSectionCheckpointDenial>,
    ) -> Result<Self, RelationalSectionCheckpointDenial> {
        use RelationalSectionCheckpointDenial as Denial;
        let size = payload.len().checked_add(header).ok_or(Denial::SizeLimit)?;
        if size > 1 << 30 {
            return Err(Denial::SizeLimit);
        }
        let memory = execution
            .reserve_retained_memory(size as u64)
            .map_err(Denial::Execution)?;
        let (copied, _) = execution
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(size as u64)?;
                } else {
                    context.poll()?;
                }
                Ok(encode())
            })
            .map_err(Denial::Execution)?;
        let bytes =
            copied.map_err(|stop| Denial::Execution(WorkCeilingDenial::Stopped(stop)))??;
        let custody = SectionSelectionCustody::prepare(source, checkpoint, execution)?;
        Ok(Self::from_custody(
            custody,
            initial_predecessor,
            bytes,
            memory,
        ))
    }

    pub(super) fn validate_publication(
        &self,
        checkpoint: &RelationalNativeCheckpoint,
    ) -> Result<(), RelationalSectionCheckpointDenial> {
        use worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial as Denial;
        if self.custody.selection.is_none() {
            return Err(RelationalSectionCheckpointDenial::PublicationAdmission(
                Denial::CaptureNotBound,
            ));
        }
        if self.initial_predecessor.is_some()
            && checkpoint
                .publication_subject()
                .and_then(|subject| subject.initial_inputs_coordinate())
                != super::super::complete_codec::coordinates(&self.bytes)
                    .map(|value| (value.revision, value.digest))
        {
            return Err(RelationalSectionCheckpointDenial::MalformedSection);
        }
        checkpoint
            .validate_publication_source(
                &(self.custody.source.clone()
                    as std::sync::Arc<
                        dyn worth_relational::facade::durability::RelationalDurableColdSource,
                    >),
            )
            .map_err(RelationalSectionCheckpointDenial::PublicationAdmission)
    }
    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Refuse a capture older than either frontier of the selected native section.
    pub(super) fn validate_previous(
        &self,
        previous: &[u8],
    ) -> Result<(), RelationalSectionCheckpointDenial> {
        use RelationalSectionCheckpointDenial as Denial;
        let complete_prior = super::super::complete_codec::coordinates(previous);
        let complete_next = super::super::complete_codec::coordinates(&self.bytes);
        let (prior, next) = match (complete_prior, complete_next) {
            (Some(prior), Some(next)) => {
                if next.revision < prior.revision
                    || next.revision == prior.revision && next.digest != prior.digest
                    || next.revision > prior.revision.saturating_add(1)
                    || next.revision > prior.revision
                        && self.initial_predecessor != Some(Some(prior.digest))
                {
                    return Err(Denial::StaleCapture);
                }
                ((prior.covered, prior.through), (next.covered, next.through))
            }
            (None, None) => (
                super::super::codec::coverage(previous).ok_or(Denial::MalformedSection)?,
                super::super::codec::coverage(&self.bytes).ok_or(Denial::MalformedSection)?,
            ),
            (Some(_), None) | (None, Some(_)) => return Err(Denial::MalformedSection),
        };
        if next.0 < prior.0 || next.1 < prior.1 {
            return Err(Denial::StaleCapture);
        }
        Ok(())
    }

    pub(super) fn prepare_members(
        &self,
        checkpoint: &RelationalNativeCheckpoint,
        previous: &[AnchoredRecordMemberLocator],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Vec<AnchoredRecordMember>,
            Option<ExecutionMemoryReservation>,
        ),
        RelationalSectionCheckpointDenial,
    > {
        members::prepare(checkpoint, &self.custody.retained, previous, execution)
    }
    pub(super) fn selection_protection(&self) -> RelationalCheckpointSelectionProtection {
        RelationalCheckpointSelectionProtection {
            _retained: self.custody.retained.clone(),
        }
    }
    /// Resolve only through this section's already attached Home source.
    /// Neither a membership argument nor a public fate can authorize selection.
    pub(super) fn acknowledge_selected(
        &self,
        checkpoint: &RelationalNativeCheckpoint,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), RelationalSectionCheckpointDenial> {
        use worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial as Ack;
        let refused = || RelationalSectionCheckpointDenial::Acknowledgement(Ack::SelectionNotKnown);
        let subject = checkpoint.publication_subject().ok_or_else(refused)?;
        let selection = self.custody.selection.as_ref().ok_or_else(refused)?;
        let expected = selection
            .selected_version(subject)
            .map_err(RelationalSectionCheckpointDenial::ReadProtection)?
            .ok_or_else(refused)?;
        context.poll().map_err(|stop| {
            RelationalSectionCheckpointDenial::Acknowledgement(Ack::Stopped(stop))
        })?;
        {
            let mut replacement = self.custody.replacement.try_lock().map_err(|_| {
                RelationalSectionCheckpointDenial::MemberSelection(
                    crate::RelationalStoreMemberSelectionDenial::Busy,
                )
            })?;
            replacement
                .advance(
                    &self.custody.source,
                    expected,
                    subject.required_members(),
                    execution,
                )
                .map_err(RelationalSectionCheckpointDenial::MemberSelection)?;
        }
        checkpoint
            .acknowledge_selected(execution, context)
            .map_err(RelationalSectionCheckpointDenial::Acknowledgement)
    }
}
