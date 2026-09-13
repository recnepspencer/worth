use std::ops::Range;
use worth_store_physical_format::{
    DurableExtentManifest, ExtentChunkCoordinate, PhysicalRecordFormatDeclaration,
    RecordArtifactFile, RecordFrameCoordinate, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_physical_integrity::IntegrityValidatedExtentMembership;

use super::super::{
    residency::frame_loading::LoadedPhysicalFrame, RecordReadObservation, RecordStreamFailure,
    RecordStreamFailureKind,
};
use super::record_chunk_view::RecordReadIdentity;
use crate::physical_runtime::record_serving::work_semantics::integrity_admission::{
    admit_extent_chunk, CleanExtentAdmissionDenial,
};

pub(in crate::physical_runtime::record_serving) struct ExtentReadChunk<'session> {
    pub(in crate::physical_runtime::record_serving) bytes: &'session [u8],
    pub(in crate::physical_runtime::record_serving) frame:
        worth_store_physical_format::RecordFrameCoordinate,
    pub(in crate::physical_runtime::record_serving) logical_range: Range<u64>,
}

#[derive(Clone, Copy)]
struct ExtentChunkReadPlan {
    completed: u64,
    payload_bytes: usize,
    frame_bytes: usize,
}

pub(in crate::physical_runtime::record_serving) struct ExtentReadState {
    artifacts: super::super::residency::record_frame_reader::RecordFrameReader<'static>,
    artifact: RecordArtifactFile,
    manifest: DurableExtentManifest,
    artifact_bytes: std::num::NonZeroU64,
    integrity_membership: IntegrityValidatedExtentMembership,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    next_ordinal: u32,
    logical_offset: u64,
    artifact_offset: u64,
    frame: Option<super::super::residency::frame_loading::LoadedPhysicalFrame>,
    payload: Range<usize>,
    payload_offset: usize,
}

impl ExtentReadState {
    pub(in crate::physical_runtime::record_serving) fn new(
        artifacts: super::super::residency::record_frame_reader::RecordFrameReader<'static>,
        artifact: RecordArtifactFile,
        manifest: DurableExtentManifest,
        artifact_bytes: std::num::NonZeroU64,
        integrity_membership: IntegrityValidatedExtentMembership,
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
        format: PhysicalRecordFormatDeclaration,
    ) -> Self {
        Self {
            artifacts,
            artifact,
            manifest,
            artifact_bytes,
            integrity_membership,
            store,
            format,
            next_ordinal: 1,
            logical_offset: 0,
            artifact_offset: 0,
            frame: None,
            payload: 0..0,
            payload_offset: 0,
        }
    }

    pub(super) fn read_next(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        target: &mut [u8],
        observation: &mut RecordReadObservation,
        identity: RecordReadIdentity,
    ) -> Result<usize, RecordStreamFailure> {
        if self.payload_offset == self.payload.len() {
            if self.logical_offset == self.manifest.logical_bytes() {
                return Ok(0);
            }
            self.load_chunk(allocation, observation, identity)?;
        }
        let count = target.len().min(self.payload.len() - self.payload_offset);
        let start = self.payload.start + self.payload_offset;
        let frame = self.frame.as_ref().expect("loaded extent frame is present");
        frame.copy_range_into(start..start + count, &mut target[..count]);
        self.payload_offset += count;
        Ok(count)
    }

    pub(super) fn next_chunk(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        observation: &mut RecordReadObservation,
        identity: RecordReadIdentity,
    ) -> Result<Option<ExtentReadChunk<'_>>, RecordStreamFailure> {
        if self.payload_offset == self.payload.len() {
            if self.logical_offset == self.manifest.logical_bytes() {
                return Ok(None);
            }
            self.load_chunk(allocation, observation, identity)?;
        }

