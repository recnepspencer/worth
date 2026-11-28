//! Lazy exact component images are read from this recovery's admitted directory.
use super::*;
use worth_relational_branch::internal_port::branch::recorded_basis::{
    RecordedRelationalComponentReference as Recorded, RecordedRelationalComponentTarget as Target,
};
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;

impl RecoveredRootInventory {
    pub(super) fn install_checkpoint_headers(
        &mut self,
        checkpoint: &RelationalCheckpointManifestSelection,
    ) -> Result<(), DurabilityError> {
        let Some(directory) = checkpoint.snapshot() else {
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
        history: &HistorySubsystem,
        schema: &RelationalSchemaRegistry,
        symbols: &RuntimeSymbolTable,
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
                symbols.clone(),
                execution,
                schema,
            )?
            .with_working_set(self.historical_pool());
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
                let entry = durable_source_entry(history, &directory, header.commit_id)?;
                if entry.commit != *target.receipt()
                    || entry.position != target.position()
                    || &entry.payload_digest != target.artifact_digest()
                    || &entry.root != target.roots()
                {
                    return Err(corrupt("recorded component source mismatch").into());
                }
                regions.readmit_root(history, header, execution)?
            }
        };
        Ok(retain_materialized(image, root))
    }

    /// Rebuild the exact checkpoint root image of `commit_id`, whichever
    /// retained component recorded it.
    pub(super) fn materialize_commit_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &HistorySubsystem,
        schema: &RelationalSchemaRegistry,
        symbols: &RuntimeSymbolTable,
        commit_id: CommitId,
        image: &RecoveredRootImage,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalRecoveryStop> {
        let directory = self
            .directory
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or_else(|| corrupt("recorded checkpoint directory unavailable"))?;
        let header = directory
            .manifest
            .roots
            .iter()
            .find(|header| header.commit_id == commit_id)
            .ok_or_else(|| corrupt("retained fork source header unavailable"))?;
        let entry = durable_source_entry(history, &directory, commit_id)?;
        if entry.commit.commit_id != commit_id {
            return Err(corrupt("retained fork source mismatch").into());
        }
        let mut regions =
            crate::durability::checkpoints::region_directory::ReadmittedRegionDirectory::new(
                directory.clone(),
                symbols.clone(),
                execution,
                schema,
            )?
            .with_working_set(self.historical_pool());
        let root = regions.readmit_root(history, header, execution)?;
        Ok(retain_materialized(image, root))
    }
}

/// The commit's durable journal entry, which must come from this directory.
fn durable_source_entry(
    history: &HistorySubsystem,
    directory: &Arc<crate::durability::data::RelationalCheckpointDirectory>,
    commit_id: CommitId,
) -> Result<
    worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry,
    RelationalRecoveryStop,
>{
    let artifact = history
        .try_commit_artifact(commit_id)
        .map_err(|cause| crate::durability::data::commit_index_access::stop(commit_id.0, cause))?
        .ok_or_else(|| corrupt("recorded component has no actual catalog source"))?;
    let (entry, source) = artifact
        .envelope_residency()
        .source_placement()
        .ok_or_else(|| corrupt("recorded component has no durable source"))?;
    if !Arc::ptr_eq(&source, &directory.source) {
        return Err(corrupt("recorded component source mismatch").into());
    }
    Ok(entry.as_ref().clone())
}

/// One materialized image row keeps a weak share; a concurrent winner is reused.
fn retain_materialized(
    image: &RecoveredRootImage,
    root: Arc<RelationalBranchRoot>,
) -> Arc<RelationalBranchRoot> {
    if let RecoveredRootHold::Checkpoint(materialized) = &image.root {
        let mut retained = materialized
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(existing) = retained.upgrade() {
            return existing;
        }
        *retained = Arc::downgrade(&root);
    }
    root
}

/// Only the owner rebinds an exact admitted header after verifying its root.
pub(super) fn bind_recorded_descriptor(
    descriptor: &RelationalBranchBasisDescriptor,
    runtime_id: u64,
    root: &Arc<RelationalBranchRoot>,
) -> Result<RelationalBranchBasisDescriptor, RelationalRecoveryStop> {
    descriptor
        .bind_readmitted_root(runtime_id, root)
        .map_err(|cause| corrupt(format!("recorded component binding: {cause:?}")).into())
}

impl RecoveredRootInventory {
    fn historical_pool(
        &self,
    ) -> Option<crate::durability::checkpoints::region_directory::RegionWorkingSet> {
        self.historical_working_set.clone().map(|accountant| {
            (
                accountant,
                worth_relational_storage::internal_port::storage::residency::working_set::ResidencyPoolKind::Historical,
            )
        })
    }
}
