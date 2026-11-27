//! Lower `require_active` extent liveness onto the grant path.
//!
//! Each binding becomes an equality predicate on every grant-path ordinal from
//! the head (or root) onward whose entity is the binding's kind. The grant
//! template carries them, so candidate selection, revalidation and every
//! delegated parent's re-observation judge the same liveness.
use worth_query_declaration::facade::application_capability::ErasedApplicationCapabilityContract;
use worth_relational::facade::authorization::{
    RelationalAuthorizationPredicate, RelationalAuthorizationTraversal,
    RelationalAuthorizationTraversalDirection,
};
use worth_relational::facade::identity::KindId;

use super::super::capability_binding_lowering::{kind, predicate};
use super::super::{authorization_denial, WorthQueryOperationAuthorizationDenial};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

/// Path ordinal of the head resource (or context root): principal is 0, the
/// grant 1, and the grant-resource relation reaches ordinal 2.
const FIRST_EXTENT_ORDINAL: usize = 2;

pub fn lower(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    traversals: &[RelationalAuthorizationTraversal],
) -> Result<Vec<RelationalAuthorizationPredicate>, WorthQueryOperationAuthorizationDenial> {
    let mut predicates = Vec::new();
    for status in contract.constraints().currentness().extent_status() {
        let status_kind = kind(layout, status.field().entity())?;
        let before = predicates.len();
        for (ordinal, reached) in reached_kinds(traversals) {
            if ordinal >= FIRST_EXTENT_ORDINAL && reached == status_kind {
                predicates.push(predicate(layout, ordinal, status_kind, status)?);
            }
        }
        if predicates.len() == before {
            return Err(authorization_denial(
                contract.name(),
                "capability extent status names an entity the grant path never crosses",
            ));
        }
    }
    Ok(predicates)
}

/// The entity kind each traversal reaches, by its path ordinal.
fn reached_kinds(
    traversals: &[RelationalAuthorizationTraversal],
) -> impl Iterator<Item = (usize, KindId)> + '_ {
    traversals.iter().enumerate().map(|(index, traversal)| {
        let reached = match traversal.direction() {
            RelationalAuthorizationTraversalDirection::Forward => traversal.to_kind(),
            RelationalAuthorizationTraversalDirection::Reverse => traversal.from_kind(),
        };
        (index + 1, reached)
    })
}
