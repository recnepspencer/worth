//! Every member's real admission precedes reservation and candidate creation.
use super::super::recording::{
    group_meaning, Attempt, RefusedRecord, WorthQueryKeptRefusal, WorthQueryRecordingKind,
    WorthQueryRefusedCandidate,
};
use super::*;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn admit_history_group_member<M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
    ) -> Result<WorthQueryHistoryGroupMemberAdmission, WorthQueryDocumentHistoryDenial>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        use WorthQueryDocumentHistoryDenial as Denial;
        self.application()
            .assess_document_history_operation::<H, M>(
                phase,
                access,
                self,
                installed,
                operation.selected,
                operation.admitted,
            )?;
        if M::WORKFLOW_CONTROL
            || M::REQUIRES_WORKFLOW_AUTHORITY
            || M::workflow_execution_requirements().is_applicable()
            || operation
                .admitted
                .allowed_graph_contract()
                .external_effect()
                .is_declared()
        {
            return Err(Denial::UnsupportedOperation);
        }
        let selected = self.selection.lock();
        Ok(WorthQueryHistoryGroupMemberAdmission {
            operation_scope: HistoryOperationScope::from_admitted(operation.admitted),
            parent: selected.journal.current_step().id(),
            expected: selected.expected.clone(),
            model: self.model.clone(),
            meaning: operation.binding.bind_admission(operation.admitted),
            operation_index: installed
                .operation_descriptors()
                .iter()
                .position(|descriptor| *descriptor == M::descriptor())
                .expect("admitted membership"),
        })
    }

    pub fn reserve_history_group(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        key: u128,
        members: Vec<WorthQueryHistoryGroupMemberAdmission>,
    ) -> Result<WorthQueryDocumentHistoryGroupPreparation, WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        let resources = installed.group_resources();
        if members.is_empty() {
            return Err(Denial::EmptyGroup);
        }
        if members.len() > resources.maximum_members.get() {
            return Err(Denial::GroupMemberLimitExceeded);
        }
        if members.len() > resources.maximum_retained_results.get() {
            return Err(Denial::GroupResultLimitExceeded);
        }
        if installed.binding_identity() != &self.binding {
            return Err(Denial::InstalledBindingMismatch);
        }
        let first = &members[0];
        let mut keys = std::collections::BTreeSet::new();
        if members.iter().any(|member| {
            member.model != self.model
                || member.parent != first.parent
                || member.expected != first.expected
                || !keys.insert(member.meaning.canonical_key())
        }) {
            return Err(Denial::GroupMemberMismatch);
        }
        let meaning = group_meaning(
            H::IDENTITY,
            key,
            members.iter().map(|member| &member.meaning),
        );
        let execution = phase
            .checked_execution_request_for(self.application().inner.runtime.product_runtime_ref())
            .map_err(Denial::Stopped)?;
        let label = &installed.definition().parts().operations[first.operation_index].label;
        let mut selected = self.selection.lock();
        let attempt = selected.lookup_and_reserve(
            WorthQueryRecordingKind::Group,
            meaning,
            |selection| {
                first.parent == selection.journal.current_step().id()
                    && first.expected == selection.expected
            },
            |selection| {
                let bytes = members
                    .len()
                    .checked_mul(
                        std::mem::size_of::<WorthQueryApplicationIdempotencyBinding>()
                            + std::mem::size_of::<HistoryOperationScope>()
                            + std::mem::size_of::<CapturedMember>(),
                    )
                    .ok_or(Denial::GroupMemberLimitExceeded)?;
                let (allocation, _) =
                    allocation::HistoryInsertionAllocation::reserve::<Self>(&execution, 0, bytes)
                        .map_err(Denial::Recording)?;
                let record = selection
                    .journal
                    .reserve_group(&execution, meaning, label)?;
                Ok((record, selection.expected.clone(), allocation))
            },
        )?;
        drop(selected);
        let (record, expected, allocation) = match attempt {
            Attempt::Fresh(reserved) => reserved,
            Attempt::Recorded(step) => {
                return Ok(WorthQueryDocumentHistoryGroupPreparation::AlreadyRecorded(
                    step,
                ))
            }
        };
        let scopes = members
            .iter()
            .map(|member| member.operation_scope.clone())
            .collect();
        let completed = Vec::with_capacity(members.len());
        let members = members.into_iter().map(|member| member.meaning).collect();
        Ok(WorthQueryDocumentHistoryGroupPreparation::Reserved(
            WorthQueryDocumentHistoryGroupReservation {
                reservation: WorthQueryDocumentHistoryReservation {
                    operation_scope: None,
                    record,
                    expected,
                    meaning,
                },
                members,
                scopes,
                completed,
                outputs: None,
                candidate: None,
                _allocation: allocation,
            },
        ))
    }

    pub fn create_history_group_candidate(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        group: &mut WorthQueryDocumentHistoryGroupReservation,
    ) -> Result<
        WorthQueryHistoryGroupCandidateCreation<'runtime, S>,
        WorthQueryDocumentHistoryCandidateDenial,
    > {
        if group.candidate.is_some() {
            return Err(WorthQueryDocumentHistoryCandidateDenial::Admission(
                WorthQueryDocumentHistoryDenial::GroupMemberMismatch,
            ));
        }
        let candidate = self.create_history_candidate(phase, access, &group.reservation)?;
        group.candidate = Some((candidate.branch(), None));
        let observed = self
            .application()
            .on_branch(candidate.branch())
            .select(phase)
            .map(|selected| {
                WorthQueryProductBranchReadIdentity::from_observation(
                    selected.product().observation(),
                )
            });
        let denial = match observed {
            Ok(initial) => {
                group.candidate = Some((candidate.branch(), Some(initial)));
                None
            }
            Err(cause) => Some(WorthQueryDocumentHistoryDenial::CandidateSelection(cause)),
        };
        Ok(WorthQueryHistoryGroupCandidateCreation { candidate, denial })
    }
    pub fn readmit_history_group(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        group: &WorthQueryDocumentHistoryGroupReservation,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        self.validate_reservation(&group.reservation)
    }

    pub fn begin_history_group_abort(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        group: WorthQueryDocumentHistoryGroupReservation,
    ) -> Result<
        WorthQueryDocumentHistoryGroupAborting,
        (
            WorthQueryDocumentHistoryGroupReservation,
            WorthQueryDocumentHistoryDenial,
        ),
    > {
        if !self
            .selection
            .lock()
            .journal
            .owns_group(&group.reservation.record)
        {
            return Err((group, WorthQueryDocumentHistoryDenial::ForeignRecording));
        }
        let WorthQueryDocumentHistoryGroupReservation {
            reservation,
            members,
            scopes,
            completed,
            outputs,
            candidate,
            _allocation,
        } = group;
        drop(outputs);
        drop(completed);
        drop(members);
        drop(scopes);
        Ok(WorthQueryDocumentHistoryGroupAborting {
            reservation,
            runtime_authority: self
                .application()
                .inner
                .runtime
                .authority_identity()
                .as_u64(),
            candidate: candidate.map(|(branch, _)| branch),
            _allocation,
        })
    }

    /// Release an aborted group's record after its candidate's close attempt.
    /// A candidate that did not close comes back with the group, so the caller
    /// retries the close.
    pub fn refuse_history_group(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        group: WorthQueryDocumentHistoryGroupAborting,
        candidate: Option<WorthQueryRefusedCandidate<'runtime, S>>,
    ) -> Result<(), WorthQueryKeptRefusal<'runtime, S, WorthQueryDocumentHistoryGroupAborting>>
    {
        self.refuse(WorthQueryRecordingKind::Group, group, candidate)
    }
}

impl RefusedRecord for WorthQueryDocumentHistoryGroupAborting {
    fn check(
        &self,
        selection: &super::super::selection::HistorySelection,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        if selection.journal.owns_group(&self.reservation.record) {
            Ok(())
        } else {
            Err(WorthQueryDocumentHistoryDenial::ForeignRecording)
        }
    }
    fn into_record(self) -> super::super::journal::ReservedHistoryRecord {
        self.reservation.record
    }
}
