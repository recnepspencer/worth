//! Recorded estate restoration retains its source-completion owner across HTTP commands.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceCoordinates;

pub(in super::super) fn start(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    source: &WorthQueryCorrectionSourceCoordinates,
    controls: BankMutationControls,
    history: std::num::NonZeroUsize,
) -> Result<(SourceMutationCustody, Option<BankHttpDenial>), BankHttpDenial> {
    let prepared = runtime
        .prepare_estate_freeze_correction(
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
        WorthQuerySourceOnlyAdvance::Progress(owner) => {
            (EstateCorrectionCustody::Active(owner), None)
        }
        WorthQuerySourceOnlyAdvance::Refused {
            operation: owner,
            denial,
        } => (
            EstateCorrectionCustody::Active(owner),
            Some(super::super::super::mutation_refusal::source_denial(denial)),
        ),
        WorthQuerySourceOnlyAdvance::Completed(owner) => {
            (EstateCorrectionCustody::Completed(owner), None)
        }
        WorthQuerySourceOnlyAdvance::Terminal(owner) => {
            (EstateCorrectionCustody::Terminal(owner), None)
        }
    };
    Ok((SourceMutationCustody::EstateCorrection(custody), refusal))
}
