//! What a transported row needs to name its committed selection read evidence.

/// The selected step's basis and the restoring commit's basis. They only name
/// the read evidence Native constructs from the actual committed selection;
/// they are never authority, and a mismatched pair yields no evidence.
pub struct HistoricalSelectionCoordinate {
    pub(in crate::domain_computation::primary_graph) selected:
        worth_relational::facade::mvcc::RelationalBranchObservation,
    pub(in crate::domain_computation::primary_graph) restored:
        worth_relational::facade::mvcc::RelationalBranchObservation,
}
