//! The Home's one `query.dispatch` journal operation: opaque Query frames over
//! this admitted Home's original journal transport. Store never decodes a frame.
use crate::query_journal::{ApplicationQueryJournal, QueryJournalDenial};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_proof::{AdmittedHomeQueryDispatch, HomeQueryDispatchOwner};
use worth_store::physical_runtime::ServingPhysicalRuntime;
mod operation;
pub use operation::ApplicationHomeDispatchJournalOperation;

#[derive(Debug)]
pub enum ApplicationHomeDispatchJournalDenial {
    ForeignHome,
    StaleIncarnation,
    PrefixMismatch,
    IdentityExhausted,
    /// The Home that issued this operation has released its serving.
    OwnerUnavailable,
    /// A recorded prefix exists that its Query owner has not admitted.
    AdmissionUnavailable,
    PublicationPending,
    PublicationUnresolved,
    DeadlineUnavailable,
    Storage(String),
    Execution(worth_execution::WorkCeilingDenial),
}

#[derive(Debug)]
pub enum ApplicationHomeDispatchPublication {
    Pending,
    Committed(worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryDispatch>),
    ProvenNoEffect(worth_proof::NoEffectHomeQueryDispatch),
    Refused(ApplicationHomeDispatchJournalDenial),
    Unresolved,
}

pub(crate) struct ApplicationHomeDispatchJournal {
    journal: ApplicationQueryJournal,
    home: HomeIdentity,
    incarnation: HomeIncarnation,
    owner: HomeQueryDispatchOwner,
    authority: worth_proof::HomeQueryDispatchAuthority,
    admitted: OnceLock<AdmittedHomeQueryDispatch>,
}
impl std::fmt::Debug for ApplicationHomeDispatchJournal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationHomeDispatchJournal")
            .field("home", &self.home)
            .field("incarnation", &self.incarnation)
            .finish_non_exhaustive()
    }
}
impl ApplicationHomeDispatchJournal {
    pub(crate) fn new(
        serving: &Arc<ServingPhysicalRuntime>,
        home: HomeIdentity,
        incarnation: HomeIncarnation,
    ) -> Self {
        let owner = HomeQueryDispatchOwner::fresh(home.bytes(), incarnation.fence());
        Self {
            journal: ApplicationQueryJournal::new(serving, "query.dispatch"),
            home,
            incarnation,
            authority: owner.authority(),
            owner,
            admitted: OnceLock::new(),
        }
    }
    /// The Query owner's decoded prefix must be this journal's actual head.
    pub(crate) fn admit(
        &self,
        proof: AdmittedHomeQueryDispatch,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeDispatchJournalDenial> {
        self.owner.admit(&proof).map_err(incarnation_denial)?;
        self.journal
            .admit_prefix(proof.position(), proof.payload_digest(), request)
            .map_err(denial)?;
        self.admitted
            .set(proof)
            .map_err(|_| ApplicationHomeDispatchJournalDenial::AdmissionUnavailable)
    }
    /// Service requires an admitted prefix and no publication of unknown fate.
    pub(crate) fn require_admitted(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeDispatchJournalDenial> {
        self.journal
            .require_admitted(self.admitted.get().is_some(), request)
            .map_err(denial)
    }
}

fn incarnation_denial(
    cause: worth_proof::HomeIncarnationDenial,
) -> ApplicationHomeDispatchJournalDenial {
    match cause {
        worth_proof::HomeIncarnationDenial::StaleIncarnation => {
            ApplicationHomeDispatchJournalDenial::StaleIncarnation
        }
        worth_proof::HomeIncarnationDenial::ForeignHome => {
            ApplicationHomeDispatchJournalDenial::ForeignHome
        }
    }
}
fn denial(cause: QueryJournalDenial) -> ApplicationHomeDispatchJournalDenial {
    use ApplicationHomeDispatchJournalDenial as Denial;
    match cause {
        QueryJournalDenial::PrefixMismatch => Denial::PrefixMismatch,
        QueryJournalDenial::IdentityExhausted => Denial::IdentityExhausted,
        QueryJournalDenial::OwnerUnavailable => Denial::OwnerUnavailable,
        QueryJournalDenial::Unavailable => Denial::DeadlineUnavailable,
        QueryJournalDenial::AdmissionRequired => Denial::AdmissionUnavailable,
        QueryJournalDenial::PublicationPending => Denial::PublicationPending,
        QueryJournalDenial::PublicationUnresolved => Denial::PublicationUnresolved,
        QueryJournalDenial::Execution(cause) => Denial::Execution(cause),
        QueryJournalDenial::Storage(detail) => Denial::Storage(detail),
    }
}
