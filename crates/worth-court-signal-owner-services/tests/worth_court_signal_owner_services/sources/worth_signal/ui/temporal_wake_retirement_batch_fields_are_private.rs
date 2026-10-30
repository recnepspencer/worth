use worth_signal::facade::{
    TemporalWakeOwner, TemporalWakeRetirementBatch, TemporalWakeRetirementReason,
};

fn main() {
    let _batch = TemporalWakeRetirementBatch {
        owner: TemporalWakeOwner::Manual,
        reason: TemporalWakeRetirementReason::Disposed,
        retired: Vec::new(),
    };
}
