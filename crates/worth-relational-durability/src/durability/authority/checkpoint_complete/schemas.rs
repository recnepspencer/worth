//! Canonical images of the exact retained original schema DATA union.
use worth_relational_contracts::internal_port as contracts;
use super::source::{FrozenOriginalInputs, OriginalCanonicalCommit};
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::durability::data::CheckpointSchemaImages;
use crate::durability::data::DurableBranchRootSchemaImage;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::schema_carrier::RecordedBranchSchemaImage;
use std::{marker::PhantomData, sync::Arc};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::CanonicalDigestId;

pub(super) struct CompleteSchemas<'a> {
    original: ExecutionOrderedMap<usize, CanonicalDigestId>,
    images: CheckpointSchemaImages,
    // Pointer lookup is valid only while the actual immutable input owners
    // remain borrowed. An unrelated later allocation cannot reuse an address.
    _sources: PhantomData<&'a ()>,
}
impl<'a> CompleteSchemas<'a> {
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
            record_image(&inputs.prestate_schema, &mut images, request)?;
            record_image(&inputs.result_schema, &mut images, request)?;
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
    ) -> Option<CanonicalDigestId> {
        self.original
            .get(&(authority as *const RelationalBranchRootSchemaAuthority as usize))
            .copied()
            .or_else(|| {
                // Commit roles retain the originally admitted image. The
                // actual root borrows that same authenticated declaration;
                // require its full image in the captured original union.
                let image = authority.retained_recorded_image()?;
                let digest = image.carrier_digest();
                let ordinal = self
                    .images
                    .binary_search_by_key(&digest, |row| row.carrier_digest())
                    .ok()?;
                (&*self.images[ordinal] == image).then_some(CanonicalDigestId::new(digest))
            })
    }
    pub(super) fn into_images(self) -> CheckpointSchemaImages {
        self.images
    }
}
fn record(
    authority: &Arc<RelationalBranchRootSchemaAuthority>,
    identities: &mut ExecutionOrderedMap<usize, CanonicalDigestId>,
    images: &mut ExecutionOrderedMap<CanonicalDigestId, DurableBranchRootSchemaImage>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    contracts::execution::request_custody::check_request(request).map_err(Denial::Request)?;
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
        .map_err(contracts::durability::data::native_checkpoint_capture_denial::capture_stop)?;
    let carrier = record_image(&image, images, request)?;
    identities
        .insert_cloned_native_for_request(&identity, &carrier, *request)
        .map_err(map_stop)
}

fn record_image(
    image: &RecordedBranchSchemaImage,
    images: &mut ExecutionOrderedMap<CanonicalDigestId, DurableBranchRootSchemaImage>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CanonicalDigestId, Denial> {
    contracts::execution::request_custody::check_request(request).map_err(Denial::Request)?;
    let carrier = CanonicalDigestId::new(image.carrier_digest());
    let image = DurableBranchRootSchemaImage::from(image.clone());
    if let Some(previous) = images.get(&carrier) {
        if previous != &image {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
    } else {
        images
            .insert_cloned_native_for_request(&carrier, &image, *request)
            .map_err(map_stop)?;
    }
    Ok(carrier)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(
        contracts::durability::data::recovery_stop::map_stop(cause),
    )
}
