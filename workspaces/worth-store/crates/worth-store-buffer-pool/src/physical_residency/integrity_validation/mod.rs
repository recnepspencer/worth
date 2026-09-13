mod clean_record;
mod invalidation;

pub(crate) use clean_record::CleanFrameIntegrityValidationRecord;
pub use clean_record::{CleanFrameIntegrityValidationDenial, PhysicalResidentFrameGeneration};
pub(crate) use invalidation::invalidate_clean_frame_validation;
