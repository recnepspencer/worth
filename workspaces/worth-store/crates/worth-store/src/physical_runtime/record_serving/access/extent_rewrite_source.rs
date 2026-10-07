use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurableExtentManifest, DurableExtentRecordPlacement,
    RecordArtifactFile,
};

use super::extent_read_session::ExtentReadState;
use super::record_chunk_view::RecordReadIdentity;
use crate::physical_runtime::record_serving::residency::record_frame_reader::RecordFrameReader;
use crate::physical_runtime::record_serving::residency::PhysicalResidencyWorkPort;
use crate::physical_runtime::record_serving::work_semantics::integrity_admission::admit_extent_manifest;
use crate::physical_runtime::record_serving::{
    AdmittedPhysicalRecordFormat, PhysicalRecordId, RecordReadObservation,
};
use crate::physical_runtime::LifecycleGeneration;

/// The exact payload and manifest of one current extent generation.
pub(in crate::physical_runtime::record_serving) struct ExtentRewriteSource {
    pub(in crate::physical_runtime::record_serving) manifest: DurableExtentManifest,
    pub(in crate::physical_runtime::record_serving) payload: Vec<u8>,
    /// Extent file plus manifest file bytes the source generation occupies.
    pub(in crate::physical_runtime::record_serving) artifact_bytes: u64,
}

pub(in crate::physical_runtime::record_serving) struct ExtentRewriteSourceRequest<'request> {
    pub(in crate::physical_runtime::record_serving) residency: PhysicalResidencyWorkPort,
    pub(in crate::physical_runtime::record_serving) store: StableStoreIdentity,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) generation: LifecycleGeneration,
    pub(in crate::physical_runtime::record_serving) allocation:
        &'request worth_store_buffer_pool::OperationAllocationGrant,
}

/// Loads the source through the same C.9 manifest and chunk admission an
/// ordinary read uses. Any rejection leaves no rewrite effect behind.
pub(in crate::physical_runtime::record_serving) fn load_extent_rewrite_source(
    request: ExtentRewriteSourceRequest<'_>,
    placement: DurableExtentRecordPlacement,
) -> Result<ExtentRewriteSource, ()> {
    let allocation = request.allocation;
    let mut cursor = open_extent_rewrite_source(request, placement)?;
    let manifest = cursor.manifest();
    let mut payload = Vec::with_capacity(manifest.logical_bytes() as usize);
    while let Some(chunk) = cursor.next_chunk(allocation).map_err(|_| ())? {
        payload.extend_from_slice(chunk);
    }
    Ok(ExtentRewriteSource {
        manifest,
        payload,
        artifact_bytes: placement.arena_range().length(),
    })
}

/// One admitted source frame at a time. The caller owns the C10 root lease and
/// allocation grant for the complete traversal; no payload-sized buffer is held.
pub(in crate::physical_runtime::record_serving) struct ExtentRewriteCursor {
    manifest: DurableExtentManifest,
    state: ExtentReadState,
    identity: RecordReadIdentity,
    observation: RecordReadObservation,
}

impl ExtentRewriteCursor {
    pub(in crate::physical_runtime::record_serving) fn manifest(&self) -> DurableExtentManifest {
        self.manifest
    }

    pub(in crate::physical_runtime::record_serving) fn next_chunk(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<Option<&[u8]>, crate::physical_runtime::record_serving::RecordStreamFailure> {
        self.state
            .next_chunk(allocation, &mut self.observation, self.identity)
            .map_err(|failure| failure.into_stream_failure())
            .map(|chunk| chunk.map(|chunk| chunk.bytes))
    }
}

pub(in crate::physical_runtime::record_serving) fn open_extent_rewrite_source(
    request: ExtentRewriteSourceRequest<'_>,
    placement: DurableExtentRecordPlacement,
) -> Result<ExtentRewriteCursor, ()> {
    let reader = RecordFrameReader::serving(request.residency.clone());
    let manifest_frame = reader
        .load_exact(
            request.allocation,
            RecordArtifactFile::ExtentArena {
                arena: placement.arena_range().arena().get(),
            },
            placement.arena_range().offset(),
            104,
            super::super::residency::frame_loading::ExactFrameSourceExtent::ArenaRange(
                placement.arena_range(),
            ),
        )
        .map_err(|_| ())?;
    let context = reader.resident_admission_context().ok_or(())?;
    let admitted = admit_extent_manifest(
        &manifest_frame,
        context,
        request.store,
        request.format.declaration(),
        placement,
    )
    .map_err(|_| ())?;
    drop(manifest_frame);
    let manifest = admitted.manifest;
    if manifest.record() != placement.record()
        || manifest.logical_bytes() != placement.payload_bytes()
    {
        return Err(());
    }
    let artifact = RecordArtifactFile::ExtentArena {
        arena: placement.arena_range().arena().get(),
    };
    let identity = RecordReadIdentity::for_extent(
        request.store,
        request.generation,
        PhysicalRecordId::from_persisted(placement.record()),
        placement.extent_cell(),
    );
    let state = ExtentReadState::new(
        RecordFrameReader::serving(request.residency),
        artifact,
        manifest,
        placement.arena_range(),
        admitted.membership,
        request.store,
        request.format.declaration(),
    );
    Ok(ExtentRewriteCursor {
        manifest,
        state,
        identity,
        observation: RecordReadObservation::default(),
    })
}
