use std::ops::Range;

use super::super::PhysicalRecordReader;
use crate::physical_runtime::record_serving::work_semantics::integrity_admission::{
    admit_inline_record, CleanInlineAdmissionDenial,
};
use crate::physical_runtime::record_serving::{
    residency::frame_loading::LoadedPhysicalFrame, PhysicalRecordId, RecordReadDenial,
    RecordReadObservation,
};
use worth_store_physical_format::DurableInlineRecordPlacement;

pub(super) struct ProjectedInlineRecord {
    pub(super) frame: LoadedPhysicalFrame,
    pub(super) payload: Range<usize>,
}

pub(super) struct InlineRecordProjection<'projection> {
    pub(super) reader: &'projection PhysicalRecordReader,
    pub(super) record: PhysicalRecordId,
    pub(super) placement: DurableInlineRecordPlacement,
    pub(super) page: LoadedPhysicalFrame,
    pub(super) observation: &'projection mut RecordReadObservation,
}

pub(super) fn project_inline_record(
    projection: InlineRecordProjection<'_>,
) -> Result<ProjectedInlineRecord, RecordReadDenial> {
    let InlineRecordProjection {
        reader,
        record,
        placement,
        page,
        observation,
    } = projection;
    let admitted = admit_inline_record(
        &page,
        reader.residency.resident_admission_context(),
        reader.store,
        reader.format.declaration(),
        placement,
    );
    let admitted = match admitted {
        Ok(admitted) => admitted,
        Err(denial) => {
            observe_denial(observation, denial);
            if !denial.preserves_resident_bytes() {
                page.reject_projection_failure();
            }
            return Err(denial.read_denial());
        }
    };
    observation.check_page_identity_generation(true);
    observation.check_slot_generation(true);
    debug_assert_eq!(record.persisted(), placement.record());
    Ok(ProjectedInlineRecord {
        frame: page,
        payload: admitted.payload,
    })
}

fn observe_denial(observation: &mut RecordReadObservation, denial: CleanInlineAdmissionDenial) {
    match denial {
        CleanInlineAdmissionDenial::PageIdentity => {
            observation.check_page_identity_generation(false);
        }
        CleanInlineAdmissionDenial::SlotGeneration => {
            observation.check_page_identity_generation(true);
            observation.check_slot_generation(false);
        }
        CleanInlineAdmissionDenial::Format
        | CleanInlineAdmissionDenial::Unavailable
        | CleanInlineAdmissionDenial::RuntimeReleased
        | CleanInlineAdmissionDenial::Residency(_)
        | CleanInlineAdmissionDenial::Damaged => {}
    }
}
