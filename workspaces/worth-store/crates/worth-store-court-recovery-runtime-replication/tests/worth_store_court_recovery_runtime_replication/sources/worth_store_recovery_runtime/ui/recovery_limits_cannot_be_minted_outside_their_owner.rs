// Recovery reads every limit its owners refused, and mints none: a refusal
// recorded by recovery's own authority is not the owner's, so no owner's
// dimension accepts it.
use worth_foundational::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};
use worth_proof::Performed;
use worth_store::physical_runtime::FilesystemObservationBound;
use worth_store_physical_integrity::ReleaseCustodyHeadWalkBound;
use worth_store_recovery_physics::{HeadReplayBound, PhysicsBound, RootHistoryBound};

worth_proof::authority_marker!(pub RecoveryForgerAuthority);

fn refusal() -> Performed<BudgetRefused, RecoveryForgerAuthority, LimitCounts> {
    Performed::record(&RecoveryForgerAuthority::witness(), LimitCounts::new(1, 0))
}

/// The one door a limit has, asked with the refusal its owner recorded.
fn mint<D: LimitDimension>(
    dimension: D,
    refusal: Performed<BudgetRefused, D::Authority, LimitCounts>,
) -> ExhaustedLimit<D> {
    ExhaustedLimit::refused(dimension, refusal)
}

fn main() {
    let _ = mint(FilesystemObservationBound::Entries, refusal());
    let _ = mint(ReleaseCustodyHeadWalkBound::Nodes, refusal());
    let _ = mint(PhysicsBound::ResidentBytes, refusal());
    let _ = mint(HeadReplayBound::HeapBytes, refusal());
    let _ = mint(RootHistoryBound::Entries, refusal());
}
