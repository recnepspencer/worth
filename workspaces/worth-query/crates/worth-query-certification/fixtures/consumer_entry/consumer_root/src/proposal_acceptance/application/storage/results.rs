//! Completion records require current admission and the actual sealed adoption owner.
use super::super::{
    super::attempts::{data::*, recording},
    candidate::adoption::Adopted,
};
use super::admission::{AdmissionFailure, ResultRows};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
pub type RecordOutcome =
    WorthQueryApplicationMutationOutcome<recording::Denial, recording::Recorded>;
#[derive(Debug)]
pub enum ResultReadFailure {
    ClaimMismatch,
    Missing,
    Ambiguous,
    AssociationChanged,
    UnsupportedVersion(u16),
}
#[derive(Debug)]
pub enum ResultFailure {
    Admission(AdmissionFailure),
    Correspondence(recording::CorrespondenceFailure),
    Read(ResultReadFailure),
    Request(WorthQueryApplicationRequestMutationDenial),
    Outcome(RecordOutcome),
    RecordedObservation {
        original: StoredResult,
        outcome: RecordOutcome,
        cause: ResultReadFailure,
    },
    RecordedAdmission {
        original: StoredResult,
        outcome: RecordOutcome,
        cause: AdmissionFailure,
    },
}
fn select_result(
    rows: &ResultRows,
    loaded: &LoadedProposal,
    claim: &ClaimReference,
) -> Result<StoredResult, ResultReadFailure> {
    if claim.data.input != loaded.input(claim.data.input.attempt) {
        return Err(ResultReadFailure::ClaimMismatch);
    }
    let mut matching = rows
        .rows()
        .iter()
        .filter(|row| row.data.claim.data.input.attempt == claim.data.input.attempt);
    match (matching.next(), matching.next()) {
        (Some(row), None) if row.data.claim != *claim => Err(ResultReadFailure::AssociationChanged),
        (Some(row), None) if row.data.format_version != 1 => Err(
            ResultReadFailure::UnsupportedVersion(row.data.format_version),
        ),
        (Some(row), None) => Ok(row.data.clone()),
        (None, _) => Err(ResultReadFailure::Missing),
        (Some(_), Some(_)) => Err(ResultReadFailure::Ambiguous),
    }
}
impl Storage<'_> {
    pub fn read_result(
        &self,
        model: &Model<'_>,
        caller: &Caller<'_>,
        loaded: &LoadedProposal,
        claim: &ClaimReference,
    ) -> Result<StoredResult, ResultFailure> {
        let rows = self
            .admit_attempt(model, caller, loaded)
            .map_err(ResultFailure::Admission)?;
        select_result(&rows, loaded, claim).map_err(ResultFailure::Read)
    }
    pub fn record_result(
        &self,
        model: &Model<'_>,
        caller: &Caller<'_>,
        loaded: &LoadedProposal,
        claim: &ClaimReference,
        adopted: &Adopted<'_>,
    ) -> Result<StoredResult, ResultFailure> {
        self.admit_attempt(model, caller, loaded)
            .map_err(ResultFailure::Admission)?;
        if claim.data.input != loaded.input(claim.data.input.attempt) {
            return Err(ResultFailure::Read(ResultReadFailure::ClaimMismatch));
        }
        let command = recording::RecordResult::from_adopted(claim, adopted)
            .map_err(ResultFailure::Correspondence)?;
        let original = command.data().clone();
        let attempt = claim.data.input.attempt;
        let outcome = self
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .advanced()
            .mutate(command)
            .without_source()
            .idempotency(&attempt)
            .execute_in_program(self.owner.host, ExecutionAllocationPolicy::SystemAllocation)
            .map_err(ResultFailure::Request)?;
        if !matches!(
            &outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
                | WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
                | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
        ) {
            return Err(ResultFailure::Outcome(outcome));
        }
        let rows = match self.admit_attempt(model, caller, loaded) {
            Ok(rows) => rows,
            Err(cause) => {
                return Err(ResultFailure::RecordedAdmission {
                    original,
                    outcome,
                    cause,
                })
            }
        };
        select_result(&rows, loaded, claim).map_err(|cause| ResultFailure::RecordedObservation {
            original,
            outcome,
            cause,
        })
    }
}
