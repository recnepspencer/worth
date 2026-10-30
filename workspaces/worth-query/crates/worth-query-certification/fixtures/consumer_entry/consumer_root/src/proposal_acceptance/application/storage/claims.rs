//! Registration resolves on the same storage product before any candidate-basis comparison.
use super::super::super::attempts::{data::*, query};
use super::admission::AdmissionFailure;
use super::operations::*;
use super::*;
pub struct RegisteredClaim {
    reference: ClaimReference,
    evidence: ClaimCompletion,
}
impl RegisteredClaim {
    pub fn reference(&self) -> &ClaimReference {
        &self.reference
    }
    pub fn evidence(&self) -> &ClaimCompletion {
        &self.evidence
    }
}
#[derive(Debug)]
pub enum ClaimReadFailure {
    Request(Failure),
    Missing,
    Ambiguous,
    Correspondence,
}
pub enum ClaimFailure {
    Admission(AdmissionFailure),
    Construction(ConstructionFailure),
    Pending(ClaimOperation),
    Refused {
        operation: ClaimOperation,
        cause: WorthQuerySourceOnlyDenial,
    },
    Terminal(ClaimTerminal),
    RegisteredObservation {
        input: AttemptInput,
        completed: ClaimCompleted,
        cause: ClaimReadFailure,
    },
    Close {
        completed: ClaimCompleted,
        cause: WorthQueryOperationCloseDenial,
    },
}
impl std::fmt::Debug for ClaimFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admission(cause) => f.debug_tuple("Admission").field(cause).finish(),
            Self::Construction(cause) => f.debug_tuple("Construction").field(cause).finish(),
            Self::Pending(operation) => f
                .debug_tuple("Pending")
                .field(&operation.posture())
                .finish(),
            Self::Refused { cause, .. } => f.debug_tuple("Refused").field(cause).finish(),
            Self::Terminal(terminal) => f
                .debug_tuple("Terminal")
                .field(&terminal.native_outcome())
                .field(&terminal.native_keyed_resolution())
                .finish(),
            Self::RegisteredObservation { cause, .. } => {
                f.debug_tuple("RegisteredObservation").field(cause).finish()
            }
            Self::Close { cause, .. } => f.debug_tuple("Close").field(cause).finish(),
        }
    }
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
        let operation = self
            .claim_operation(caller, &input, attempt)
            .map_err(ClaimFailure::Construction)?;
        let request = self
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch());
        let completed = match operation.advance(&request) {
            WorthQuerySourceOnlyAdvance::Progress(operation) => {
                return Err(ClaimFailure::Pending(operation))
            }
            WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                return Err(ClaimFailure::Refused {
                    operation,
                    cause: denial,
                })
            }
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
                return Err(ClaimFailure::Terminal(terminal))
            }
            WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        };
        let reference = match self.read_claim(caller, &input) {
            Ok(reference) => reference,
            Err(cause) => {
                return Err(ClaimFailure::RegisteredObservation {
                    input,
                    completed,
                    cause,
                })
            }
        };
        match completed.close_with_evidence(&request) {
            Ok((_closed, evidence)) => Ok(RegisteredClaim {
                reference,
                evidence,
            }),
            Err((completed, cause)) => Err(ClaimFailure::Close { completed, cause }),
        }
    }
    pub(in crate::proposal_acceptance::application) fn read_claim(
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
