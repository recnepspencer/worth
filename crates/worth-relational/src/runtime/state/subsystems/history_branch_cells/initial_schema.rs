//! Initial Empty-cell replacement prepares fully before its actual durable cutover.
use super::*;
use crate::runtime::OriginalEmptySchemaObservation;
impl HistorySubsystem {
    pub(crate) fn transition_empty_branches_to_initial_schema(
        &self,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        prepare: impl FnOnce(
            &OriginalEmptySchemaObservation,
            &OriginalEmptySchemaObservation,
        ) -> Result<
            crate::runtime::initial_schema_installation::InitialSchemaCutover,
            crate::runtime::RelationalInitialSchemaInstallationDenial,
        >,
    ) -> Result<(), crate::runtime::RelationalInitialSchemaInstallationDenial> {
        self.record_branch_population_scan();
        let existing = self.branch_cells.take_all();
        let transitioned = existing
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
            .collect::<Result<BTreeMap<_, _>, RelationalBranchCellDenial>>();
        match transitioned {
            Ok(cells) => {
                let reservations = cells
                    .values()
                    .filter(|cell| {
                        matches!(cell.observation().target(), FoundationalBranchTarget::Empty)
                    })
                    .map(|cell| {
                        let previous_root = existing
                            .get(cell.identity().branch_id())
                            .and_then(RelationalBranchReferenceCell::root)
                            .ok_or(RelationalBranchCellDenial::RetentionOwnerUnavailable)?;
                        let next_root = cell
                            .root()
                            .ok_or(RelationalBranchCellDenial::RetentionOwnerUnavailable)?;
                        self.reserve_branch_head_retirement(
                            cell.identity(),
                            &previous_root,
                            cell.head_retention(),
                        )
                        .map(|reservation| (reservation, previous_root, next_root))
                        .map_err(branch_head_retirement_denial)
                    })
                    .collect::<Result<Vec<_>, _>>();
                let reservations = match reservations {
                    Ok(reservations) => reservations,
                    Err(denial) => {
                        self.branch_cells.restore_all(existing);
                        return Err(crate::runtime::initial_schema_installation::denial::branch_transition_denial(denial));
                    }
                };
                // Main is the only legal live Empty branch. Initial schema
                // installation seals once; retain its exact original before
                // and after DATA before replacing any candidate cell.
                let Some((before, after)) = existing
                    .get(&self.main_branch)
                    .and_then(OriginalEmptySchemaObservation::from_empty_cell)
                    .zip(
                        cells
                            .get(&self.main_branch)
                            .and_then(OriginalEmptySchemaObservation::from_empty_cell),
                    )
                else {
                    self.branch_cells.restore_all(existing);
                    return Err(crate::runtime::initial_schema_installation::denial::branch_transition_denial(RelationalBranchCellDenial::RetentionOwnerUnavailable));
                };
                let cutover = match prepare(&before, &after) {
                    Ok(cutover) => cutover,
                    Err(cause) => {
                        self.branch_cells.restore_all(existing);
                        return Err(cause);
                    }
                };
                cutover.apply(|accepted| {
                    if let Some(inputs) = accepted {
                        self.canonical_publication_routes
                            .retain_accepted_initial_inputs(inputs);
                    }
                    self.canonical_publication_routes
                        .retain_original_empty_installation(before, after);
                    self.branch_cells.restore_all(cells);
                    for (reservation, _previous_head_custody, next_root) in reservations {
                        reservation.transfer_head(&next_root).replace_head();
                    }
                });
                Ok(())
            }
            Err(denial) => {
                self.branch_cells.restore_all(existing);
                Err(
                    crate::runtime::initial_schema_installation::denial::branch_transition_denial(
                        denial,
                    ),
                )
            }
        }
    }
}
