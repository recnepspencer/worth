//! Lazy exact component images are read from this recovery's admitted directory.
use super::*;
use crate::branch::{
    RecordedRelationalComponentReference as Recorded, RecordedRelationalComponentTarget as Target,
};
use crate::durability::data::RelationalRecoveryStop;

impl RecoveredRootInventory {
    pub(super) fn install_checkpoint_headers(
        &mut self,
        runtime: &RelationalRuntime,
    ) -> Result<(), DurabilityError> {
        let Some(directory) = runtime.durability.checkpoint_selection().snapshot() else {
            return Ok(());
        };
        self.directory = Some(Arc::downgrade(&directory));
        for header in &directory.manifest.roots {
            let image = self
                .roots
                .entry(header.commit_id)
                .or_insert_with(|| RecoveredRootImage {
                    root: RecoveredRootHold::Checkpoint(Default::default()),
                    observations: Vec::new(),
                });
            image
                .observations
                .extend(header.recorded_components.iter().cloned());
        }
        let mut empty_observations = self
            .empty
            .iter()
            .flat_map(|image| &image.observations)
            .map(|descriptor| {
                let canonical = descriptor
                    .in_canonical_frame()
                    .map_err(|cause| corrupt(format!("Empty component frame: {cause:?}")))?;
                rmp_serde::to_vec_named(&canonical).map_err(|cause| corrupt(cause.to_string()))
            })
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        for header in &directory.manifest.retained_empty_components {
            let key = rmp_serde::to_vec_named(&header.descriptor)
                .map_err(|cause| corrupt(cause.to_string()))?;
            if empty_observations.insert(key) {
                self.empty.push(RecoveredRootImage {
                    root: RecoveredRootHold::Checkpoint(Default::default()),
                    observations: vec![header.descriptor.clone()],
                });
            }
        }
        Ok(())
    }

    pub(super) fn materialize_checkpoint_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        recorded: &Recorded,
        image: &RecoveredRootImage,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalRecoveryStop> {
        let directory = self
            .directory
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or_else(|| corrupt("recorded checkpoint directory unavailable"))?;
        let mut regions =
            crate::durability::checkpoints::region_directory::ReadmittedRegionDirectory::new(
                directory.clone(),
                runtime.services.symbols.clone(),
            )?;
        let root = match recorded.target() {
            Target::Empty => {
                let header = directory
                    .manifest
                    .retained_empty_components
                    .iter()
                    .find(|header| &header.descriptor == recorded.descriptor())
                    .ok_or_else(|| corrupt("recorded Empty header unavailable"))?;
                let schema = regions
                    .schemas
                    .get(&header.schema_carrier_digest)
                    .ok_or_else(|| corrupt("recorded Empty schema unavailable"))?;
                RelationalBranchRoot::empty_with_schema_authority(schema.clone())
            }
            Target::Canonical(target) => {
                let header = directory
                    .manifest
                    .roots
                    .iter()
                    .find(|header| {
                        header.commit_id == target.receipt().commit_id
                            && header.recorded_components.contains(recorded.descriptor())
                    })
                    .ok_or_else(|| corrupt("recorded canonical header unavailable"))?;
                let artifact = runtime
                    .history
                    .commit_artifact(header.commit_id)
                    .ok_or_else(|| corrupt("recorded component has no actual catalog source"))?;
                let (entry, source) = artifact
                    .envelope_residency()
                    .source_placement()
                    .ok_or_else(|| corrupt("recorded component has no durable source"))?;
                if !Arc::ptr_eq(&source, &directory.source)
                    || entry.commit != *target.receipt()
                    || entry.position != target.position()
                    || &entry.payload_digest != target.artifact_digest()
                    || &entry.root != target.roots()
                {
                    return Err(corrupt("recorded component source mismatch").into());
                }
                let positioned = artifact
                    .envelope_residency()
                    .demand_positioned(execution, 1 << 30)?;
                let schema = regions
                    .schemas
                    .get(&header.schema_carrier_digest)
                    .cloned()
                    .ok_or_else(|| corrupt("recorded canonical schema unavailable"))?;
                let leaves = regions.regions(header.schema_carrier_digest, &header.regions)?;
                runtime
                    .history
                    .readmit_branch_region_directory(
                        leaves,
                        positioned.canonical_arc().clone(),
                        entry.root.clone(),
                        schema,
                    )
                    .map_err(|cause| corrupt(format!("recorded root readmission: {cause:?}")))?
            }
        };
        if let RecoveredRootHold::Checkpoint(materialized) = &image.root {
            let mut retained = materialized
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(existing) = retained.upgrade() {
                return Ok(existing);
            }
            *retained = Arc::downgrade(&root);
        }
        Ok(root)
    }
}

/// Only the owner rebinds an exact admitted header after verifying its root.
pub(super) fn bind_recorded_descriptor(
    descriptor: &RelationalBranchBasisDescriptor,
    runtime: &RelationalRuntime,
    root: &Arc<RelationalBranchRoot>,
) -> Result<RelationalBranchBasisDescriptor, RelationalRecoveryStop> {
    descriptor
        .bind_readmitted_root(runtime.runtime_instance_id(), root)
        .map_err(|cause| corrupt(format!("recorded component binding: {cause:?}")).into())
}
