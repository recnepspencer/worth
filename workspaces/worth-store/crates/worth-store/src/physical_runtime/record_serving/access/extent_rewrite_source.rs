use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurableExtentManifest, DurableExtentRecordPlacement,
    RecordArtifactFile, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};

use super::extent_read_session::ExtentReadState;
use super::record_chunk_view::RecordReadIdentity;
use crate::physical_runtime::record_serving::residency::record_frame_reader::RecordFrameReader;
use crate::physical_runtime::record_serving::residency::PhysicalResidencyWorkPort;
use crate::physical_runtime::record_serving::work_semantics::integrity_admission::admit_extent_manifest;
use crate::physical_runtime::record_serving::{
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, PhysicalRecordId,
    RecordReadObservation,
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
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
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
    let reader = RecordFrameReader::serving(request.residency.clone());
    let manifest_frame = reader
        .load_bounded(
            request.allocation,
            RecordArtifactFile::ExtentManifest {
                extent: placement.extent().get(),
                generation: placement.extent_generation(),
            },
            request
                .access
                .transfer_limit()
                .get()
                .min(request.format.declaration().page_size().bytes()),
        )
        .map_err(|_| ())?;
    let manifest_bytes = manifest_frame.len() as u64;
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
    let extent_bytes = manifest.logical_bytes()
        + u64::from(manifest.chunk_count())
            * (DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u64;
    let artifact = RecordArtifactFile::Extent {
        extent: placement.extent().get(),
        generation: placement.extent_generation(),
    };
    let identity = RecordReadIdentity::for_extent(
        request.store,
        request.generation,
        PhysicalRecordId::from_persisted(placement.record()),
        placement.extent_cell(),
    );
    let mut state = ExtentReadState::new(
        RecordFrameReader::serving(request.residency),
        artifact,
        manifest,
        std::num::NonZeroU64::new(extent_bytes).ok_or(())?,
        admitted.membership,
        request.store,
        request.format.declaration(),
    );
    let length = usize::try_from(manifest.logical_bytes()).map_err(|_| ())?;
    let mut payload = vec![0; length];
    let mut observation = RecordReadObservation::default();
    let mut filled = 0;
    while filled < length {
        let count = state
            .read_next(
                request.allocation,
                &mut payload[filled..],
                &mut observation,
                identity,
            )
            .map_err(|_| ())?;
        if count == 0 {
            return Err(());
        }
        filled += count;
    }
    Ok(ExtentRewriteSource {
        manifest,
        payload,
        artifact_bytes: extent_bytes.saturating_add(manifest_bytes),
    })
}
