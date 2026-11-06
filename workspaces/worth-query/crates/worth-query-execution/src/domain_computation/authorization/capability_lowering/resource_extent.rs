//! Lower the indexed grant head and prove its context-anchored continuation to scope.
use {crate::domain_computation::authorization::capability_binding_lowering::kind, crate::domain_computation::authorization::capability_registry::WorthQueryCapabilityContextAnchor, crate::domain_computation::authorization::lowering::lower_authorization_traversal, crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial, crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout};
use worth_query_declaration::facade::application_capability::ErasedApplicationCapabilityContract;
use worth_relational::facade::authorization::RelationalAuthorizationTraversal;

pub(super) fn lower(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    grantee: RelationalAuthorizationTraversal,
    resource: RelationalAuthorizationTraversal,
) -> Result<
    (
        Vec<RelationalAuthorizationTraversal>,
        Vec<WorthQueryCapabilityContextAnchor>,
    ),
    WorthQueryOperationAuthorizationDenial,
> {
    let path = contract.target().resource_path();
    let mut traversals = vec![grantee, resource];
    for step in path.traversals() {
        traversals.push(lower_authorization_traversal(layout, step)?);
    }
    let anchors = path
        .root_slot()
        .map(|slot| {
            Ok(WorthQueryCapabilityContextAnchor {
                ordinal: 2,
                kind: kind(layout, slot.entity())?,
                context: slot.context().into(),
                context_type: slot.context_identity().as_str().into(),
                slot: slot.slot().into(),
                slot_type: slot.slot_identity().as_str().into(),
                entity: slot.entity().into(),
            })
        })
        .transpose()?
        .into_iter()
        .collect();
    Ok((traversals, anchors))
}
