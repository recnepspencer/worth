//! Concrete obligation admission over the original Home journal transport.
use crate::query_journal::{
    ApplicationQueryJournal, QueryJournalDenial, QueryJournalPublicationOutcome as Physical,
};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_proof::{AdmittedHomeQueryObligations, HomeQueryObligationsOwner};
use worth_query_execution::facade::program_support::*;
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub(crate) struct ApplicationQueryObligations {
    journal: ApplicationQueryJournal,
    home: HomeIdentity,
    incarnation: HomeIncarnation,
    owner: HomeQueryObligationsOwner,
    authority: worth_proof::HomeQueryObligationsAuthority,
    admitted: OnceLock<AdmittedHomeQueryObligations>,
}
impl std::fmt::Debug for ApplicationQueryObligations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationQueryObligations")
            .field("home", &self.home)
            .field("incarnation", &self.incarnation)
            .finish_non_exhaustive()
    }
}
impl ApplicationQueryObligations {
    pub(crate) fn new(
        serving: &Arc<ServingPhysicalRuntime>,
        home: HomeIdentity,
        incarnation: HomeIncarnation,
    ) -> Self {
        let owner = HomeQueryObligationsOwner::fresh(home.bytes(), incarnation.fence());
        Self {
            journal: ApplicationQueryJournal::new(serving, "query.obligations"),
            home,
            incarnation,
            authority: owner.authority(),
            owner,
            admitted: OnceLock::new(),
        }
    }
    pub(crate) fn admit(
        &self,
        proof: AdmittedHomeQueryObligations,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryProgramSupportDenial> {
        self.owner.admit(&proof).map_err(incarnation)?;
        self.journal
            .admit_prefix(proof.position(), proof.payload_digest(), request)
            .map_err(denial)?;
        self.admitted
            .set(proof)
            .map_err(|_| QueryProgramSupportDenial::SupportUnavailable)
    }
    pub(crate) fn require_admitted(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryProgramSupportDenial> {
        self.journal
            .require_admitted(self.admitted.get().is_some(), request)
            .map_err(denial)
    }
}
impl QueryObligationsMedium for ApplicationQueryObligations {
    fn authority(&self) -> &worth_proof::HomeQueryObligationsAuthority {
        &self.authority
    }
    fn home(&self) -> HomeIdentity {
        self.home
    }
    fn incarnation(&self) -> HomeIncarnation {
        self.incarnation
    }
    fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
    ) -> Result<Option<Vec<u8>>, QueryProgramSupportDenial> {
        self.journal
            .read_frame(request, position, QUERY_PROGRAM_SUPPORT_FRAME_MAX_BYTES)
            .map_err(denial)
    }
    fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        frame: &QueryProgramSupportFrame,
    ) -> QueryObligationPublicationOutcome {
        if frame.home() != self.home {
            return QueryObligationPublicationOutcome::Refused(
                QueryProgramSupportDenial::ForeignHome,
            );
        }
        match self
            .journal
            .advance(request, frame.position(), frame.bytes())
        {
            Physical::Pending => QueryObligationPublicationOutcome::Pending,
            Physical::Committed { position, digest } => {
                QueryObligationPublicationOutcome::Committed(
                    self.owner
                        .acknowledge_query_obligation_commit(position, digest),
                )
            }
            Physical::ProvenNoEffect { position, digest } => {
                QueryObligationPublicationOutcome::ProvenNoEffect(
                    self.owner
                        .acknowledge_query_obligation_no_effect(position, digest),
                )
            }
            Physical::Refused(cause) => QueryObligationPublicationOutcome::Refused(denial(cause)),
            Physical::Unresolved => QueryObligationPublicationOutcome::Unresolved,
        }
    }
    fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryObligation,
    ) -> Result<(), QueryProgramSupportDenial> {
        proof
            .subject()
            .admit_to(&self.authority)
            .map_err(incarnation)?;
        self.journal
            .acknowledge_no_effect(proof.subject().position(), proof.subject().payload_digest())
            .map_err(denial)
    }
}
fn incarnation(cause: worth_proof::HomeIncarnationDenial) -> QueryProgramSupportDenial {
    match cause {
        worth_proof::HomeIncarnationDenial::ForeignHome => QueryProgramSupportDenial::ForeignHome,
        worth_proof::HomeIncarnationDenial::StaleIncarnation => {
            QueryProgramSupportDenial::StaleIncarnation
        }
    }
}
fn denial(cause: QueryJournalDenial) -> QueryProgramSupportDenial {
    match cause {
        QueryJournalDenial::PrefixMismatch => QueryProgramSupportDenial::PrefixMismatch,
        QueryJournalDenial::IdentityExhausted => QueryProgramSupportDenial::IdentityExhausted,
        QueryJournalDenial::OwnerUnavailable => QueryProgramSupportDenial::OwnerUnavailable,
        QueryJournalDenial::Unavailable | QueryJournalDenial::AdmissionRequired => {
            QueryProgramSupportDenial::SupportUnavailable
        }
        QueryJournalDenial::PublicationPending => QueryProgramSupportDenial::PublicationPending,
        QueryJournalDenial::PublicationUnresolved => {
            QueryProgramSupportDenial::PublicationUnresolved
        }
        QueryJournalDenial::Execution(cause) => QueryProgramSupportDenial::Execution(cause),
        QueryJournalDenial::Storage(cause) => QueryProgramSupportDenial::Storage(cause),
    }
}
