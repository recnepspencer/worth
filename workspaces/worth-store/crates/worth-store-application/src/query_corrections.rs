//! Concrete correction-journal admission over the original Home journal transport.
use crate::query_journal::{
    ApplicationQueryJournal, QueryJournalDenial, QueryJournalPublicationOutcome as Physical,
};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_proof::{AdmittedHomeQueryCorrections, HomeQueryCorrectionsOwner};
use worth_query_execution::facade::correction_material::*;
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub(crate) struct ApplicationQueryCorrections {
    journal: ApplicationQueryJournal,
    home: HomeIdentity,
    incarnation: HomeIncarnation,
    owner: HomeQueryCorrectionsOwner,
    authority: worth_proof::HomeQueryCorrectionsAuthority,
    admitted: OnceLock<AdmittedHomeQueryCorrections>,
}
impl std::fmt::Debug for ApplicationQueryCorrections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationQueryCorrections")
            .field("home", &self.home)
            .field("incarnation", &self.incarnation)
            .finish_non_exhaustive()
    }
}
impl ApplicationQueryCorrections {
    pub(crate) fn new(
        serving: &Arc<ServingPhysicalRuntime>,
        home: HomeIdentity,
        incarnation: HomeIncarnation,
    ) -> Self {
        let owner = HomeQueryCorrectionsOwner::fresh(home.bytes(), incarnation.fence());
        Self {
            journal: ApplicationQueryJournal::new(serving, "query.corrections"),
            home,
            incarnation,
            authority: owner.authority(),
            owner,
            admitted: OnceLock::new(),
        }
    }
    pub(crate) fn require_admitted(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryCorrectionMaterialDenial> {
        self.journal
            .require_admitted(self.admitted.get().is_some(), request)
            .map_err(denial)
    }
    pub(crate) fn admit(
        &self,
        proof: AdmittedHomeQueryCorrections,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryCorrectionMaterialDenial> {
        self.owner.admit(&proof).map_err(incarnation)?;
        self.journal
            .admit_prefix(proof.position(), proof.payload_digest(), request)
            .map_err(denial)?;
        self.admitted
            .set(proof)
            .map_err(|_| QueryCorrectionMaterialDenial::AdmissionAlreadyRecorded)
    }
}
impl QueryCorrectionsMedium for ApplicationQueryCorrections {
    fn authority(&self) -> &worth_proof::HomeQueryCorrectionsAuthority {
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
        maximum_bytes: u64,
    ) -> Result<Option<Vec<u8>>, QueryCorrectionMaterialDenial> {
        self.journal
            .read_frame(request, position, maximum_bytes)
            .map_err(denial)
    }
    fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        frame: &QueryCorrectionMaterialFrame,
    ) -> QueryCorrectionMaterialPublicationOutcome {
        if frame.home() != self.home {
            return QueryCorrectionMaterialPublicationOutcome::Refused(
                QueryCorrectionMaterialDenial::ForeignHome,
            );
        }
        match self
            .journal
            .advance(request, frame.position(), frame.bytes())
        {
            Physical::Pending => QueryCorrectionMaterialPublicationOutcome::Pending,
            Physical::Committed { position, digest } => {
                QueryCorrectionMaterialPublicationOutcome::Committed(
                    self.owner
                        .acknowledge_query_correction_commit(position, digest),
                )
            }
            Physical::ProvenNoEffect { position, digest } => {
                QueryCorrectionMaterialPublicationOutcome::ProvenNoEffect(
                    self.owner
                        .acknowledge_query_correction_no_effect(position, digest),
                )
            }
            Physical::Refused(cause) => {
                QueryCorrectionMaterialPublicationOutcome::Refused(denial(cause))
            }
            Physical::Unresolved => QueryCorrectionMaterialPublicationOutcome::Unresolved,
        }
    }
    fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryCorrection,
    ) -> Result<(), QueryCorrectionMaterialDenial> {
        proof
            .subject()
            .admit_to(&self.authority)
            .map_err(incarnation)?;
        self.journal
            .acknowledge_no_effect(proof.subject().position(), proof.subject().payload_digest())
            .map_err(denial)
    }
}
fn incarnation(cause: worth_proof::HomeIncarnationDenial) -> QueryCorrectionMaterialDenial {
    match cause {
        worth_proof::HomeIncarnationDenial::ForeignHome => {
            QueryCorrectionMaterialDenial::ForeignHome
        }
        worth_proof::HomeIncarnationDenial::StaleIncarnation => {
            QueryCorrectionMaterialDenial::StaleIncarnation
        }
    }
}
fn denial(cause: QueryJournalDenial) -> QueryCorrectionMaterialDenial {
    match cause {
        QueryJournalDenial::PrefixMismatch => QueryCorrectionMaterialDenial::PrefixMismatch,
        QueryJournalDenial::IdentityExhausted => QueryCorrectionMaterialDenial::IdentityExhausted,
        QueryJournalDenial::OwnerUnavailable => QueryCorrectionMaterialDenial::OwnerUnavailable,
        QueryJournalDenial::Unavailable => QueryCorrectionMaterialDenial::Unavailable,
        QueryJournalDenial::AdmissionRequired => QueryCorrectionMaterialDenial::AdmissionRequired,
        QueryJournalDenial::PublicationPending => QueryCorrectionMaterialDenial::PublicationPending,
        QueryJournalDenial::PublicationUnresolved => {
            QueryCorrectionMaterialDenial::PublicationUnresolved
        }
        QueryJournalDenial::Execution(cause) => QueryCorrectionMaterialDenial::Execution(cause),
        QueryJournalDenial::Storage(cause) => QueryCorrectionMaterialDenial::Storage(cause),
    }
}
