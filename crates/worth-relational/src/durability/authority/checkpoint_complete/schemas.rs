//! Canonical images of the exact retained original schema DATA union.
use super::source::{FrozenOriginalInputs, OriginalCanonicalCommit};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::data::{
    CheckpointSchemaImages, DurableBranchRootSchemaImage,
    RelationalNativeCheckpointCaptureDenial as Denial,
};
use std::{marker::PhantomData, sync::Arc};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) struct CompleteSchemas<'a> {
    original: ExecutionOrderedMap<usize, [u8; 32]>,
    images: CheckpointSchemaImages,
    // Pointer lookup is valid only while the actual immutable input owners
    // remain borrowed. An unrelated later allocation cannot reuse an address.
    _sources: PhantomData<&'a ()>,
}
impl<'a> CompleteSchemas<'a> {
    pub(super) fn capture_interim(
        frames: &'a [OriginalCanonicalCommit],
        components: impl Iterator<
            Item = (
                &'a crate::branch::RelationalBranchBasisDescriptor,
                &'a Arc<crate::branch::RelationalBranchRoot>,
            ),
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut identities = ExecutionOrderedMap::default();
        let mut images = ExecutionOrderedMap::default();
        for frame in frames {
            record(
                &frame.inputs.prestate_schema,
                &mut identities,
                &mut images,
                request,
            )?;
            record(
                &frame.inputs.result_schema,
                &mut identities,
                &mut images,
                request,
            )?;
        }
        for (_, root) in components {
            record(
                &root.retained_schema_authority(),
                &mut identities,
                &mut images,
                request,
            )?;
        }
        Ok(Self {
            original: identities,
            images: CheckpointSchemaImages::copy_for_request(&images, request)
                .map_err(Denial::Request)?,
            _sources: PhantomData,
        })
    }
    pub(super) fn capture(
        original: &'a FrozenOriginalInputs,
        _commits: &'a [OriginalCanonicalCommit],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut identities = ExecutionOrderedMap::default();
        let mut images = ExecutionOrderedMap::default();
        let birth = original
            .branches
            .birth
            .as_ref()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        record(&birth.schema, &mut identities, &mut images, request)?;
        if let Some(installation) = &original.branches.installation {
            record(
                &installation.before.schema,
                &mut identities,
                &mut images,
                request,
            )?;
            record(
                &installation.after.schema,
                &mut identities,
                &mut images,
                request,
            )?;
        }
        for transition in original.branches.transitions() {
            record(&transition.schema, &mut identities, &mut images, request)?;
        }
        for artifact in original.artifacts.iter() {
            let inputs = artifact
                .original_inputs()
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            record(
                &inputs.prestate_schema,
                &mut identities,
                &mut images,
                request,
            )?;
            record(&inputs.result_schema, &mut identities, &mut images, request)?;
        }
        let images =
            CheckpointSchemaImages::copy_for_request(&images, request).map_err(Denial::Request)?;
        Ok(Self {
            original: identities,
            images,
            _sources: PhantomData,
        })
    }
    pub(super) fn carrier_for(
        &self,
        authority: &RelationalBranchRootSchemaAuthority,
    ) -> Option<[u8; 32]> {
        self.original
            .get(&(authority as *const RelationalBranchRootSchemaAuthority as usize))
            .copied()
    }
    pub(super) fn into_parts(
        self,
    ) -> (CheckpointSchemaImages, ExecutionOrderedMap<usize, [u8; 32]>) {
        (self.images, self.original)
    }
}
fn record(
    authority: &Arc<RelationalBranchRootSchemaAuthority>,
    identities: &mut ExecutionOrderedMap<usize, [u8; 32]>,
    images: &mut ExecutionOrderedMap<[u8; 32], DurableBranchRootSchemaImage>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    crate::execution::check_request(request).map_err(Denial::Request)?;
    #[expect(
        clippy::disallowed_methods,
        reason = "Fixed allocation identity deduplicates custody; canonical order never uses this address"
    )]
    let identity = Arc::as_ptr(authority) as usize;
    if identities.get(&identity).is_some() {
        return Ok(());
    }
    let image = authority
        .descriptive_image_for_request(request)
        .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
    let carrier = image.carrier_digest();
    if let Some(previous) = images.get(&carrier) {
        if previous != &image {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
    } else {
        images
            .insert_cloned_native_for_request(&carrier, &image, *request)
            .map_err(map_stop)?;
    }
    identities
        .insert_cloned_native_for_request(&identity, &carrier, *request)
        .map_err(map_stop)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(
        crate::durability::checkpoint_history::capture::map_stop(cause),
    )
}
