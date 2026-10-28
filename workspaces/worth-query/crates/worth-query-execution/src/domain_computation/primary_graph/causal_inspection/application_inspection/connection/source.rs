//! Original source affinity consumes the separately admitted source when selected.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
pub(super) fn validate(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommitReceipt,
) -> Result<
    Arc<OriginalSourceInspectionDisclosure>,
    MapKernelFailure<WorthQueryProducerConnectionDenial>,
> {
    use WorthQueryProducerConnectionDenial as Denial;
    checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
    let original = target
        .original()
        .ok_or(MapKernelFailure::Domain(Denial::ForeignSource))?;
    let WorthQueryApplicationBasisSelectionIdentity::Product(read) = &original.source.selection
    else {
        return Err(MapKernelFailure::Domain(Denial::ForeignSource));
    };
    let publication = source.committed_product_publication();
    if source.authority_binding().runtime_authority().as_u64() != original.source.runtime_authority
        || publication.product_branch() != read.branch_identity()
        || publication.product_incarnation() != read.lifecycle_incarnation()
    {
        return Err(MapKernelFailure::Domain(Denial::ForeignSource));
    }
    let disclosure = source
        .authority_binding()
        .retain_original_inspection_disclosure()
        .ok_or(MapKernelFailure::Domain(
            Denial::SourceDisclosureUnavailable,
        ))?;
    if let Some(target) = target.connection_source() {
        let original = target
            .source()
            .ok_or(MapKernelFailure::Domain(Denial::SourceDisclosureMismatch))?;
        if !original.changes.shares_original(source.committed_changes())
            || !Arc::ptr_eq(&original.disclosure, &disclosure)
            || original.runtime != source.authority_binding().runtime_authority().as_u64()
            || source.authority_binding().original_mutation_binding()
                != Some(disclosure.association().source_type())
        {
            return Err(MapKernelFailure::Domain(Denial::SourceDisclosureMismatch));
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
        return Err(MapKernelFailure::Domain(Denial::SourceDisclosureMismatch));
    }
    Ok(disclosure)
}
