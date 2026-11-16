use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
use crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_installation::facade::ApplicationSchema;

use super::denial;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;

/// A program-bound producer commits under the program its occurrence
/// actually selects. This checks the commit authority before canonical input
/// work; the prepared input's reuse key holds no program.
pub(super) fn require_selected_program<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    authority: &WorthQueryProducerCommitAuthority,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match authority {
        WorthQueryProducerCommitAuthority::Ordinary => {}
        WorthQueryProducerCommitAuthority::ProgramOutput
        | WorthQueryProducerCommitAuthority::SelectedProgram { .. } => {
            let actual = selected.inspect_selected_program().map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::PublicationStale,
                    "selected producer program could not be inspected",
                )
            })?;
            if let WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision } =
                authority
                && (identity != actual.identity() || revision != actual.revision())
            {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::PublicationStale,
                    "presented producer program differs from the selected occurrence",
                ));
            }
        }
    }
    Ok(())
}

/// An incomplete restored source cannot authorize a contact-free cutoff.
pub(super) fn prepared_key<Query>(
    source: &WorthQueryObservedSource<Query>,
    input_identity: [u8; 32],
    edition: InstalledProducerEdition,
) -> Result<Option<PreparedInputReuseKey>, WorthQueryOutputDemandDenial> {
    let Some(selection) = source.retain_selected_membership() else {
        return Ok(None);
    };
    Ok(Some(PreparedInputReuseKey::new(
        selection,
        input_identity,
        edition,
    )))
}

/// Capture only the decision context declared by this producer. Completion
/// still seals actual context use; this preparation alone proves no reuse.
pub(super) fn prepared_context(
    contract: Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerInputReuseContract>,
    key: [u8; 32],
    principal: impl FnOnce() -> crate::domain_computation::primary_graph::freshness::WorthQueryPrincipalReuseWitness,
    scope: impl FnOnce() -> crate::domain_computation::authorization::WorthQueryDecisionScopeWitness,
) -> Option<crate::domain_computation::primary_graph::output_lineage::PreparedDecisionReuseContext>
{
    let contract = contract.filter(|contract| {
        contract.determinism() == worth_foundational::facade::DeterminismContract::CanonicalBitwise
    })?;
    let declared = contract.context();
    use super::super::WorthQueryDecisionContextDependencies as Context;
    crate::domain_computation::primary_graph::output_lineage::PreparedDecisionReuseContext::new(
        contract,
        declared.contains(Context::KEY).then_some(key),
        declared.contains(Context::PRINCIPAL).then(principal),
        declared.contains(Context::SCOPE).then(scope),
    )
}
