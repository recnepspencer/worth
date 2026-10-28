//! Original source interpretation must be exactly the currently admitted binding.
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
    let original = target.original();
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
