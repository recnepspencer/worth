//! Original source input flows into the ordinary SendMoney delivery owner.
use super::*;
pub(in crate::http::server) fn start(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    source: &WorthQueryCorrectionSourceCoordinates,
    controls: BankMutationControls,
    history: std::num::NonZeroUsize,
    page: &worth_query_host::facade::primary_graph::WorthQueryCorrectionAlternativesPage,
) -> Result<(SourceMutationCustody, Option<BankHttpDenial>), BankHttpDenial> {
    let limits = WorthQueryCorrectionHistoryLimits::new(Some(history), std::num::NonZeroUsize::MIN);
    let request = runtime.request(principal, controls.request());
    let alternative = request
        .corrections()
        .select_alternative_coordinates(page, source, limits)
        .map_err(correction_denial)?;
    let prepared = runtime
        .prepare_send_money_reapplication(
            principal,
            controls.request(),
            &alternative,
            controls.idempotency_key().as_str(),
            limits,
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
        WorthQueryOperationCorrectionPreparation::PublicationPending
        | WorthQueryOperationCorrectionPreparation::ProductUnpublished(_) => {
            return Err(denial(BankHttpDenialKind::Unavailable))
        }
    };
    let (custody, refusal) = match outcome {
        WorthQuerySourceOnlyAdvance::Progress(owner) => (ReapplicationCustody::Active(owner), None),
        WorthQuerySourceOnlyAdvance::Refused { operation, denial } => (
            ReapplicationCustody::Active(operation),
            Some(crate::http::server::mutation_refusal::source_denial(denial)),
        ),
        WorthQuerySourceOnlyAdvance::Completed(owner) => {
            (ReapplicationCustody::Completed(owner), None)
        }
        WorthQuerySourceOnlyAdvance::Terminal(owner) => {
            (ReapplicationCustody::Terminal(owner), None)
        }
    };
    Ok((SourceMutationCustody::Reapplication(custody), refusal))
}
