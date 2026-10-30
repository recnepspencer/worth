use super::{RelationalBranchRoot, RelationalRootAuthoritativeAllocationKind};
use crate::branch::RelationalPersistentRegionAllocationKind;

/// Allocation-owner evidence assembled independently of the sharing summary.
///
/// This deliberately does not call `authoritative_allocation_observations`:
/// certification compares this owner walk with the separately derived sharing
/// inventory so an omitted inventory category makes the comparison fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalRootOwnerAllocationLedgerEntry {
    pub(crate) kind: RelationalRootAuthoritativeAllocationKind,
    pub(crate) owner_id: u64,
    pub(crate) authoritative_bytes: u64,
}

impl RelationalBranchRoot {
    pub(crate) fn owner_allocation_ledger_entries(
        &self,
    ) -> Vec<RelationalRootOwnerAllocationLedgerEntry> {
        let mut entries = vec![RelationalRootOwnerAllocationLedgerEntry {
            kind: RelationalRootAuthoritativeAllocationKind::RootMetadata,
            // Allocation identity, independent of the semantic root identity
            // that a separately allocated readmission can preserve.
            owner_id: self as *const Self as usize as u64,
            authoritative_bytes: std::mem::size_of::<Self>() as u64,
        }];
        entries.push(RelationalRootOwnerAllocationLedgerEntry {
            kind: RelationalRootAuthoritativeAllocationKind::SchemaAuthority,
            owner_id: self.schema_authority.allocation_id(),
            authoritative_bytes: self.schema_authority.authoritative_allocation_bytes(),
        });
        entries.extend(
            self.regions
                .allocation_observations()
                .into_iter()
                .map(|observation| RelationalRootOwnerAllocationLedgerEntry {
                    kind: owner_kind(observation.allocation_kind),
                    owner_id: observation.node_id,
                    authoritative_bytes: observation.authoritative_bytes,
                }),
        );
        entries
    }
}

fn owner_kind(
    kind: RelationalPersistentRegionAllocationKind,
) -> RelationalRootAuthoritativeAllocationKind {
    match kind {
        RelationalPersistentRegionAllocationKind::SetObject => {
            RelationalRootAuthoritativeAllocationKind::PersistentRegionSetObject
        }
        RelationalPersistentRegionAllocationKind::MapNodeObject => {
            RelationalRootAuthoritativeAllocationKind::PersistentRegionMapNodeObject
        }
        RelationalPersistentRegionAllocationKind::ReplacementStorage => {
            RelationalRootAuthoritativeAllocationKind::PersistentRegionReplacementStorage
        }
        RelationalPersistentRegionAllocationKind::RemovalStorage => {
            RelationalRootAuthoritativeAllocationKind::PersistentRegionRemovalStorage
        }
    }
}
