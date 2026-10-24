use worth_store::physical_runtime::RecoveryStagingResidueOutcome;

use crate::progression::ReopenedPhysicalRecovery;

/// The bound on stale candidates one recovery removes. A killed runtime leaves
/// at most one publication attempt's three candidates per crash; a larger
/// population is evidence for inspection, not residue.
const MAXIMUM_STALE_CANDIDATES: usize = 48;

/// Removes root-protocol candidates a killed runtime left in record staging,
/// after recovery's own publication is namespace durable.
pub(super) fn prepare(reopened: &ReopenedPhysicalRecovery) -> RecoveryStagingResidueOutcome {
    reopened
        .state
        .coordination
        .owner()
        .remove_record_staging_residue(&reopened.state.authority.media, MAXIMUM_STALE_CANDIDATES)
}
