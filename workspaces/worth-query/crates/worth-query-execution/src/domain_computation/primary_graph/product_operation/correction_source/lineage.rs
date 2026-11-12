//! First-use reversal derives its canonical lineage at the exact selected basis.
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionLineageBinding,
    WorthQueryRecordedCorrectionSource,
};

pub(super) fn source_locator<Schema>(
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
) -> WorthQueryRecordedCorrectionSource {
    WorthQueryRecordedCorrectionSource::from_untrusted_parts(
        source.ancestry().source().identity().clone(),
        source.occurrence().commit_reference().clone(),
    )
}

pub(super) fn reverse_lineage<Schema: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
) -> Result<WorthQueryCorrectionLineageBinding, WorthQueryCorrectionDerivationDenial> {
    use WorthQueryCorrectionDerivationDenial as Denial;
    let locator = source_locator(source);
    let execution = phase
        .execution_request_for(&target.application().runtime.product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    if target
        .application()
        .runtime
        .primary_provider
        .reversal_of_source_at_basis(&execution, target.product().relational_basis(), &locator)
        .map_err(Denial::History)?
        .is_some()
    {
        return Err(Denial::SourceAlreadyReversed);
    }
    reverse_identity(source)
}

pub(super) fn reverse_identity<Schema>(
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
) -> Result<WorthQueryCorrectionLineageBinding, WorthQueryCorrectionDerivationDenial> {
    use WorthQueryCorrectionDerivationDenial as Denial;
    let locator = source_locator(source);
    match source.occurrence().causality() {
        None => Ok(WorthQueryCorrectionLineageBinding::reverse(locator, None)),
        Some(fact) => match fact.role() {
            WorthQueryAftermathCausalRole::RecordedReapply => {
                Ok(WorthQueryCorrectionLineageBinding::reverse(
                    fact.correction_lineage().root().clone(),
                    Some(locator),
                ))
            }
            WorthQueryAftermathCausalRole::RecordedReverse => Err(Denial::UnsupportedSourceMeaning),
        },
    }
}
