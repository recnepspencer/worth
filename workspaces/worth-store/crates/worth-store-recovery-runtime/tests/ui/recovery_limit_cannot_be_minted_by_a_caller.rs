// Recovery's limits are minted only by recovery's own allowance: a caller's
// refusal is not recovery's, so recovery's dimension does not accept it, and
// no public limit can be built from it.
use worth_foundational::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};
use worth_proof::Performed;
use worth_store_recovery_runtime::{PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure};

worth_proof::authority_marker!(pub CallerAuthority);

fn refusal() -> Performed<BudgetRefused, CallerAuthority, LimitCounts> {
    Performed::record(&CallerAuthority::witness(), LimitCounts::new(2, 1))
}

/// The one door a limit has, asked with the refusal its owner recorded.
fn mint<D: LimitDimension>(
    dimension: D,
    refusal: Performed<BudgetRefused, D::Authority, LimitCounts>,
) -> ExhaustedLimit<D> {
    ExhaustedLimit::refused(dimension, refusal)
}

fn main() {
    let limit = mint(PhysicalRecoveryLimitDimension::ManifestEntries, refusal());
    let _ = PhysicalRecoveryLimitFailure::from(limit);
}
