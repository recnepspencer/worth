//! Initial Empty-cell replacement prepares fully before its actual durable cutover.
use super::initial_schema_cells::{prepare_empty_branch_cells, reserve_empty_branch_heads};
use super::*;
use crate::branch::transition::original_inputs::OriginalEmptySchemaObservation;

pub(crate) fn transition_empty_branches_to_initial_schema(
    history: &crate::runtime::state::subsystems::history::history_canonical_routes::RelationalPublicationHistory,
    registry: &crate::schema::data::RelationalSchemaRegistry,
    prepare: impl FnOnce(
        &OriginalEmptySchemaObservation,
        &OriginalEmptySchemaObservation,
    ) -> Result<
        crate::runtime::initial_schema_installation::InitialSchemaCutover,
        crate::runtime::RelationalInitialSchemaInstallationDenial,
    >,
) -> Result<(), crate::runtime::RelationalInitialSchemaInstallationDenial> {
    history.record_branch_population_scan();
    let existing = history.branch_cells.take_all();
    let transitioned = prepare_empty_branch_cells(&existing, registry);
    match transitioned {
        Ok(cells) => {
            let reservations = reserve_empty_branch_heads(history, &existing, &cells);
            let reservations = match reservations {
                Ok(reservations) => reservations,
                Err(denial) => {
                    history.branch_cells.restore_all(existing);
                    return Err(crate::runtime::initial_schema_installation::denial::branch_transition_denial(denial));
                }
            };
            // Main is the only legal live Empty branch. Initial schema
            // installation seals once; retain its exact original before
            // and after DATA before replacing any candidate cell.
            let Some((before, after)) = existing
                .get(&history.main_branch)
                .and_then(OriginalEmptySchemaObservation::from_empty_cell)
                .zip(
                    cells
                        .get(&history.main_branch)
                        .and_then(OriginalEmptySchemaObservation::from_empty_cell),
                )
            else {
                history.branch_cells.restore_all(existing);
                return Err(
                    crate::runtime::initial_schema_installation::denial::branch_transition_denial(
                        RelationalBranchCellDenial::RetentionOwnerUnavailable,
                    ),
                );
            };
            let cutover = match prepare(&before, &after) {
                Ok(cutover) => cutover,
                Err(cause) => {
                    history.branch_cells.restore_all(existing);
                    return Err(cause);
                }
            };
            cutover.apply(|accepted| {
                if let Some(inputs) = accepted {
                    history
                        .canonical_publication_routes
                        .retain_accepted_initial_inputs(inputs);
                }
                history
                    .canonical_publication_routes
                    .retain_original_empty_installation(before, after);
                history.branch_cells.restore_all(cells);
                for replacement in reservations {
                    replacement
                        .retirement
                        .transfer_head(&replacement.next_root)
                        .replace_head();
                }
            });
            Ok(())
        }
        Err(denial) => {
            history.branch_cells.restore_all(existing);
            Err(
                crate::runtime::initial_schema_installation::denial::branch_transition_denial(
                    denial,
                ),
            )
        }
    }
}
