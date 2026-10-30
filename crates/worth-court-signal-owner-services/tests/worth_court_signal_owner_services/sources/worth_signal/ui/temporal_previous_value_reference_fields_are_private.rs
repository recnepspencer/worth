use worth_signal::facade::{
    Aspect, AspectVersion, ClockTick, NodeId, PreviousValueRevision, SignalBranchId,
    TemporalPreviousValueReference, TemporalWakeId,
};

fn main() {
    let _reference = TemporalPreviousValueReference {
        revision: PreviousValueRevision::new(1),
        branch_id: SignalBranchId(0),
        access_wake_id: TemporalWakeId::new(0),
        node: NodeId::new(0, 0),
        captured_at_tick: ClockTick::new(2),
        aspect_version: AspectVersion::from_updates([(Aspect::new(0), 1)]),
        output_identity: None,
    };
}
