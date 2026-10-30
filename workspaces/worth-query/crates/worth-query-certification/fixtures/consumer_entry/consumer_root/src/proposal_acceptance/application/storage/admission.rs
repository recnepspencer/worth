//! Current disclosure and exact frozen mutation assessment precede protected attempt lookup.
use super::super::super::attempts::result_query;
use super::super::candidate::preparation::{self, PreparationRefusal};
use super::*;
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationResult;
pub(in crate::proposal_acceptance::application) type ResultRows =
    WorthQueryPublishedApplicationResult<result_query::ResultQuery, result_query::ResultRow>;
#[derive(Debug)]
pub enum AdmissionFailure {
    Request(Failure),
    Preparation(PreparationRefusal),
    StorageModelMismatch,
}
impl Storage<'_> {
    pub(in crate::proposal_acceptance::application) fn admit_attempt(
        &self,
        model: &Model<'_>,
        caller: &Caller<'_>,
        loaded: &LoadedProposal,
    ) -> Result<ResultRows, AdmissionFailure> {
        self.admit_loaded_proposal(caller, loaded)
            .map_err(AdmissionFailure::Request)?;
        preparation::authorize_attempt(model, caller, loaded.proposal())
            .map_err(AdmissionFailure::Preparation)?;
        if self.model != model.identity {
            return Err(AdmissionFailure::StorageModelMismatch);
        }
        self.owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .query(result_query::ReadResults {
                model: self.model.label.clone(),
            })
            .execute_governed(
                access::ReadDomain::reference(),
                access::InspectDomain::reference(),
                access::ReadAccess {
                    model: self.model.label.clone(),
                },
            )
            .map_err(|cause| AdmissionFailure::Request(Failure::Query(cause)))
    }
}
