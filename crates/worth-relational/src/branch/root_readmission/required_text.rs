//! Dictionary closure for retained storage before native root readmission.
use crate::branch::root::RelationalBranchRootCaptureDenial;
use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;
use crate::symbols::data::StringInterner;
use std::collections::BTreeMap;

/// Capture and recovery use the storage owner's same complete symbol closure.
pub(super) fn validate(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    partitions: &BTreeMap<PartitionId, PartitionState>,
    symbols: &StringInterner,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    for partition in partitions.values() {
        partition.visit_retained_symbols(lane, &mut |symbol| {
            symbols.retained_text(symbol).map(|_| ()).ok_or(
                RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(symbol),
            )
        })?;
    }
    Ok(())
}