        let logical_start = self.delivered_bytes();
        let start = self.payload.start + self.payload_offset;
        let end = self.payload.end;
        self.payload_offset = self.payload.len();
        let logical_end = self.delivered_bytes();
        let frame = self.frame.as_ref().expect("loaded extent frame is present");
        Ok(Some(ExtentReadChunk {
            bytes: &frame[start..end],
            frame: frame.coordinate(),
            logical_range: logical_start..logical_end,
        }))
    }

    fn load_chunk(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        observation: &mut RecordReadObservation,
        identity: RecordReadIdentity,
    ) -> Result<(), RecordStreamFailure> {
        let plan = self.plan_chunk_read();
        self.frame = None;
        let frame = self.load_planned_chunk(allocation, plan, observation, identity)?;
        let (frame, payload) = self.admit_loaded_chunk(frame, plan, observation)?;
        self.install_chunk(frame, payload, plan)
    }

    fn plan_chunk_read(&self) -> ExtentChunkReadPlan {
        let payload_bytes = (self.manifest.logical_bytes() - self.logical_offset)
            .min(u64::from(self.manifest.chunk_payload_capacity()))
            as usize;
        let frame_bytes =
            DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES + payload_bytes;
        ExtentChunkReadPlan {
            completed: self.delivered_bytes(),
            payload_bytes,
            frame_bytes,
        }
    }

    fn load_planned_chunk(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        plan: ExtentChunkReadPlan,
        observation: &mut RecordReadObservation,
        identity: RecordReadIdentity,
    ) -> Result<LoadedPhysicalFrame, RecordStreamFailure> {
        let coordinate = RecordFrameCoordinate::new(
            self.artifact,
            self.artifact_offset,
            plan.frame_bytes as u32,
        )
        .ok_or_else(|| {
            RecordStreamFailure::during_read(
                RecordStreamFailureKind::ArtifactDamaged,
                plan.completed,
            )
        })?;
        let frame = self
            .artifacts
            .load_exact(
                allocation,
                self.artifact,
                self.artifact_offset,
                plan.frame_bytes as u32,
                super::super::residency::frame_loading::ExactFrameSourceExtent::CompleteArtifact(
                    self.artifact_bytes,
                ),
            )
            .map_err(|failure| {
                observation.observe_physical_work(failure.work_trace());
                frame_load_stream_failure(identity, coordinate, failure, plan.completed)
            })?;
        observation.observe_physical_work(frame.work_trace());
        observation.observe_transfer(frame.len());
        Ok(frame)
    }

    fn admit_loaded_chunk(
        &self,
        frame: LoadedPhysicalFrame,
        plan: ExtentChunkReadPlan,
        observation: &mut RecordReadObservation,
    ) -> Result<(LoadedPhysicalFrame, Range<usize>), RecordStreamFailure> {
        let coordinate = match ExtentChunkCoordinate::new(
            self.manifest.record(),
            self.manifest.extent_cell(),
            self.manifest.logical_bytes(),
            self.logical_offset,
            self.next_ordinal,
        ) {
            Some(coordinate) => coordinate,
            None => {
                frame.reject_projection_failure();
                return Err(RecordStreamFailure::during_read(
                    RecordStreamFailureKind::ArtifactDamaged,
                    plan.completed,
                ));
            }
        };
        let context = self.artifacts.resident_admission_context().ok_or_else(|| {
            RecordStreamFailure::during_read(
                RecordStreamFailureKind::ArtifactDamaged,
                plan.completed,
            )
        })?;
        let admitted = admit_extent_chunk(
            &frame,
            context,
            self.store,
            self.format,
            coordinate,
            self.integrity_membership,
        );
        let admitted = match admitted {
            Ok(admitted) => admitted,
            Err(denial) => {
                let stale = denial == CleanExtentAdmissionDenial::ExtentMembership;
                if stale {
                    observation.check_generation(false);
                }
                if !denial.preserves_resident_bytes() {
                    frame.reject_projection_failure();
                }
                return Err(RecordStreamFailure::during_read(
                    denial.stream_failure_kind(),
                    plan.completed,
                ));
            }
        };
        observation.check_generation(true);
        if admitted.payload.len() != plan.payload_bytes {
            frame.reject_projection_failure();
            return Err(RecordStreamFailure::during_read(
                RecordStreamFailureKind::FormatMismatch,
                plan.completed,
            ));
        }
        Ok((frame, admitted.payload))
    }

    fn install_chunk(
        &mut self,
        frame: LoadedPhysicalFrame,
        payload: Range<usize>,
        plan: ExtentChunkReadPlan,
    ) -> Result<(), RecordStreamFailure> {
        let next_logical_offset = self.logical_offset + plan.payload_bytes as u64;
        let next_ordinal = if next_logical_offset < self.manifest.logical_bytes() {
            let Some(next_ordinal) = self.next_ordinal.checked_add(1) else {
                frame.reject_projection_failure();
                return Err(RecordStreamFailure::during_read(
                    RecordStreamFailureKind::ArtifactDamaged,
                    plan.completed,
                ));
            };
            next_ordinal
        } else {
            self.next_ordinal
        };
        self.payload = payload;
        self.payload_offset = 0;
        self.frame = Some(frame);
        self.artifact_offset += plan.frame_bytes as u64;
        self.logical_offset = next_logical_offset;
        self.next_ordinal = next_ordinal;
        Ok(())
    }

    fn delivered_bytes(&self) -> u64 {
        self.logical_offset
            .saturating_sub(self.payload.len() as u64)
            .saturating_add(self.payload_offset as u64)
    }
}

fn frame_load_stream_failure(
    identity: RecordReadIdentity,
    coordinate: RecordFrameCoordinate,
    failure: super::super::residency::frame_loading::FrameLoadFailure,
    completed: u64,
) -> RecordStreamFailure {
    let denial = super::locate::failure_classification::read_failure(failure);
    if let super::super::RecordReadDenial::ResidencyUnavailable(residency) = denial {
        if let Some(pressure) = identity.pressure_evidence(residency, coordinate) {
            return RecordStreamFailure::during_read_pressure(pressure, completed);
        }
    }
    let kind = match denial {
        super::super::RecordReadDenial::FormatMismatch => RecordStreamFailureKind::FormatMismatch,
        super::super::RecordReadDenial::StalePlacement(_) => {
            RecordStreamFailureKind::StalePlacement
        }
        super::super::RecordReadDenial::ArtifactUnavailable => {
            RecordStreamFailureKind::ArtifactUnavailable
        }
        super::super::RecordReadDenial::ArtifactDamaged => RecordStreamFailureKind::ArtifactDamaged,
        super::super::RecordReadDenial::PhysicalWork(
            super::super::RecordReadWorkDenial::RuntimeReleased,
        ) => RecordStreamFailureKind::RuntimeReleased,
        super::super::RecordReadDenial::ResidencyUnavailable(residency) => {
            RecordStreamFailureKind::ResidencyUnavailable(residency)
        }
        _ => RecordStreamFailureKind::Backend,
    };
    RecordStreamFailure::during_read(kind, completed)
}
