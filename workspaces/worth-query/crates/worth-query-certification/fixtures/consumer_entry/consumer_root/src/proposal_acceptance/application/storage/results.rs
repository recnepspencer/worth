//! Completion records require current admission and the actual sealed adoption owner.
use super::super::{
    super::attempts::{data::*, recording},
    candidate::adoption::Adopted,
};
use super::admission::{AdmissionFailure, ResultRows};
use super::operations::*;
use super::*;
#[derive(Debug)]
pub enum ResultReadFailure {
    ClaimMismatch,
    Missing,
    Ambiguous,
    AssociationChanged,
    UnsupportedVersion(u16),
}
pub enum ResultFailure {
    Admission(AdmissionFailure),
    Correspondence(recording::CorrespondenceFailure),
    Read(ResultReadFailure),
    Construction(ConstructionFailure),
    Pending(ResultOperation),
    Refused {
        operation: ResultOperation,
        cause: WorthQuerySourceOnlyDenial,
    },
    Terminal(ResultTerminal),
    RecordedObservation {
        original: StoredResult,
        completed: ResultCompleted,
        cause: ResultReadFailure,
    },
    RecordedAdmission {
        original: StoredResult,
        completed: ResultCompleted,
        cause: AdmissionFailure,
    },
    Close {
        original: StoredResult,
        completed: ResultCompleted,
        cause: WorthQueryOperationCloseDenial,
    },
}
impl std::fmt::Debug for ResultFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admission(cause) => f.debug_tuple("Admission").field(cause).finish(),
            Self::Correspondence(cause) => f.debug_tuple("Correspondence").field(cause).finish(),
            Self::Read(cause) => f.debug_tuple("Read").field(cause).finish(),
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
            Self::RecordedObservation { cause, .. } => {
                f.debug_tuple("RecordedObservation").field(cause).finish()
            }
            Self::RecordedAdmission { cause, .. } => {
                f.debug_tuple("RecordedAdmission").field(cause).finish()
            }
            Self::Close { cause, .. } => f.debug_tuple("Close").field(cause).finish(),
        }
    }
}
pub(in crate::proposal_acceptance::application) fn select_result(
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
        let operation = self
            .result_operation(caller, &command, attempt)
            .map_err(ResultFailure::Construction)?;
        let request = self
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch());
        let completed = match operation.advance(&request) {
            WorthQuerySourceOnlyAdvance::Progress(operation) => {
                return Err(ResultFailure::Pending(operation))
            }
            WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                return Err(ResultFailure::Refused {
                    operation,
                    cause: denial,
                })
            }
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
                return Err(ResultFailure::Terminal(terminal))
            }
            WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        };
        let rows = match self.admit_attempt(model, caller, loaded) {
            Ok(rows) => rows,
            Err(cause) => {
                return Err(ResultFailure::RecordedAdmission {
                    original,
                    completed,
                    cause,
                })
            }
        };
        let result = match select_result(&rows, loaded, claim) {
            Ok(result) => result,
            Err(cause) => {
                return Err(ResultFailure::RecordedObservation {
                    original,
                    completed,
                    cause,
                })
            }
        };
        match completed.close(&request) {
            Ok(_closed) => Ok(result),
            Err((completed, cause)) => Err(ResultFailure::Close {
                original,
                completed,
                cause,
            }),
        }
    }
}
