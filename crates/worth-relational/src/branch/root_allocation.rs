use super::RelationalBranchRoot;

#[cfg(test)]
#[path = "root_allocation_tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RelationalRootAuthoritativeAllocationKind {
    RootMetadata,
    SchemaAuthority,
    PersistentRegionSetObject,
    PersistentRegionMapNodeObject,
    PersistentRegionReplacementStorage,
    PersistentRegionRemovalStorage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalRootAuthoritativeAllocationObservation {
    pub(crate) kind: RelationalRootAuthoritativeAllocationKind,
    pub(crate) owner_id: u64,
    pub(crate) authoritative_bytes: u64,
}

impl RelationalBranchRoot {
    pub(crate) fn authoritative_allocation_observations(
        &self,
    ) -> Vec<RelationalRootAuthoritativeAllocationObservation> {
        let mut observations = vec![RelationalRootAuthoritativeAllocationObservation {
            kind: RelationalRootAuthoritativeAllocationKind::RootMetadata,
            // The borrowed root remains alive during inspection. Readmission
            // can preserve its semantic id in a different physical allocation.
            owner_id: self as *const Self as usize as u64,
            authoritative_bytes: std::mem::size_of::<Self>() as u64,
        }];
        observations.push(RelationalRootAuthoritativeAllocationObservation {
            kind: RelationalRootAuthoritativeAllocationKind::SchemaAuthority,
            owner_id: self.schema_authority().allocation_id(),
            authoritative_bytes: self.schema_authority().authoritative_allocation_bytes(),
        });
        for node in self.regions.allocation_observations() {
            let kind = match node.allocation_kind {
                crate::branch::RelationalPersistentRegionAllocationKind::SetObject => {
                    RelationalRootAuthoritativeAllocationKind::PersistentRegionSetObject
                }
                crate::branch::RelationalPersistentRegionAllocationKind::MapNodeObject => {
                    RelationalRootAuthoritativeAllocationKind::PersistentRegionMapNodeObject
                }
                crate::branch::RelationalPersistentRegionAllocationKind::ReplacementStorage => {
                    RelationalRootAuthoritativeAllocationKind::PersistentRegionReplacementStorage
                }
                crate::branch::RelationalPersistentRegionAllocationKind::RemovalStorage => {
                    RelationalRootAuthoritativeAllocationKind::PersistentRegionRemovalStorage
                }
            };
            observations.push(RelationalRootAuthoritativeAllocationObservation {
                kind,
                owner_id: node.node_id,
                authoritative_bytes: node.authoritative_bytes,
            });
        }
        observations
    }
}
