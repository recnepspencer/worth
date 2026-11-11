//! Admission derives reuse solely from the registered ordered membership.
use super::*;
use crate::derived_relation::DerivedRelationNode as Node;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrderedSourceRecipe {
    pub(crate) predecessor: KindId,
    pub(crate) introduced: KindId,
    pub(crate) retired: KindId,
}

impl RelationalSchemaRegistry {
    /// Bind birth sharing through this membership's own to-one order relation.
    /// Neither a predecessor coordinate nor a provider root can be supplied here.
    pub fn bind_ordered_predecessor_managed_index_source(
        &self,
        path: &DerivedRelationHandle,
        fields: Vec<ManagedIndexSourceField>,
        routes: Vec<ManagedIndexSourceProjectionRoute>,
    ) -> Result<ManagedIndexSourceProgram, Denial> {
        if !self.admits_derived_relation(path) {
            return Err(Denial::ForeignDefinition);
        }
        let membership = ordered_membership(self, path)?;
        let Some(Node::OrderedEvents {
            predecessor,
            introduced,
            retired,
        }) = membership.parts().nodes.last()
        else {
            return Err(Denial::OrderedPredecessorMembershipRequired);
        };
        let Node::Stored { relation, .. } = membership.parts().nodes[*predecessor] else {
            return Err(Denial::OrderedPredecessorMembershipRequired);
        };
        let leaf = |index: usize| match membership.parts().nodes[index] {
            Node::Stored { relation, .. } => Ok(relation),
            _ => Err(Denial::OrderedPredecessorMembershipRequired),
        };
        let ordered = OrderedSourceRecipe {
            predecessor: relation,
            introduced: leaf(*introduced)?,
            retired: leaf(*retired)?,
        };
        let registration = self
            .relation_registration(relation)
            .map_err(|_| Denial::InvalidPhysicalSource(relation))?;
        if !registration
            .relation_integrity
            .cardinality_contracts
            .iter()
            .any(|contract| contract.source_max == Some(1))
        {
            return Err(Denial::OrderedPredecessorMustBeToOne);
        }
        let row = path
            .parts()
            .to
            .entity_kind()
            .ok_or(Denial::InvalidMemberDomain)?;
        path.parts()
            .from
            .entity_kind()
            .ok_or(Denial::InvalidMemberDomain)?;
        super::projection_routes::validate_fields(self, row, &fields)?;
        super::projection_routes::validate_routes(self, row, &routes)?;
        super::projection_routes::validate_route_names(&routes)?;
        path.entry
            .managed_source_binding
            .bind(path, fields, routes, Some(ordered))
    }
}

pub(super) fn ordered_membership(
    registry: &RelationalSchemaRegistry,
    path: &DerivedRelationHandle,
) -> Result<DerivedRelationHandle, Denial> {
    match path.parts().nodes.as_slice() {
        [Node::Derived(identity)] => {
            let membership = registry
                .derived_relation(identity)
                .ok_or_else(|| Denial::MissingDefinition(identity.clone()))?;
            if matches!(
                membership.parts().nodes.last(),
                Some(Node::OrderedEvents { .. })
            ) {
                Ok(membership)
            } else {
                Err(Denial::OrderedPredecessorMembershipRequired)
            }
        }
        nodes if matches!(nodes.last(), Some(Node::OrderedEvents { .. })) => Ok(path.clone()),
        _ => Err(Denial::OrderedPredecessorMembershipRequired),
    }
}

#[cfg(test)]
mod tests;
