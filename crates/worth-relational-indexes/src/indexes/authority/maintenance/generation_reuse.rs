//! Preserve payload identity when a reached maintainer produces no change.
use super::work::MaintenanceWork;
use crate::runtime::state::subsystems::indexing::PreparedIndexBasisSelection;
use std::sync::Arc;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexGeneration;
use worth_relational_contracts::internal_port::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind as Denial;

pub(super) fn retain(
    previous: Arc<DerivedIndexGeneration>,
    basis: &mut Option<PreparedIndexBasisSelection>,
    reused: &mut Vec<DerivedIndexGeneration>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    work.grow_vec(reused)?;
    work.prepare(
        1,
        (previous.source_branch_id.0.len() + previous.applicability.branch_id.0.len()) as u64,
    )?;
    reused.push(previous.as_ref().clone());
    if let Some(basis) = basis {
        basis.replace_admitted(previous, &mut |units, bytes| work.prepare(units, bytes))?;
    }
    work.note_reused_generations(1);
    Ok(())
}
