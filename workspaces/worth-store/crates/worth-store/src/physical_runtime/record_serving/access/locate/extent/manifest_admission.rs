use super::super::failure_classification::read_failure;
use super::super::PhysicalRecordReader;
use crate::physical_runtime::record_serving::work_semantics::integrity_admission::{
    admit_extent_manifest as admit_clean_extent_manifest, CleanExtentAdmissionDenial,
};
use crate::physical_runtime::record_serving::{
    residency::record_frame_reader::RecordFrameReader, PhysicalRecordId, RecordReadDenial,
    RecordReadObservation,
};
use worth_store_physical_format::{
    DurableExtentManifest, DurableExtentRecordPlacement, RecordArtifactFile,
};

pub(super) struct AdmittedExtentManifest {
    pub(super) artifact: RecordArtifactFile,
    pub(super) manifest: DurableExtentManifest,
    pub(super) range: worth_store_physical_format::ExtentArenaRange,
    pub(super) integrity_membership:
        worth_store_physical_integrity::IntegrityValidatedExtentMembership,
}

pub(super) struct ExtentManifestAdmission<'admission, 'media> {
    pub(super) reader: &'admission PhysicalRecordReader,
    pub(super) record: PhysicalRecordId,
    pub(super) placement: DurableExtentRecordPlacement,
    pub(super) observation: &'admission mut RecordReadObservation,
    pub(super) allocation: &'admission worth_store_buffer_pool::OperationAllocationGrant,
    pub(super) artifacts: &'admission RecordFrameReader<'media>,
}

pub(super) fn admit_extent_manifest(
    mut admission: ExtentManifestAdmission<'_, '_>,
) -> Result<AdmittedExtentManifest, RecordReadDenial> {
    let (manifest, integrity_membership) = admission.load_manifest()?;
    let range = admission.placement.arena_range();
    let artifact = RecordArtifactFile::ExtentArena {
        arena: range.arena().get(),
    };
    let layout = worth_store_physical_format::ExtentArenaFrameLayout::new(
        admission.reader.format.declaration(),
        manifest.alignment(),
    )
    .ok_or(RecordReadDenial::FormatMismatch)?;
    if layout.allocated_bytes(manifest.chunk_count()) != Some(range.length()) {
        return Err(RecordReadDenial::FormatMismatch);
    }
    Ok(AdmittedExtentManifest {
        artifact,
        manifest,
        range,
        integrity_membership,
    })
}

impl ExtentManifestAdmission<'_, '_> {
    fn load_manifest(
        &mut self,
    ) -> Result<
        (
            DurableExtentManifest,
            worth_store_physical_integrity::IntegrityValidatedExtentMembership,
        ),
        RecordReadDenial,
    > {
        let bytes = self
            .artifacts
            .load_exact(
                self.allocation,
                RecordArtifactFile::ExtentArena {
                    arena: self.placement.arena_range().arena().get(),
                },
                self.placement.arena_range().offset(),
                104,
                crate::physical_runtime::record_serving::residency::frame_loading::ExactFrameSourceExtent::ArenaRange(self.placement.arena_range()),
            )
            .map_err(|failure| {
                self.observation.observe_physical_work(failure.work_trace());
                read_failure(failure)
            })?;
        self.observation.observe_physical_work(bytes.work_trace());
        self.observation.observe_manifest_block(bytes.len());
        self.observation.observe_transfer(bytes.len());
        match self.project_manifest(&bytes) {
            Ok(manifest) => Ok(manifest),
            Err(denial) => {
                if !preserves_resident_bytes(denial) {
                    bytes.reject_projection_failure();
                }
                Err(denial)
            }
        }
    }

    fn project_manifest(
        &mut self,
        frame: &crate::physical_runtime::record_serving::residency::frame_loading::LoadedPhysicalFrame,
    ) -> Result<
        (
            DurableExtentManifest,
            worth_store_physical_integrity::IntegrityValidatedExtentMembership,
        ),
        RecordReadDenial,
    > {
        let admitted = admit_clean_extent_manifest(
            frame,
            self.reader.residency.resident_admission_context(),
            self.reader.store,
            self.reader.format.declaration(),
            self.placement,
        )
        .map_err(|denial| {
            if denial == CleanExtentAdmissionDenial::ExtentMembership {
                self.observation.check_generation(false);
            }
            denial.read_denial()
        })?;
        let manifest = admitted.manifest;
        if manifest.record() != self.record.persisted()
            || manifest.logical_bytes() != self.placement.payload_bytes()
        {
            return Err(RecordReadDenial::FormatMismatch);
        }
        self.observation.check_generation(true);
        Ok((manifest, admitted.membership))
    }
}

fn preserves_resident_bytes(denial: RecordReadDenial) -> bool {
    matches!(
        denial,
        RecordReadDenial::ArtifactUnavailable
            | RecordReadDenial::PhysicalWork(
                crate::physical_runtime::record_serving::RecordReadWorkDenial::RuntimeReleased
            )
            | RecordReadDenial::ResidencyUnavailable(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_unavailability_preserves_extent_manifest_resident_bytes() {
        assert!(preserves_resident_bytes(
            CleanExtentAdmissionDenial::Unavailable.read_denial()
        ));
        assert!(!preserves_resident_bytes(
            CleanExtentAdmissionDenial::Damaged.read_denial()
        ));
    }
}
