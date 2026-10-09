use worth_store_recovery_physics::{
    PhysicalSourceSelection, PhysicalWalSegmentCandidate, SelectedPhysicalWalTail,
};

fn requires_clone<T: Clone>() {}

fn main() {
    requires_clone::<PhysicalWalSegmentCandidate>();
    requires_clone::<SelectedPhysicalWalTail>();
    requires_clone::<PhysicalSourceSelection>();
}
