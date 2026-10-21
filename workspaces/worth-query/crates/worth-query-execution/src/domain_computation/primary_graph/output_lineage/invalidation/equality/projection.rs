//! A certified successor shares output custody, without retaining obsolete inputs.
use super::super::mark_state::FullVerificationReason;
use super::super::mark_state::MarkState;
use super::super::SettlementRegistrationStop;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use std::sync::Arc;

pub(super) fn carry(
    state: &mut MarkState,
    predecessor: &Arc<RecordedSettlementIdentity>,
    successor: &Arc<RecordedSettlementIdentity>,
) -> Result<(), SettlementRegistrationStop> {
    let mut current = Arc::clone(predecessor);
    let mut projection = None;
    for _ in 0..=state.equal_links.len() {
        let row = state
            .settlements
            .get(&current)
            .ok_or(SettlementRegistrationStop::Alignment(
                FullVerificationReason::MissingSettlement,
            ))?;
        if let Some(output) = &row.output_facts {
            projection = Some(output.clone());
            break;
        }

        let Some(prior) = state
            .equal_links
            .get(&current)
            .and_then(|link| link.prior.clone())
        else {
            break;
        };
        current = prior;
    }
    // A source-only row has no separate output projection. Its equality chain
    // retains the origin as before; only a carried projection permits pruning.
    let Some(projection) = projection else {
        return Ok(());
    };

    let mut row =
        (**state
            .settlements
            .get(successor)
            .ok_or(SettlementRegistrationStop::Alignment(
                FullVerificationReason::MissingSettlement,
            ))?)
        .clone();
    // Both Arc fields share the facts' original allocation and its final-owner
    // ticket. The successor's own fact postings already cover this projection.
    row.output_facts = Some(projection);

    state
        .settlements
        .insert(Arc::clone(successor), Arc::new(row));
    Ok(())
}
