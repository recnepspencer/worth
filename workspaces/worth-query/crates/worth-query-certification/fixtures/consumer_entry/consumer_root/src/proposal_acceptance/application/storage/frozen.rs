//! Governed loading freezes domain meaning without retaining read or execution authority.
use super::super::super::attempts::data::AttemptInput;
use super::*;

#[derive(Clone, Debug)]
pub struct LoadedProposal {
    source: WorthQueryProductBranch,
    reference: ProposalReference,
    proposal: StoredProposal,
}
impl LoadedProposal {
    pub fn reference(&self) -> &ProposalReference {
        &self.reference
    }
    pub fn proposal(&self) -> &StoredProposal {
        &self.proposal
    }
    pub fn input(&self, attempt: u64) -> AttemptInput {
        AttemptInput {
            attempt,
            reference: self.reference.clone(),
            proposal: self.proposal.clone(),
        }
    }
}
impl Storage<'_> {
    pub fn load_frozen(
        &self,
        caller: &Caller<'_>,
        reference: &ProposalReference,
    ) -> Result<LoadedProposal, Failure> {
        self.load(caller, reference).map(|proposal| LoadedProposal {
            source: self.branch(),
            reference: reference.clone(),
            proposal,
        })
    }
    pub fn admit_proposal_read(&self, caller: &Caller<'_>) -> Result<(), Failure> {
        // Authorize the current complete collection; do not substitute its latest
        // record for an already selected frozen proposal occurrence/revision.
        self.owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .query(storage_query::ReadProposals {
                model: self.model.label.clone(),
            })
            .execute_governed(
                access::ReadDomain::reference(),
                access::InspectDomain::reference(),
                access::ReadAccess {
                    model: self.model.label.clone(),
                },
            )
            .map(|_| ())
            .map_err(Failure::Query)
    }
    pub fn admit_loaded_proposal(
        &self,
        caller: &Caller<'_>,
        loaded: &LoadedProposal,
    ) -> Result<(), Failure> {
        self.admit_proposal_read(caller)?;
        // Exact selectors include their World owner and lifecycle incarnation.
        // Routing data grants no READ permission; the current query above does.
        if self.branch() != loaded.source {
            return Err(Failure::StorageMismatch);
        }
        Ok(())
    }
}
