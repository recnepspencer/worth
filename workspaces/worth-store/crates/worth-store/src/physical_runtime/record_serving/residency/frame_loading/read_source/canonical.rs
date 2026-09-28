use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{FrameReadSource, FrameReadSourceFailure, ObservedArtifactLength, PreparedFrameRead};
use crate::physical_runtime::{
    record_serving::{
        read_work_port::CanonicalRecordReadFailureEvidence,
        residency::frame_work_trace::FrameWorkTrace, CanonicalRecordReadFailure,
        CanonicalRecordReadPort, PreparedCanonicalRecordRead, RecordReadPartition,
    },
    PhysicalWorkIdentity,
};

#[derive(Clone)]
pub(in crate::physical_runtime::record_serving) struct CanonicalFrameReadSource {
    port: std::sync::Arc<CanonicalRecordReadPort>,
    context: CanonicalReadContext,
}

#[derive(Clone, Copy)]
enum CanonicalReadContext {
    Ordinary,
    Scan,
}

impl CanonicalFrameReadSource {
    pub(in crate::physical_runtime::record_serving) fn new(port: CanonicalRecordReadPort) -> Self {
        Self {
            port: std::sync::Arc::new(port),
            context: CanonicalReadContext::Ordinary,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn for_scan(mut self) -> Self {
        self.context = CanonicalReadContext::Scan;
        self
    }

    /// A publication candidate must be read from media after its sync. Pool
    /// hits cannot establish which bytes the new root would expose.
    pub(in crate::physical_runtime::record_serving) fn read_fresh_exact(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Result<
        (
            Box<[u8]>,
            crate::physical_runtime::instance::PhysicalProjectionFailureCapability,
        ),
        crate::physical_runtime::record_serving::read_work_port::CanonicalRecordReadFailureEvidence,
    > {
        let identity = self
            .port
            .prepare(coordinate, self.range_partition(coordinate.artifact()))?;
        let work = identity.identity();
        identity.execute().map_err(|failure| {
            crate::physical_runtime::record_serving::read_work_port::CanonicalRecordReadFailureEvidence::during_work(
                failure, work,
            )
        })
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving::residency::frame_loading) fn prepare_prefetch(
        &self,
        grant: &worth_store_buffer_pool::PrefetchResidencyGrant,
    ) -> Result<Box<dyn PreparedFrameRead + '_>, FrameReadSourceFailure> {
        let artifact = grant.frame().coordinate().artifact();
        bind_prepared(
            self.port
                .prepare_prefetch(grant, self.range_partition(artifact)),
        )
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving::residency::frame_loading) fn prepare_read_ahead(
        &self,
        grant: &worth_store_buffer_pool::ReadAheadFrameGrant<'_, '_>,
    ) -> Result<Box<dyn PreparedFrameRead + '_>, FrameReadSourceFailure> {
        let artifact = grant.frame().coordinate().artifact();
        bind_prepared(
            self.port
                .prepare_read_ahead(grant, self.range_partition(artifact)),
        )
    }

    fn range_partition(&self, artifact: RecordArtifactFile) -> RecordReadPartition {
        match self.context {
            CanonicalReadContext::Ordinary => RecordReadPartition::for_range(artifact),
            CanonicalReadContext::Scan => RecordReadPartition::Scan,
        }
    }

    fn metadata_partition(&self, artifact: RecordArtifactFile) -> RecordReadPartition {
        match self.context {
            CanonicalReadContext::Ordinary => RecordReadPartition::for_metadata(artifact),
            CanonicalReadContext::Scan => RecordReadPartition::Scan,
        }
    }
}

fn bind_prepared(
    prepared: Result<PreparedCanonicalRecordRead, CanonicalRecordReadFailureEvidence>,
) -> Result<Box<dyn PreparedFrameRead>, FrameReadSourceFailure> {
    prepared
        .map(|prepared| Box::new(prepared) as Box<dyn PreparedFrameRead>)
        .map_err(|failure| {
            FrameReadSourceFailure::work(failure.failure(), FrameWorkTrace::one(failure.identity()))
        })
}

impl FrameReadSource for CanonicalFrameReadSource {
    fn prepare_exact(
        &self,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
    ) -> Result<Box<dyn PreparedFrameRead + '_>, FrameReadSourceFailure> {
        let coordinate = RecordFrameCoordinate::new(artifact, offset, length).ok_or_else(|| {
            FrameReadSourceFailure::work(
                CanonicalRecordReadFailure::InvalidCoordinate,
                FrameWorkTrace::default(),
            )
        })?;
        bind_prepared(
            self.port
                .prepare(coordinate, self.range_partition(artifact)),
        )
    }

    fn file_length(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<ObservedArtifactLength, FrameReadSourceFailure> {
        self.port
            .file_length(artifact, self.metadata_partition(artifact))
            .map(|(bytes, work, projection_failure)| {
                ObservedArtifactLength::admitted(
                    bytes,
                    FrameWorkTrace::one(Some(work)),
                    projection_failure,
                )
            })
            .map_err(|failure| {
                FrameReadSourceFailure::work(
                    failure.failure(),
                    FrameWorkTrace::one(failure.identity()),
                )
            })
    }
}

impl PreparedFrameRead for PreparedCanonicalRecordRead {
    fn identity(&self) -> Option<PhysicalWorkIdentity> {
        Some(self.identity())
    }

    fn execute(
        self: Box<Self>,
        target: &mut [u8],
    ) -> Result<
        Option<crate::physical_runtime::instance::PhysicalProjectionFailureCapability>,
        FrameReadSourceFailure,
    > {
        let identity = (*self).identity();
        let (bytes, projection_failure) = (*self).execute().map_err(|failure| {
            FrameReadSourceFailure::work(failure, FrameWorkTrace::one(Some(identity)))
        })?;
        if bytes.len() != target.len() {
            projection_failure.consume();
            return Err(FrameReadSourceFailure::work(
                CanonicalRecordReadFailure::SettlementMismatch,
                FrameWorkTrace::one(Some(identity)),
            ));
        }
        target.copy_from_slice(&bytes);
        Ok(Some(projection_failure))
    }
}
