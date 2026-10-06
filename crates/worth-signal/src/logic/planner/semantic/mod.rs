mod artifacts;
mod finalization;
mod reporting;
mod segments;
pub(crate) mod stage_recording;

pub(in crate::logic::planner) use self::finalization::finalize_serial_stage_batch;

pub(in crate::logic::planner) use self::finalization::finalize_stage_batch;
pub(in crate::logic::planner) use self::segments::{
    reserve_stage_identities, StageSemanticIdentity,
};

pub(in crate::logic::planner) use self::segments::{
    segment_for_single_update, SemanticTaskUpdate, StageSemanticBatch,
};
