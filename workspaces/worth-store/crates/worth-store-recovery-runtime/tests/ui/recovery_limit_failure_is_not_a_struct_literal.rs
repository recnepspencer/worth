// A reported limit holds only counts recovery's allowance refused: a caller
// cannot build one from numbers of its own.
use worth_store_recovery_runtime::{PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure};

fn main() {
    let _ = PhysicalRecoveryLimitFailure {
        dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
        observed: 2,
        admitted: 1,
    };
}
