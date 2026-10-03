//! Resolve only authentic paired equality links; mark status is never consulted.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::output_facts::RegisteredOutputFacts;
use std::sync::Arc;

pub(super) fn terminal<'state>(
    state: &'state MarkState,
    identity: &'state Arc<RecordedSettlementIdentity>,
    admission: &mut InvalidationEditAdmission,
) -> Result<&'state Arc<RecordedSettlementIdentity>, FullVerificationStop> {
    let mut current = identity;
    for _ in 0..=state.equal_links.len() {
        admission.ordered_read(state.equal_links.len())?;
        let Some(link) = state.equal_links.get(current) else {
            return Ok(current);
        };
        let Some(next) = &link.next else {
            return Ok(current);
        };
        admission.work(1)?;
        admission.ordered_read(state.equal_links.len())?;
        if !state
            .equal_links
            .get(next)
            .is_some_and(|following| following.prior.as_ref() == Some(current))
        {
            return Err(gap());
        }
        current = next;
    }
    Err(gap())
}

/// A stable alias preserves the preceding performed output, while owning fresh
/// handler facts. Only its output projection is inherited; stale handler facts
/// from the performed predecessor never enter full verification.
pub(super) fn output_projection<'state>(
    state: &'state MarkState,
    terminal: &'state Arc<RecordedSettlementIdentity>,
    observed_rows: &mut OrdSet<Arc<RecordedSettlementIdentity>>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<&'state RegisteredOutputFacts>, FullVerificationStop> {
    let mut current = terminal;
    for _ in 0..=state.equal_links.len() {
        image_fence::observe_row(observed_rows, current, admission)?;
        admission.ordered_read(state.settlements.len())?;
        let row = state.settlements.get(current).ok_or(gap())?;
        if let Some(projection) = &row.output_facts {
            return Ok(Some(projection));
        }
        if row.output_coverage != OutputFactCoverage::Stable {
            return Ok(None);
        }
        admission.ordered_read(state.equal_links.len())?;
        let prior = state
            .equal_links
            .get(current)
            .and_then(|link| link.prior.as_ref())
            .ok_or(gap())?;
        admission.work(1)?;
        admission.ordered_read(state.equal_links.len())?;
        if !state
            .equal_links
            .get(prior)
            .is_some_and(|preceding| preceding.next.as_ref() == Some(current))
        {
            return Err(gap());
        }
        current = prior;
    }
    Err(gap())
}

fn gap() -> FullVerificationStop {
    FullVerificationStop::Alignment(FullVerificationReason::RetainedDeliveryGap)
}
