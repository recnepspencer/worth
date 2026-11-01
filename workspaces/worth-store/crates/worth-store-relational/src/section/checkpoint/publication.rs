//! One framing and selection owner shared by ordinary and genuine Complete captures.
use super::*;

pub(super) struct SectionPublication {
    source: std::sync::Arc<crate::cold_source::StoreColdSource>,
    retained: crate::cold_source::RetainedMemberPublication,
    bytes: Vec<u8>,
    pub(super) selection:
        Option<std::sync::Arc<dyn super::super::RelationalCheckpointSelectionSource>>,
    replacement: std::sync::Mutex<crate::cold_source::selected_replacement::SelectedReplacement>,
    // Framed data and all original custody are destroyed before their bill.
    _memory: Option<ExecutionMemoryReservation>,
}

impl SectionPublication {
    pub(super) fn prepare(
        source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
        checkpoint: &RelationalNativeCheckpoint,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalSectionCheckpointDenial> {
        use RelationalSectionCheckpointDenial as Denial;
        let size = checkpoint
            .bytes()
            .len()
            .checked_add(26)
            .ok_or(Denial::SizeLimit)?;
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
                Ok(super::super::codec::encode(checkpoint))
            })
            .map_err(Denial::Execution)?;
        let bytes = copied
            .map_err(|stop| Denial::Execution(WorkCeilingDenial::Stopped(stop)))?
            .ok_or(Denial::MalformedSection)?;
        let retained = source
            .protect_member_selection_for_request(
                checkpoint.members().iter().map(|member| member.locator()),
                execution,
            )
            .map_err(Denial::ReadProtection)?;
        Ok(Self {
            source: source.clone(),
            retained,
            bytes,
            selection: None,
            replacement: Default::default(),
            _memory: memory,
        })
    }

    pub(super) fn validate_publication(
        &self,
        checkpoint: &RelationalNativeCheckpoint,
    ) -> Result<(), RelationalSectionCheckpointDenial> {
        use worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial as Denial;
        if self.selection.is_none() {
            return Err(RelationalSectionCheckpointDenial::PublicationAdmission(
                Denial::CaptureNotBound,
            ));
        }
        checkpoint
            .validate_publication_source(
                &(self.source.clone()
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
        let prior = super::super::codec::coverage(previous).ok_or(Denial::MalformedSection)?;
        let next = super::super::codec::coverage(&self.bytes).ok_or(Denial::MalformedSection)?;
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
        members::prepare(checkpoint, &self.retained, previous, execution)
    }
    pub(super) fn selection_protection(&self) -> RelationalCheckpointSelectionProtection {
        RelationalCheckpointSelectionProtection {
            _retained: self.retained.clone(),
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
        let selection = self.selection.as_ref().ok_or_else(refused)?;
        let expected = selection
            .selected_version(subject)
            .map_err(RelationalSectionCheckpointDenial::ReadProtection)?
            .ok_or_else(refused)?;
        context.poll().map_err(|stop| {
            RelationalSectionCheckpointDenial::Acknowledgement(Ack::Stopped(stop))
        })?;
        {
            let mut replacement = self.replacement.try_lock().map_err(|_| {
                RelationalSectionCheckpointDenial::MemberSelection(
                    crate::RelationalStoreMemberSelectionDenial::Busy,
                )
            })?;
            replacement
                .advance(
                    &self.source,
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
