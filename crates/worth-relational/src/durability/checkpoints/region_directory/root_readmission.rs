//! One selected directory-root binder, reused by heads, components and lazy index verification.
use super::*;
use crate::branch::RelationalBranchRoot;
use crate::durability::checkpoint_admission::CheckpointAdmissionOwner;
use crate::durability::data::RelationalCheckpointRoot;
use crate::runtime::HistorySubsystem;

impl ReadmittedRegionDirectory {
    pub(crate) fn readmit_root(
        &mut self,
        history: &HistorySubsystem,
        image: &RelationalCheckpointRoot,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalRecoveryStop> {
        let artifact = history
            .commit_artifact(image.commit_id)
            .ok_or_else(|| corrupt("directory root has no actual catalog source"))?;
        let (entry, source) = artifact
            .envelope_residency()
            .source_placement()
            .ok_or_else(|| corrupt("directory root has no actual source placement"))?;
        if !Arc::ptr_eq(&source, &self.directory.source)
            || entry.commit.commit_id != image.commit_id
        {
            return Err(corrupt("directory root catalog source mismatch").into());
        }
        let schema = self
            .schemas
            .get(&image.schema_carrier_digest)
            .cloned()
            .ok_or_else(|| corrupt("directory root has no bound schema"))?;
        let leaves = self.regions(image.schema_carrier_digest, &image.regions, request)?;
        let content = crate::durability::checkpoints::content::read_records(
            request,
            &image.content,
            self.directory.source.as_ref(),
        )?;
        let root = if self.directory.admission_image.is_some() {
            let commit =
                CheckpointAdmissionOwner::read_existing_commit(&self.directory, &entry, request)
                    .map_err(RelationalRecoveryStop::AdmissionImage)?
                    .ok_or_else(|| {
                        corrupt("directory root has no original admitted checkpoint row")
                    })?;
            if !commit.links_artifact(&artifact) {
                return Err(corrupt(
                    "directory root checkpoint row mismatches actual catalog artifact",
                )
                .into());
            }
            history.readmit_checkpoint_region_directory(leaves, commit, &content, request)
        } else {
            let original = history.original_root_evidence_for_request(request, image.commit_id)?;
            history.readmit_branch_region_directory(
                request,
                &content,
                leaves,
                original,
                entry.root.clone(),
                schema,
            )
        };
        root.map_err(root_stop)
    }
}
fn root_stop(cause: crate::branch::RelationalBranchRootCaptureDenial) -> RelationalRecoveryStop {
    use crate::branch::RelationalBranchRootCaptureDenial as Cause;
    match cause {
        Cause::Request(cause) | Cause::Execution(cause) => RelationalRecoveryStop::Request(cause),
        Cause::Content(cause) => cause.into(),
        Cause::StorageUnavailable(cause) => cause.into(),
        cause => corrupt(format!("selected root readmission: {cause:?}")).into(),
    }
}
