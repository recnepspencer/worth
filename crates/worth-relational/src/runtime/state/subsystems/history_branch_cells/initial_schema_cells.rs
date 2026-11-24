//! Prepare new Empty-cell roots and reserve the original heads for replacement.
use super::*;

pub(super) fn prepare_empty_branch_cells(
    existing: &BTreeMap<BranchId, RelationalBranchReferenceCell>,
    registry: &crate::schema::data::RelationalSchemaRegistry,
) -> Result<BTreeMap<BranchId, RelationalBranchReferenceCell>, RelationalBranchCellDenial> {
    existing
        .iter()
        .map(|(branch_id, cell)| {
            let mut candidate = cell.clone_for_head_replacement();
            if matches!(
                candidate.observation().target(),
                FoundationalBranchTarget::Empty
            ) {
                candidate.advance_metadata()?;
                candidate.install_root(RelationalBranchRoot::empty_with_schema(
                    registry,
                    crate::schema::data::runtime_descriptor_semantics_policy()
                        .current_write_version(),
                ));
            }
            Ok((branch_id.clone(), candidate))
        })
        .collect::<Result<BTreeMap<_, _>, RelationalBranchCellDenial>>()
}

pub(super) struct InitialBranchHeadReplacement {
    pub(super) retirement: crate::history::retention::RelationalHeadRetirementReservation,
    pub(super) _previous_head_custody: Arc<RelationalBranchRoot>,
    pub(super) next_root: Arc<RelationalBranchRoot>,
}

pub(super) fn reserve_empty_branch_heads(
    history: &HistorySubsystem,
    existing: &BTreeMap<BranchId, RelationalBranchReferenceCell>,
    cells: &BTreeMap<BranchId, RelationalBranchReferenceCell>,
) -> Result<Vec<InitialBranchHeadReplacement>, RelationalBranchCellDenial> {
    cells
        .values()
        .filter(|cell| matches!(cell.observation().target(), FoundationalBranchTarget::Empty))
        .map(|cell| {
            let previous_root = existing
                .get(cell.identity().branch_id())
                .and_then(RelationalBranchReferenceCell::root)
                .ok_or(RelationalBranchCellDenial::RetentionOwnerUnavailable)?;
            let next_root = cell
                .root()
                .ok_or(RelationalBranchCellDenial::RetentionOwnerUnavailable)?;
            history
                .reserve_branch_head_retirement(
                    cell.identity(),
                    &previous_root,
                    cell.head_retention(),
                )
                .map(|retirement| InitialBranchHeadReplacement {
                    retirement,
                    _previous_head_custody: previous_root,
                    next_root,
                })
                .map_err(branch_head_retirement_denial)
        })
        .collect()
}
