//! Capture and placement preparation, retaining the actual selection fence.
use super::RelationalStoreSection;
use std::collections::BTreeMap;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::durability::{
    RelationalCheckpointArtifactPolicy, RelationalCheckpointMember, RelationalNativeCheckpoint,
    RelationalNativeCheckpointCaptureDenial,
};
use worth_store::physical_runtime::{AnchoredRecordMember, AnchoredRecordMemberLocator};

#[derive(Debug)]
pub enum RelationalSectionCheckpointDenial {
    Capture(RelationalNativeCheckpointCaptureDenial),
    Execution(WorkCeilingDenial),
    MalformedSection,
    StaleCapture,
    MissingCarriedMember,
    SizeLimit,
    Acknowledgement(
        worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial,
    ),
    PublicationAdmission(
        worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial,
    ),
}

/// A native capture and the read-custody fence that must outlive publication.
/// No Clone/Copy: only selection may acknowledge this capture.
///
/// ```compile_fail,E0599
/// use worth_store_relational::PreparedRelationalSection;
/// use worth_store::physical_runtime::AnchoredRecordMembership;
/// fn pair_descriptions(section: PreparedRelationalSection, receipt: &AnchoredRecordMembership) {
///     section.select(receipt);
/// }
/// ```
///
/// ```compile_fail,E0599
/// use worth_relational::facade::durability::RelationalNativeCheckpoint;
/// fn public_fate_is_not_selection(native: RelationalNativeCheckpoint) {
///     native.acknowledge_publication();
/// }
/// ```
///
/// ```compile_fail,E0432
/// use worth_relational::facade::durability::ActualCheckpointSelection;
/// ```
///
/// ```
/// use worth_store_relational::PreparedRelationalSection;
/// fn observe(section: &PreparedRelationalSection) {
///     let _description = section.publication_subject();
/// }
/// ```
pub struct PreparedRelationalSection {
    checkpoint: RelationalNativeCheckpoint,
    source: std::sync::Arc<crate::cold_source::StoreColdSource>,
    retained: crate::cold_source::RetainedMemberPublication,
    bytes: Vec<u8>,
    _memory: Option<ExecutionMemoryReservation>,
    selection: Option<std::sync::Arc<dyn super::RelationalCheckpointSelectionSource>>,
}

/// Lifetime-only transport to an accepted physical step. No publication or
/// acknowledgement authority, source map, or Native capture is exposed.
#[derive(Clone)]
pub struct RelationalCheckpointSelectionProtection {
    _retained: crate::cold_source::RetainedMemberPublication,
}

impl RelationalStoreSection {
    pub fn prepare_checkpoint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
        let mut prepared = prepare(&self.runtime, &self.cold_source, execution)?;
        prepared.selection = Some(self.checkpoint_selection.clone());
        Ok(prepared)
    }
}

pub(super) fn prepare(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
    use RelationalSectionCheckpointDenial as Denial;
    let members = source.member_locators();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &members,
                source: Some(source.clone()),
            },
        )
        .map_err(Denial::Capture)?;
    prepare_captured(source, checkpoint, execution)
}

pub(crate) fn prepare_captured(
    source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    checkpoint: RelationalNativeCheckpoint,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
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
            Ok(super::codec::encode(&checkpoint))
        })
        .map_err(Denial::Execution)?;
    let bytes = copied
        .map_err(|stop| Denial::Execution(WorkCeilingDenial::Stopped(stop)))?
        .ok_or(Denial::MalformedSection)?;
    let retained = source
        .protect_member_selection(checkpoint.members().iter().map(|member| member.locator()))
        .map_err(|_| Denial::MissingCarriedMember)?;
    Ok(PreparedRelationalSection {
        checkpoint,
        source: source.clone(),
        retained,
        bytes,
        _memory: memory,
        selection: None,
    })
}

