//! Canonical records are descriptive data; none can reconstruct adoption or permission.
use super::super::schema::data::*;
use worth_query_decl::facade::{application_schema::WorthValue, worth_query_value_binding};

#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct AttemptInput {
    pub attempt: u64,
    pub reference: ProposalReference,
    pub proposal: StoredProposal,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct StoredClaim {
    pub input: AttemptInput,
    pub principal: Coordinates,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct ClaimReference {
    pub occurrence: Coordinates,
    pub data: StoredClaim,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub enum StoredStepOutcome {
    Fresh { days: u64 },
    Recovered,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct StoredResult {
    pub format_version: u16,
    pub claim: ClaimReference,
    pub steps: Vec<StoredStepOutcome>,
}
worth_query_value_binding!(pub ClaimDataBinding for StoredClaim {identity:"proposal-application.stored-claim.v1",scalar:String,canonical,});
worth_query_value_binding!(pub ResultDataBinding for StoredResult {identity:"proposal-application.stored-result.v1",scalar:String,canonical,});
