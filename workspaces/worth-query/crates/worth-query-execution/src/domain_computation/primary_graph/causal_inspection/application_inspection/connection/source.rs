//! Original source affinity consumes the separately admitted source when selected.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
pub(super) fn validate(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommitReceipt,
) -> Result<Arc<OriginalSourceInspectionDisclosure>, MapKernelFailure<ConnectionRefusal>> {
    use WorthQueryProducerConnectionDenial as Denial;
    checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
    let original = target
        .original()
        .ok_or(MapKernelFailure::Domain(Denial::ForeignSource.into()))?;
    let WorthQueryApplicationBasisSelectionIdentity::Product(read) = &original.source.selection
    else {
        return Err(MapKernelFailure::Domain(Denial::ForeignSource.into()));
    };
    let publication = source.committed_product_publication();
    if source.authority_binding().runtime_authority().as_u64() != original.source.runtime_authority
        || publication.product_branch() != read.branch_identity()
        || publication.product_incarnation() != read.lifecycle_incarnation()
    {
        return Err(MapKernelFailure::Domain(Denial::ForeignSource.into()));
    }
    let disclosure = source
        .authority_binding()
        .retain_original_inspection_disclosure()
        .ok_or(MapKernelFailure::Domain(
            Denial::SourceDisclosureUnavailable.into(),
        ))?;
    if let Some(target) = target.connection_source() {
        let original = target.source().ok_or(MapKernelFailure::Domain(
            Denial::SourceDisclosureMismatch.into(),
        ))?;
        if !original.changes.shares_original(source.committed_changes())
            || !Arc::ptr_eq(&original.disclosure, &disclosure)
            || original.runtime != source.authority_binding().runtime_authority().as_u64()
            || source.authority_binding().original_mutation_binding()
                != Some(disclosure.association().source_type())
        {
            return Err(MapKernelFailure::Domain(
                Denial::SourceDisclosureMismatch.into(),
            ));
        }
        return Ok(disclosure);
    }
    if disclosure.association().declaration().parts().query
        != target.contract().definition().parts().binding
        || disclosure.scope().subject() != original.source.model_root
        || disclosure.disclosure().query() != &original.source.query_identity
        || disclosure.disclosure().parameters().identity()
            != &original.source.parameter_binding_identity
        || source.authority_binding().original_mutation_binding()
            != Some(disclosure.association().source_type())
    {
        return Err(MapKernelFailure::Domain(
            Denial::SourceDisclosureMismatch.into(),
        ));
    }
    Ok(disclosure)
}

pub(super) fn require_input(
    observed: &overlap::ObservationConnection<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommittedChanges,
    inputs: &FactPayload,
    source_diagnostic: Option<&DiagnosticComparisonAdmission<'_>>,
) -> Result<(), MapKernelFailure<ConnectionRefusal>> {
    if let Some(diagnostic) = source_diagnostic {
        observed.require_independent_input(
            context,
            source,
            inputs,
            diagnostic,
            target
                .connection_source()
                .and_then(|target| target.source())
                .ok_or(MapKernelFailure::Domain(
                    WorthQueryProducerConnectionDenial::SourceDisclosureMismatch.into(),
                ))?
                .disclosure
                .scope()
                .subject(),
        )
    } else {
        observed.require_input(context, source, inputs)
    }
}
