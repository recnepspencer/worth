//! Native capture selects cells and leaf facts, never reconstructive images.
use std::collections::BTreeMap;

use super::checkpoint_capture::CapturedCheckpointBasis;
use crate::durability::data::*;
use crate::durability::log::native_checkpoint_codec::CapturedManifest;

impl CapturedCheckpointBasis {
    pub(super) fn build_native_capture(
        mut self,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        source: Option<std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    ) -> Result<CapturedManifest, RelationalNativeCheckpointCaptureDenial> {
        let roots = std::mem::take(&mut self.branch_roots);
        let mirror = std::mem::take(&mut self.partitions);
        let mirror_schema = self.mirror_schema_authority.clone();
        let mut symbols = crate::symbols::data::StringInterner::default();
        symbols.restore_snapshot(self.symbol_table.clone());
        // With storage sources removed, this constructs metadata only. The
        // memory checkpoint door alone materializes its selected substrate.
        let metadata = self.build_checkpoint_image()?;
        let mut captured = CapturedManifest::capture_metadata(&metadata, policy)?;
        captured.cold_source = source;
        let mut schemas = BTreeMap::new();
        for root in roots {
            policy.check_live()?;
            let schema = root.schema_authority();
            let image = DurableBranchRootSchemaImage::capture(schema)
                .map_err(|error| corrupt(error.to_string()))?;
            let schema_digest = image.carrier_digest();
            schemas.entry(schema_digest).or_insert(image);
            let mut leaves = Vec::new();
            for region in root.regions() {
                let capture = region
                    .cell
                    .capture_checkpoint(schema, schema_digest, &symbols)
                    .map_err(storage_failure)?;
                if capture.leaf.partition_id != region.partition_id
                    || capture.leaf.content_digest != region.content_digest
                    || capture.leaf.entity_slots != region.entity_slots
                    || capture.leaf.relation_slots != region.relation_slots
                    || capture.leaf.unavailable_records != region.unavailable_records
                {
                    return Err(
                        corrupt("captured region differs from its selected root leaf").into(),
                    );
                }
                leaves.push(capture.leaf);
                captured.publications.push(capture.publication);
                for member in capture.members {
                    insert_member(&mut captured, member);
                }
            }
            captured.manifest.roots.push(RelationalCheckpointRoot {
                commit_id: root.commit_id(),
                schema_carrier_digest: schema_digest,
                regions: leaves,
            });
        }
        if let Some(schema) = mirror_schema {
            let image = DurableBranchRootSchemaImage::capture(&schema)
                .map_err(|error| corrupt(error.to_string()))?;
            let digest = image.carrier_digest();
            schemas.entry(digest).or_insert(image);
            for (_, region) in mirror.cells() {
                policy.check_live()?;
                let capture = region
                    .cell()
                    .capture_checkpoint(&schema, digest, &symbols)
                    .map_err(storage_failure)?;
                captured.manifest.mirror.push(capture.leaf);
                captured.publications.push(capture.publication);
                for member in capture.members {
                    insert_member(&mut captured, member);
                }
            }
        } else if mirror.len() != 0 {
            return Err(corrupt("nonempty mirror has no selected schema authority").into());
        }
        captured.manifest.schemas = schemas.into_values().collect();
        Ok(captured)
    }
}

fn insert_member(captured: &mut CapturedManifest, member: RelationalCheckpointMember) {
    let key = member.locator();
    if matches!(member, RelationalCheckpointMember::Carried { .. }) {
        captured.members.insert(key, member);
    } else {
        captured.members.entry(key).or_insert(member);
    }
}

fn storage_failure(failure: crate::storage::data::RelationalStorageUnavailable) -> DurabilityError {
    let RelationalRecoveryStop::Durability(error) = RelationalRecoveryStop::from(failure) else {
        unreachable!("storage unavailability is a durable-store refusal")
    };
    error
}

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
