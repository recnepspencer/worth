//! Check sealed original inputs before any Native or Source preparation.
use super::{home_program_transition_body_digest, RelationalHomeProgramBodyDenial};
use crate::durability::medium::RelationalHomeProgramWriteDenial as Denial;
use crate::mvcc::{BranchBoundRelationalTransaction, RelationalPublicationDenial};
use crate::transactions::data::TransactionCommitError;
use worth_execution::ExecutionRequest;

impl BranchBoundRelationalTransaction {
    pub(crate) fn require_home_program_body(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let Some(step) = &self.home_step else {
            return Ok(());
        };
        let batches = self.batches();
        let authored = match &self.home_target_coverage {
            None => batches,
            Some(coverage) => {
                let (authored, suffix) = batches
                    .split_at_checked(coverage.authored_batches)
                    .ok_or_else(|| denied(Denial::BodyMismatch))?;
                if suffix.len() != 1 {
                    return Err(denied(Denial::BodyMismatch));
                }
                let actual = home_program_transition_body_digest(
                    request,
                    &crate::mvcc::RelationalTransactionIntent::ordinary(),
                    suffix,
                )
                .map_err(body_denial)?;
                if actual != coverage.rejudgment_body {
                    return Err(denied(Denial::BodyMismatch));
                }
                authored
            }
        };
        let actual = home_program_transition_body_digest(request, &self.intent, authored)
            .map_err(body_denial)?;
        if actual.bytes() != step.admission.body_commitment() {
            return Err(denied(Denial::BodyMismatch));
        }
        Ok(())
    }
}

fn denied(cause: Denial) -> TransactionCommitError {
    TransactionCommitError::publication_denied(RelationalPublicationDenial::HomeProgramTransition(
        cause,
    ))
}
pub(super) fn body_denial(cause: RelationalHomeProgramBodyDenial) -> TransactionCommitError {
    match cause {
        RelationalHomeProgramBodyDenial::Request(cause) => {
            TransactionCommitError::execution(cause.into())
        }
        RelationalHomeProgramBodyDenial::CapacityOverflow => denied(Denial::BodyCapacityOverflow),
        RelationalHomeProgramBodyDenial::Encoding => denied(Denial::BodyEncoding),
        RelationalHomeProgramBodyDenial::UnsupportedMaterialization => {
            denied(Denial::UnsupportedBody)
        }
    }
}
