use super::CleanFrameIntegrityValidationRecord;

pub(crate) fn invalidate_clean_frame_validation(
    record: &mut Option<CleanFrameIntegrityValidationRecord>,
) {
    *record = None;
}
