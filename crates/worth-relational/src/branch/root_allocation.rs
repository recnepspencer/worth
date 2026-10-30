use super::RelationalBranchRoot;
use crate::branch::root_regions::RelationalPersistentRegionNodeObservation;
#[path = "root_allocation/union.rs"]
mod union;

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

pub(crate) trait RelationalRootAllocationVisitor {
    type Error;
    /// Returning false prunes the children of this immutable allocation.
    fn allocation(&mut self, allocation: RelationalRootAuthoritativeAllocationObservation) -> bool;
    fn region(
        &mut self,
        region: super::RelationalRootRegionObservation,
        partition: Result<
            Option<&crate::storage::overlay::PartitionState>,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    ) -> Result<(), Self::Error>;
}

impl From<RelationalPersistentRegionNodeObservation>
    for RelationalRootAuthoritativeAllocationObservation
{
    fn from(node: RelationalPersistentRegionNodeObservation) -> Self {
        use crate::branch::RelationalPersistentRegionAllocationKind as Region;
        let kind = match node.allocation_kind {
            Region::SetObject => {
                RelationalRootAuthoritativeAllocationKind::PersistentRegionSetObject
            }
            Region::MapNodeObject => {
                RelationalRootAuthoritativeAllocationKind::PersistentRegionMapNodeObject
            }
            Region::ReplacementStorage => {
                RelationalRootAuthoritativeAllocationKind::PersistentRegionReplacementStorage
            }
            Region::RemovalStorage => {
                RelationalRootAuthoritativeAllocationKind::PersistentRegionRemovalStorage
            }
        };
        Self {
            kind,
            owner_id: node.node_id,
            authoritative_bytes: node.authoritative_bytes,
        }
    }
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
            observations.push(node.into());
        }
        observations
    }
}
