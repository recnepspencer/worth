//! Transfer an existing local grant at the actual acknowledged-open seam.
use super::*;
use journal::*;
use worth_foundational::facade::{
    CanonicalDigestId, ProgramSupportObligationIdentity, ProgramSupportObligationKind,
};
use worth_runtime_world::facade::WorldOpenedPublicationAttempt;

pub(super) struct PendingInterpretationTransfer {
    revision: ApplicationProgramRevision,
    membership: Arc<Mutex<InterpretationMembership>>,
    identity: ProgramSupportObligationIdentity,
}
impl PendingInterpretationTransfer {
    pub(super) fn validate(
        &self,
        state: &QueryObligationState,
        frame: &QueryObligationFrame,
    ) -> Result<(), QueryObligationDenial> {
        let QueryObligationRecord::ProgramSupport(QueryProgramSupportRecord::Acquire(row)) =
            frame.record()
        else {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        };
        if row.identity != self.identity
            || row.kind != ProgramSupportObligationKind::OriginalInterpretation
            || row.source.bytes() != self.revision.as_bytes()
            || row.target != row.source
        {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        let entry = state
            .entries
            .get(&self.revision)
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        self.membership
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .validate_pending(entry.retained_interpretations)
    }
    pub(super) fn committed(self, state: &mut QueryObligationState) {
        self.membership
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .committed(
                &mut state
                    .entries
                    .get_mut(&self.revision)
                    .expect("validated transfer")
                    .retained_interpretations,
                self.identity,
            );
    }
    pub(super) fn no_effect(self, state: &mut QueryObligationState) {
        self.membership
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .no_effect(
                &mut state
                    .entries
                    .get_mut(&self.revision)
                    .expect("validated transfer")
                    .retained_interpretations,
            );
    }
}
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn transfer_original_interpretation(
        &self,
        token: &WorthQueryProgramSupportInterpretation,
        opened: &WorldOpenedPublicationAttempt,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        if !Arc::ptr_eq(&self.state, &token.state)
            || opened.program_revision().bytes() != token.revision.as_bytes()
        {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let original = opened.association().attempt();
        if original.owner().home() != medium.home() {
            return Err(QueryObligationDenial::ForeignHome);
        }
        opened
            .attempt_identity()
            .owner_identity()
            .home_incarnation_fence()
            .ok_or(QueryObligationDenial::ForeignHome)?
            .admit(medium.incarnation().fence())
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => {
                    QueryObligationDenial::ForeignHome
                }
                worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                    QueryObligationDenial::StaleIncarnation
                }
            })?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        {
            let mut state = self.state();
            let mut membership = token
                .membership
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match &*membership {
                InterpretationMembership::Durable(identity) => {
                    let row = state
                        .journal
                        .as_ref()
                        .and_then(|journal| {
                            journal
                                .ledger
                                .obligations()
                                .find(|row| row.identity == *identity)
                        })
                        .ok_or(QueryObligationDenial::UnknownObligation)?;
                    return if row.original == original && row.source == opened.program_revision() {
                        Ok(())
                    } else {
                        Err(QueryObligationDenial::ObligationSubjectMismatch)
                    };
                }
                InterpretationMembership::Local => {}
                InterpretationMembership::Pending
                | InterpretationMembership::AbandonedPending
                | InterpretationMembership::Disposed => {
                    return Err(QueryObligationDenial::ObligationSubjectMismatch)
                }
            }
            let journal = state
                .journal
                .as_mut()
                .ok_or(QueryObligationDenial::SupportUnavailable)?;
            let position = journal
                .ledger
                .position()
                .checked_add(1)
                .and_then(std::num::NonZeroU64::new)
                .ok_or(QueryObligationDenial::IdentityExhausted)?;
            let identity = ProgramSupportObligationIdentity::from_recorded(medium.home(), position);
            let revision = CanonicalDigestId::new(*token.revision.as_bytes());
            journal.prepare_pending(QueryProgramSupportRecord::Acquire(
                QueryProgramSupportObligation {
                    identity,
                    kind: ProgramSupportObligationKind::OriginalInterpretation,
                    source: revision,
                    target: revision,
                    original,
                },
            ))?;
            membership.begin()?;
            journal.interpretation_transfer = Some(PendingInterpretationTransfer {
                revision: token.revision,
                membership: Arc::clone(&token.membership),
                identity,
            });
        }
        driver.finish_pending(medium.as_ref(), request)
    }
}
