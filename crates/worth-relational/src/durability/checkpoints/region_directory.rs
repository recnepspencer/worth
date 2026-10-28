//! Readmit a manifest's immutable region directory without reading members.
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::branch::{RelationalBranchRootSchemaAuthority, RelationalRootRegionCheckpoint};
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalCheckpointDirectory,
    RelationalCheckpointRegion, RelationalRecoveryStop,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::PartitionId;
use crate::storage::residency::{
    RelationalColdRegion, RelationalRegionKindAuthority, RelationalRegionResidency,
};

type CellKey = (
    PartitionId,
    [u8; 32],
    RelationalDurableMemberLocator,
    RelationalDurableMemberLocator,
);

pub(crate) struct ReadmittedRegionDirectory {
    pub(crate) schemas: BTreeMap<[u8; 32], Arc<RelationalBranchRootSchemaAuthority>>,
    directory: Arc<RelationalCheckpointDirectory>,
    symbols: crate::runtime::RuntimeSymbolTable,
    cells: BTreeMap<CellKey, Arc<RelationalRegionResidency>>,
}

impl ReadmittedRegionDirectory {
    pub(crate) fn new(
        directory: Arc<RelationalCheckpointDirectory>,
        symbols: crate::runtime::RuntimeSymbolTable,
    ) -> Result<Self, DurabilityError> {
        let mut schemas = BTreeMap::new();
        for (index, image) in directory.manifest.schemas.iter().enumerate() {
            let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
            let expected = registry.authority_snapshot();
            let authority = RelationalBranchRootSchemaAuthority::readmit_exact(
                index as u64 + 1,
                registry,
                contracts,
                &expected,
                image.descriptor_semantics_version(),
            )
            .ok_or_else(|| corrupt("schema carrier executable readmission failed"))?;
            if schemas.insert(image.carrier_digest(), authority).is_some() {
                return Err(corrupt("duplicate schema carrier"));
            }
        }
        Ok(Self {
            schemas,
            directory,
            // Cells retain an isolated readmission dictionary, never a live
            // runtime's writable symbol authority. Serving leases readmit into
            // the demanding runtime separately, including after a fork.
            symbols: symbols.detached_owner_snapshot(),
            cells: BTreeMap::new(),
        })
    }

    pub(crate) fn regions(
        &mut self,
        schema_digest: [u8; 32],
        leaves: &[RelationalCheckpointRegion],
    ) -> Result<Vec<RelationalRootRegionCheckpoint>, RelationalRecoveryStop> {
        let schema = self
            .schemas
            .get(&schema_digest)
            .cloned()
            .ok_or_else(|| corrupt("region directory names a missing schema carrier"))?;
        let authority = Arc::new(RelationalRegionKindAuthority::default());
        let mut result = Vec::new();
        for leaf in leaves {
            let key = (leaf.partition_id, schema_digest, leaf.head, leaf.history);
            let cell = match self.cells.entry(key) {
                std::collections::btree_map::Entry::Occupied(entry) => entry.get().clone(),
                std::collections::btree_map::Entry::Vacant(entry) => {
                    let cell = RelationalRegionResidency::cold(RelationalColdRegion {
                        kind_authority: leaf.kind_authority,
                        source: self.directory.source.clone(),
                        partition_id: leaf.partition_id,
                        head: leaf.head,
                        history: leaf.history,
                        history_members: self.directory.history_members.clone(),
                        schema: schema.clone(),
                        schema_carrier_digest: schema_digest,
                        symbols: self.symbols.clone(),
                        content_digest: leaf.content_digest,
                        entity_slots: leaf.entity_slots,
                        relation_slots: leaf.relation_slots,
                        unavailable_records: leaf.unavailable_records,
                    })?;
                    entry.insert(cell).clone()
                }
            };
            result.push(RelationalRootRegionCheckpoint {
                partition_id: leaf.partition_id,
                content_digest: leaf.content_digest,
                kind_commitment: leaf.kind_authority,
                entity_slots: leaf.entity_slots,
                relation_slots: leaf.relation_slots,
                unavailable_records: leaf.unavailable_records,
                cell,
                kind_authority: Some(authority.clone()),
            });
        }
        authority
            .bind(
                result
                    .iter()
                    .map(|region| (region.partition_id, region.cell.clone())),
            )
            .map_err(|cause| corrupt(format!("region kind authority: {cause:?}")))?;
        Ok(result)
    }
}

pub(crate) fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
