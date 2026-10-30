//! Physical allocation union; unlike a logical root report, shared children prune.
use super::*;
#[cfg(test)]
#[path = "union_tests.rs"]
mod tests;

impl RelationalBranchRoot {
    pub(crate) fn try_visit_authoritative_allocation_union<V: RelationalRootAllocationVisitor>(
        &self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        visitor.allocation(self.metadata_allocation_observation());
        visitor.allocation(RelationalRootAuthoritativeAllocationObservation {
            kind: RelationalRootAuthoritativeAllocationKind::SchemaAuthority,
            owner_id: self.schema_authority().allocation_id(),
            authoritative_bytes: self.schema_authority().authoritative_allocation_bytes(),
        });
        self.regions.try_visit_allocation_union(visitor)
    }
}