impl PreparedRelationalSection {
    pub fn validate_publication(&self) -> Result<(), RelationalSectionCheckpointDenial> {
        use worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial as Denial;
        if self.selection.is_none() {
            return Err(RelationalSectionCheckpointDenial::PublicationAdmission(
                Denial::CaptureNotBound,
            ));
        }
        self.checkpoint
            .validate_publication_source(
                &(self.source.clone()
                    as std::sync::Arc<
                        dyn worth_relational::facade::durability::RelationalDurableColdSource,
                    >),
            )
            .map_err(RelationalSectionCheckpointDenial::PublicationAdmission)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn covered(&self) -> worth_relational::facade::publication::PatchStreamPosition {
        self.checkpoint
            .covered_position()
            .expect("encoded section has canonical coverage")
    }

    pub fn covered_transitions(
        &self,
    ) -> worth_relational::facade::branch::RelationalBranchTransitionSequence {
        self.checkpoint
            .covered_branch_transitions()
            .expect("encoded section has transition coverage")
    }

    /// Refuse a capture older than either frontier of the selected native section.
    pub fn validate_previous(
        &self,
        previous: &[u8],
    ) -> Result<(), RelationalSectionCheckpointDenial> {
        use RelationalSectionCheckpointDenial as Denial;
        let prior = super::codec::coverage(previous).ok_or(Denial::MalformedSection)?;
        let next = super::codec::coverage(&self.bytes).ok_or(Denial::MalformedSection)?;
        if next.0 < prior.0 || next.1 < prior.1 {
            return Err(Denial::StaleCapture);
        }
        Ok(())
    }

    /// Prepare physical membership, including every still-live read obligation.
    /// The caller retains the returned payload reservation through publication.
    pub fn prepare_members(
        &self,
        previous: &[AnchoredRecordMemberLocator],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Vec<AnchoredRecordMember>,
            Option<ExecutionMemoryReservation>,
        ),
        RelationalSectionCheckpointDenial,
    > {
        use RelationalSectionCheckpointDenial as Denial;
        let old = previous
            .iter()
            .map(|member| ((member.sha256(), member.length()), *member))
            .collect::<BTreeMap<_, _>>();
        let size = self
            .checkpoint
            .members()
            .iter()
            .try_fold(0_u64, |size, member| match member {
                RelationalCheckpointMember::New { locator, bytes } => {
                    if old.contains_key(&(locator.sha256, locator.length)) {
                        Some(size)
                    } else {
                        size.checked_add(bytes.len() as u64)
                    }
                }
                RelationalCheckpointMember::Carried { .. } => Some(size),
            })
            .ok_or(Denial::SizeLimit)?;
        let memory = execution
            .reserve_retained_memory(size)
            .map_err(Denial::Execution)?;
        let (prepared, _) = execution
            .run_native_with_checkpoint(|context| {
                let mut members = BTreeMap::new();
                for member in self.checkpoint.members() {
                    if context.has_accounting() {
                        context.checkpoint(1)?;
                    } else {
                        context.poll()?;
                    }
                    let key = member.locator();
                    let placed = if let Some(locator) = old.get(&(key.sha256, key.length)) {
                        AnchoredRecordMember::Carried { locator: *locator }
                    } else {
                        match member {
                            RelationalCheckpointMember::New { locator, bytes } => {
                                if context.has_accounting() {
                                    context.checkpoint(bytes.len() as u64)?;
                                }
                                AnchoredRecordMember::New {
                                    key: locator.sha256,
                                    bytes: bytes.to_vec(),
                                }
                            }
                            RelationalCheckpointMember::Carried { .. } => {
                                return Ok(Err(Denial::MissingCarriedMember))
                            }
                        }
                    };
                    members.insert(key, placed);
                }
                for key in self.retained.member_locators() {
                    if context.has_accounting() {
                        context.checkpoint(1)?;
                    } else {
                        context.poll()?;
                    }
                    if let std::collections::btree_map::Entry::Vacant(entry) = members.entry(key) {
                        let Some(locator) = old.get(&(key.sha256, key.length)) else {
                            return Ok(Err(Denial::MissingCarriedMember));
                        };
                        entry.insert(AnchoredRecordMember::Carried { locator: *locator });
                    }
                }
                Ok(Ok(members.into_values().collect()))
            })
            .map_err(Denial::Execution)?;
        let members =
            prepared.map_err(|stop| Denial::Execution(WorkCeilingDenial::Stopped(stop)))??;
        Ok((members, memory))
    }

    pub fn publication_subject(
        &self,
    ) -> Option<&worth_relational::facade::durability::OpaqueCapturedCheckpointSubject> {
        self.checkpoint.publication_subject()
    }

    pub fn selection_protection(&self) -> RelationalCheckpointSelectionProtection {
        RelationalCheckpointSelectionProtection {
            _retained: self.retained.clone(),
        }
    }

    pub(super) fn with_selection(
        mut self,
        selection: std::sync::Arc<dyn super::RelationalCheckpointSelectionSource>,
    ) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Resolve only through this section's already attached Home source.
    /// Neither a membership argument nor a public fate can authorize selection.
    pub fn acknowledge_selected(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), RelationalSectionCheckpointDenial> {
        use worth_relational::facade::durability::{
            CheckpointSelectionObservation, RelationalCheckpointAcknowledgementDenial as Ack,
        };
        let refused = || RelationalSectionCheckpointDenial::Acknowledgement(Ack::SelectionNotKnown);
        let subject = self.publication_subject().ok_or_else(refused)?;
        let selection = self.selection.as_ref().ok_or_else(refused)?;
        if !matches!(
            super::selection::observe(selection.as_ref(), subject),
            CheckpointSelectionObservation::Selected(_)
        ) {
            return Err(refused());
        }
        let membership = selection.selected_membership(subject).ok_or_else(refused)?;
        context.poll().map_err(|stop| {
            RelationalSectionCheckpointDenial::Acknowledgement(Ack::Stopped(stop))
        })?;
        self.source
            .select_members(membership.version().generation(), membership.members());
        self.checkpoint
            .acknowledge_selected(context)
            .map_err(RelationalSectionCheckpointDenial::Acknowledgement)
    }
}
