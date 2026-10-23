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
        source
            .occurrence()
            .committed_product_publication()
            .composite_commit()
            .clone(),
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
        .execution_request_for(&target.application().product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    if target
        .application()
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
    match source.occurrence().aftermath_causality() {
        None => Ok(WorthQueryCorrectionLineageBinding::reverse(locator, None)),
        Some(fact) if fact.role() == WorthQueryAftermathCausalRole::RecordedReapply => {
            let prior = fact
                .correction_lineage()
                .ok_or(Denial::UnsupportedSourceMeaning)?;
            Ok(WorthQueryCorrectionLineageBinding::reverse(
                prior.root().clone(),
                Some(locator),
            ))
        }
        Some(_) => Err(Denial::UnsupportedSourceMeaning),
    }
}
