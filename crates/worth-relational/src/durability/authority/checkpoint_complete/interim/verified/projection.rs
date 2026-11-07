//! Verify the emitted index against the exact captured originals and actual roots.
use super::*;
use crate::durability::{
    authority::checkpoint_complete::{admission_projection, roots::CapturedRoots},
    data::RelationalCheckpointManifest,
};
use worth_execution::ExecutionRequestArrayBuilder;
use worth_foundational::CanonicalDigestId;
use worth_proof::NativeCheckpointOriginalsVerified;

impl FrozenMemoryOriginals {
    pub(in crate::durability::authority::checkpoint_complete::interim) fn project(
        artifacts: ExecutionArray<Arc<RelationalCommitArtifact>>,
        current: crate::branch::CheckpointBranchMetadata,
        owner: &Weak<crate::runtime::RelationalRuntimeState>,
        manifest: &RelationalCheckpointManifest,
        originals: ReadmittedMemoryOriginals,
        roots: &CapturedRoots,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<NativeCheckpointOriginalsVerified<VerifiedMemoryProjection>, Denial> {
        let ReadmittedMemoryOriginals {
            frames,
            positioned: originals,
        } = originals;
        if frames.len() != manifest.commits.len() || originals.len() != frames.len() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let mut rows =
            ExecutionRequestArrayBuilder::allocate(*request, frames.len()).map_err(array_stop)?;
        let mut canonical = ExecutionOrderedMap::default();
        for ((entry, frame), original) in manifest
            .commits
            .iter()
            .zip(frames.iter())
            .zip(originals.iter())
        {
            if original.position() != frame.position
                || !Arc::ptr_eq(original.canonical_arc(), &frame.envelope)
                || entry.position != frame.position
                || entry.commit != frame.envelope.commit
                || &entry.root != frame.artifact.roots()
                || entry.payload_digest != frame.original_payload_digest
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let carrier = frame
                .inputs
                .result_schema
                .descriptive_image_for_request(request)
                .map_err(stop)?
                .carrier_digest();
            rows.push(admission_projection::row::encode_original(
                &frame.envelope,
                entry.position,
                CanonicalDigestId::new(entry.payload_digest),
                &entry.root,
                CanonicalDigestId::new(carrier),
                request,
            )?)
            .map_err(array_stop)?;
            let retained =
                RetainedCanonicalIndex::copy_original(entry, request).map_err(Denial::Request)?;
            canonical
                .insert_cloned_native_for_request(&entry.commit.commit_id, &retained, *request)
                .map_err(map_stop)?;
        }
        Ok(NativeCheckpointOriginalsVerified::from_owner(
            VerifiedMemoryProjection {
                current,
                originals,
                rows: rows.seal().map_err(array_stop)?,
                roots: admission_projection::root_directory::encode(manifest, roots, request)?,
                artifacts,
                owner: owner.clone(),
                canonical,
            },
        ))
    }
}
