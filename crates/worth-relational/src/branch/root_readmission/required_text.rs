//! Dictionary closure for retained storage before native root readmission.
use crate::{
    branch::RelationalBranchRootCaptureDenial, identity::data::PartitionId,
    storage::overlay::PartitionState, symbols::data::StringInterner,
};
use std::collections::BTreeMap;

/// Capture and recovery use the storage owner's same complete symbol closure.
pub(super) fn validate(
    partitions: &BTreeMap<PartitionId, PartitionState>,
    symbols: &StringInterner,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    for partition in partitions.values() {
        partition.visit_retained_symbols(&mut |symbol| {
            symbols.retained_text(symbol).map(|_| ()).ok_or(
                RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(symbol),
            )
        })?;
    }
    Ok(())
}
