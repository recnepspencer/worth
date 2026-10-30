//! Registration resolves on the same storage product before any candidate-basis comparison.
use super::super::super::attempts::{claim, data::*, query};
use super::admission::AdmissionFailure;
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
pub type ClaimOutcome = WorthQueryApplicationMutationOutcome<claim::Denial, claim::Registered>;
pub struct RegisteredClaim {
    reference: ClaimReference,
    outcome: ClaimOutcome,
}
impl RegisteredClaim {
    pub fn reference(&self) -> &ClaimReference {
        &self.reference
    }
    pub fn outcome(&self) -> &ClaimOutcome {
        &self.outcome
    }
}
#[derive(Debug)]
pub enum ClaimReadFailure {
    Request(Failure),
    Missing,
    Ambiguous,
    Correspondence,
}
#[derive(Debug)]
pub enum ClaimFailure {
    Admission(AdmissionFailure),
    Request(WorthQueryApplicationRequestMutationDenial),
    Outcome(ClaimOutcome),
    RegisteredObservation {
        input: AttemptInput,
        outcome: ClaimOutcome,
        cause: ClaimReadFailure,
    },
}
impl Storage<'_> {
    pub fn claim(
        &self,
        model: &Model<'_>,
        caller: &Caller<'_>,
        loaded: &LoadedProposal,
        attempt: u64,
    ) -> Result<RegisteredClaim, ClaimFailure> {
        self.admit_attempt(model, caller, loaded)
            .map_err(ClaimFailure::Admission)?;
        let input = loaded.input(attempt);
        let outcome = self
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .advanced()
            .mutate(input.clone())
            .without_source()
            .idempotency(&attempt)
            .execute_in_program(self.owner.host, ExecutionAllocationPolicy::SystemAllocation)
            .map_err(ClaimFailure::Request)?;
        if !matches!(
            &outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
                | WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
                | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
        ) {
            return Err(ClaimFailure::Outcome(outcome));
        }
        let reference = match self.read_claim(caller, &input) {
            Ok(reference) => reference,
            Err(cause) => {
                return Err(ClaimFailure::RegisteredObservation {
                    input,
                    outcome,
                    cause,
                })
            }
        };
        // read_claim consumes no outcome on success; keep the actual ordinary evidence.
        Ok(RegisteredClaim { reference, outcome })
    }
    fn read_claim(
        &self,
        caller: &Caller<'_>,
        input: &AttemptInput,
    ) -> Result<ClaimReference, ClaimReadFailure> {
        let rows = self
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .query(query::ReadClaims {
                model: self.model.label.clone(),
            })
            .execute_governed(
                access::ReadDomain::reference(),
                access::InspectDomain::reference(),
                access::ReadAccess {
                    model: self.model.label.clone(),
                },
            )
            .map_err(|cause| ClaimReadFailure::Request(Failure::Query(cause)))?;
        let mut matching = rows
            .rows()
            .iter()
            .filter(|row| row.data.input.attempt == input.attempt);
        match (matching.next(), matching.next()) {
            (Some(row), None) if row.data.input == *input => Ok(ClaimReference {
                occurrence: row.occurrence.clone(),
                data: row.data.clone(),
            }),
            (Some(_), None) => Err(ClaimReadFailure::Correspondence),
            (None, _) => Err(ClaimReadFailure::Missing),
            (Some(_), Some(_)) => Err(ClaimReadFailure::Ambiguous),
        }
    }
}
