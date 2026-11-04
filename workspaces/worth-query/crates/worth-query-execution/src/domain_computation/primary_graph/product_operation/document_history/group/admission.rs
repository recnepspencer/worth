//! Every member's real admission precedes reservation and candidate creation.
use super::*;
use sha2::{Digest, Sha256};

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
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
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
        use super::super::journal::AttemptLookup;
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
        let mut key_hash = Sha256::new();
        key_hash.update(b"worth.query.document-history.group.key.v1");
        key_hash.update(H::IDENTITY.as_bytes());
        key_hash.update(key.to_le_bytes());
        let mut intent_hash = Sha256::new();
        intent_hash.update(b"worth.query.document-history.group.members.v1");
        for member in &members {
            intent_hash.update(member.meaning.canonical_key());
            let intent = member.meaning.intent_text();
            intent_hash.update((intent.len() as u64).to_le_bytes());
            intent_hash.update(intent.as_bytes());
        }
        let meaning = WorthQueryApplicationIdempotencyBinding::new(
            key_hash.finalize().into(),
            intent_hash.finalize().into(),
        );
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let mut selected = self.selection.lock();
        match selected.journal.lookup(meaning) {
            AttemptLookup::Completed(node) => {
                return Ok(WorthQueryDocumentHistoryGroupPreparation::AlreadyRecorded(
                    selected
                        .journal
                        .step(node)
                        .expect("completed group")
                        .clone(),
                ))
            }
            AttemptLookup::Drift => return Err(Denial::IntentDrift),
            AttemptLookup::Reserved => return Err(Denial::AttemptInFlight),
            AttemptLookup::Unseen => {}
        }
        if first.parent != selected.journal.current_step().id()
            || first.expected != selected.expected
        {
            return Err(Denial::HeadDiverged);
        }
        let bytes = members
            .len()
            .checked_mul(
                std::mem::size_of::<WorthQueryApplicationIdempotencyBinding>()
                    + std::mem::size_of::<CapturedMember>(),
            )
            .ok_or(Denial::GroupMemberLimitExceeded)?;
        let (allocation, _) =
            allocation::HistoryInsertionAllocation::reserve::<Self>(&execution, 0, bytes)
                .map_err(Denial::Recording)?;
        let completed = Vec::with_capacity(members.len());
        let label = &installed.definition().parts().operations[first.operation_index].label;
        let members = members.into_iter().map(|member| member.meaning).collect();
        let record = selected.journal.reserve_group(&execution, meaning, label)?;
        let expected = selected.expected.clone();
        drop(selected);
        Ok(WorthQueryDocumentHistoryGroupPreparation::Reserved(
            WorthQueryDocumentHistoryGroupReservation {
                reservation: WorthQueryDocumentHistoryReservation {
                    record,
                    expected,
                    meaning,
                },
                members,
                completed,
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
            completed,
            candidate,
            _allocation,
        } = group;
        drop(completed);
        drop(members);
        Ok(WorthQueryDocumentHistoryGroupAborting {
            reservation,
            runtime_authority: self.application().runtime.authority_identity().as_u64(),
            candidate: candidate.map(|(branch, _)| branch),
            _allocation,
        })
    }

    pub fn cancel_history_group(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        group: WorthQueryDocumentHistoryGroupAborting,
    ) -> Result<
        (),
        (
            WorthQueryDocumentHistoryGroupAborting,
            WorthQueryDocumentHistoryDenial,
        ),
    > {
        let mut selected = self.selection.lock();
        if !selected.journal.owns_group(&group.reservation.record) {
            return Err((group, WorthQueryDocumentHistoryDenial::ForeignRecording));
        }
        selected.journal.cancel(group.reservation.record);
        Ok(())
    }
}
