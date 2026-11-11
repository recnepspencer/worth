//! Host implements Query's dispatch medium over the same Home's journal operation.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_query_execution::facade::application_home_integration::{
    QueryDispatchFrame, QueryDispatchJournalDenial, QueryDispatchMedium,
    QueryDispatchPublicationOutcome, QUERY_DISPATCH_FRAME_MAX_BYTES,
};
use worth_store_application::{
    ApplicationHomeDispatchJournalDenial, ApplicationHomeDispatchJournalOperation,
    ApplicationHomeDispatchPublication,
};

/// Translates vocabulary only. Custody, physical publication and the journal
/// name stay in the Home-issued operation; frame meaning stays in Query.
#[derive(Debug)]
pub(super) struct HomeQueryDispatchMedium {
    operation: ApplicationHomeDispatchJournalOperation,
}
impl HomeQueryDispatchMedium {
    pub(super) fn over(
        operation: ApplicationHomeDispatchJournalOperation,
    ) -> Arc<dyn QueryDispatchMedium> {
        Arc::new(Self { operation })
    }
}
impl QueryDispatchMedium for HomeQueryDispatchMedium {
    fn authority(&self) -> &worth_proof::HomeQueryDispatchAuthority {
        self.operation.authority()
    }
    fn home(&self) -> HomeIdentity {
        self.operation.home()
    }
    fn incarnation(&self) -> HomeIncarnation {
        self.operation.incarnation()
    }
    fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
    ) -> Result<Option<Vec<u8>>, QueryDispatchJournalDenial> {
        self.operation
            .read_frame(request, position, QUERY_DISPATCH_FRAME_MAX_BYTES)
            .map_err(query_denial)
    }
    fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        frame: &QueryDispatchFrame,
    ) -> QueryDispatchPublicationOutcome {
        if frame.home() != self.operation.home() {
            return QueryDispatchPublicationOutcome::Refused(
                QueryDispatchJournalDenial::ForeignHome,
            );
        }
        match self
            .operation
            .advance_publication(request, frame.position(), frame.bytes())
        {
            ApplicationHomeDispatchPublication::Pending => QueryDispatchPublicationOutcome::Pending,
            ApplicationHomeDispatchPublication::Committed(proof) => {
                QueryDispatchPublicationOutcome::Committed(proof)
            }
            ApplicationHomeDispatchPublication::ProvenNoEffect(proof) => {
                QueryDispatchPublicationOutcome::ProvenNoEffect(proof)
            }
            ApplicationHomeDispatchPublication::Refused(cause) => {
                QueryDispatchPublicationOutcome::Refused(query_denial(cause))
            }
            ApplicationHomeDispatchPublication::Unresolved => {
                QueryDispatchPublicationOutcome::Unresolved
            }
        }
    }
    fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryDispatch,
    ) -> Result<(), QueryDispatchJournalDenial> {
        self.operation
            .acknowledge_no_effect(proof)
            .map_err(query_denial)
    }
}

fn query_denial(cause: ApplicationHomeDispatchJournalDenial) -> QueryDispatchJournalDenial {
    use ApplicationHomeDispatchJournalDenial as Home;
    use QueryDispatchJournalDenial as Query;
    match cause {
        Home::ForeignHome => Query::ForeignHome,
        Home::StaleIncarnation => Query::StaleIncarnation,
        Home::PrefixMismatch | Home::AdmissionUnavailable => Query::PrefixMismatch,
        Home::IdentityExhausted => Query::IdentityExhausted,
        Home::OwnerUnavailable => Query::OwnerUnavailable,
        Home::PublicationPending => Query::PublicationPending,
        Home::PublicationUnresolved => Query::PublicationUnresolved,
        Home::DeadlineUnavailable => {
            Query::Storage("the request deadline is below physical resolution".to_owned())
        }
        Home::Storage(detail) => Query::Storage(detail),
        Home::Execution(cause) => Query::Execution(cause),
    }
}
