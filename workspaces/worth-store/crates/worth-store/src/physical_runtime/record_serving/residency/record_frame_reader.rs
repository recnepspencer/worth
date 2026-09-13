use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::RecordArtifactFile;

use super::{
    capability::PhysicalResidencyWorkPort,
    frame_loading::{
        DirectFrameReadSource, ExactFrameSourceExtent, FrameLoadFailure, LoadedPhysicalFrame,
    },
    frame_ports::FrameLoadPort,
};

/// Read-only access to record frames through either bootstrap media or the
/// serving runtime's canonical physical-work route.
pub(in crate::physical_runtime::record_serving) struct RecordFrameReader<'media> {
    route: RecordFrameReadRoute<'media>,
}

enum RecordFrameReadRoute<'media> {
    Bootstrap {
        media: &'media QualifiedFilesystemMedia,
        loader: &'media (dyn FrameLoadPort + Send + Sync),
    },
    Serving(PhysicalResidencyWorkPort),
}

impl<'media> RecordFrameReader<'media> {
    pub(in crate::physical_runtime::record_serving) fn resident_admission_context(
        &self,
    ) -> Option<crate::physical_runtime::integrity::ResidentAdmissionContext<'_>> {
        match &self.route {
            RecordFrameReadRoute::Serving(residency) => {
                Some(residency.resident_admission_context())
            }
            RecordFrameReadRoute::Bootstrap { .. } => None,
        }
    }

    pub(in crate::physical_runtime::record_serving) const fn bootstrap(
        media: &'media QualifiedFilesystemMedia,
        loader: &'media (dyn FrameLoadPort + Send + Sync),
    ) -> Self {
        Self {
            route: RecordFrameReadRoute::Bootstrap { media, loader },
        }
    }

    pub(in crate::physical_runtime::record_serving) const fn serving(
        residency: PhysicalResidencyWorkPort,
    ) -> RecordFrameReader<'static> {
        RecordFrameReader {
            route: RecordFrameReadRoute::Serving(residency),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn load_exact(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
        source_extent: ExactFrameSourceExtent,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        match &self.route {
            RecordFrameReadRoute::Bootstrap { media, loader } => loader.load_exact(
                allocation,
                &DirectFrameReadSource::new(media),
                artifact,
                offset,
                length,
                source_extent,
            ),
            RecordFrameReadRoute::Serving(residency) => residency.load_exact(
                allocation,
                worth_store_physical_format::RecordFrameCoordinate::new(artifact, offset, length)
                    .ok_or_else(|| {
                    super::frame_loading::FrameLoadFailure::new(
                        super::frame_loading::FrameLoadFailureKind::InvalidCoordinate,
                    )
                })?,
                source_extent,
            ),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn load_bounded(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        artifact: RecordArtifactFile,
        limit: u32,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        match &self.route {
            RecordFrameReadRoute::Bootstrap { media, loader } => loader.load_bounded(
                allocation,
                &DirectFrameReadSource::new(media),
                artifact,
                limit,
            ),
            RecordFrameReadRoute::Serving(residency) => {
                residency.load_bounded(allocation, artifact, limit)
            }
        }
    }
}
