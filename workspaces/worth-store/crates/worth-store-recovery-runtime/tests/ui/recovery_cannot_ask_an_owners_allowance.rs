// An allowance is asked only by the owner whose bound it names: recovery can
// neither name one nor ask it for a limit.
use worth_store::physical_runtime::FilesystemObservationAllowance;
use worth_store_physical_integrity::ReleaseCustodyHeadWalkAllowance;
use worth_store_recovery_physics::{HeadReplayAllowance, PhysicsAllowance, RootHistoryAllowance};

fn main() {
    let _ = PhysicsAllowance::resident_bytes(0).admit(1);
    let _ = FilesystemObservationAllowance::entries(0).admit(1);
    let _ = ReleaseCustodyHeadWalkAllowance::nodes(0).admit(1);
    let _ = HeadReplayAllowance::heap_bytes(0).admit(1);
    let _ = RootHistoryAllowance::entries(0).admit(1);
}
