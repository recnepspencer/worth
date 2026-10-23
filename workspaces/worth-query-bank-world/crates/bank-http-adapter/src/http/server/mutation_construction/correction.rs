//! The serialized money worker stores the actual correction owner before replying.
use super::*;
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryCorrectionSourceCoordinates,
};
pub(super) mod estate;
mod refusal;
use refusal::correction_denial;

pub(super) fn start(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    source: &WorthQueryCorrectionSourceCoordinates,
    controls: BankMutationControls,
    history: std::num::NonZeroUsize,
) -> Result<(SourceMutationCustody, Option<BankHttpDenial>), BankHttpDenial> {
    let prepared = runtime
        .prepare_send_money_correction(
            principal,
            controls.request(),
            source,
            controls.idempotency_key().as_str(),
            WorthQueryCorrectionHistoryLimits::new(
                Some(history),
                std::num::NonZeroUsize::new(1).unwrap(),
            ),
        )
        .map_err(correction_denial)?;
    let outcome = match prepared {
        WorthQueryOperationCorrectionPreparation::Prepared(prepared) => {
            prepared.publish().map_err(correction_denial)?
        }
        WorthQueryOperationCorrectionPreparation::Continued(outcome) => outcome,
        WorthQueryOperationCorrectionPreparation::IdempotencyIntentDrift => {
            return Err(denial(BankHttpDenialKind::Stale))
        }
        // These are exact-key observations of an existing native publication owner.
        WorthQueryOperationCorrectionPreparation::PublicationPending
        | WorthQueryOperationCorrectionPreparation::ProductUnpublished(_) => {
            return Err(denial(BankHttpDenialKind::Unavailable))
        }
    };
    let (custody, refusal) = match outcome {
        WorthQuerySourceOnlyAdvance::Progress(owner) => (CorrectionCustody::Active(owner), None),
        WorthQuerySourceOnlyAdvance::Refused {
            operation: owner,
            denial,
        } => (
            CorrectionCustody::Active(owner),
            Some(super::super::mutation_refusal::source_denial(denial)),
        ),
        WorthQuerySourceOnlyAdvance::Completed(owner) => {
            (CorrectionCustody::Completed(owner), None)
        }
        WorthQuerySourceOnlyAdvance::Terminal(owner) => (CorrectionCustody::Terminal(owner), None),
    };
    Ok((SourceMutationCustody::Correction(custody), refusal))
}
