//! Native touched-key extraction and exact persistent mark delivery.
use super::super::RecordedSettlementIdentity;
use super::fact_key::FactPostingKey;
use super::logical_marking::AppliedNativeMarking;
use super::logical_marking::NativeMarkingReport;
use super::logical_marking::WorthQueryNativeMarkingPrecision;
use super::logical_marking::WorthQueryNativeMarkingWork;
use super::mark_state::DeliveryEpoch;
use super::mark_state::FullVerificationReason;
use super::mark_state::MarkState;
use super::touch_keys;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;
use im::OrdSet;
use std::sync::Arc;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::PublicationCompanionPreflight;
pub(super) mod affected;
pub(super) use affected::mark_keys;
pub(super) use affected::propagate;
pub(super) fn selectors(
    context: &PublicationCompanionPreflight<'_>,
) -> Result<Option<Arc<[FactPostingKey]>>, CompanionPreflightStop> {
    let Some(touches) = context
        .canonical_commit()
        .descriptive_touches()
        .exact_touches()
    else {
        return Ok(None);
    };
    let mut keys = Vec::new();
    for touch in touches {
        context.checkpoint()?;
        touch_keys::visit(touch, |key| {
            keys.push(key);
            Ok(())
        })?;
    }
    Ok(Some(keys.into()))
}
pub(super) struct MarkedDelivery {
    pub(super) state: MarkState,
    pub(super) report: NativeMarkingReport,
    pub(super) affected: Option<OrdSet<Arc<RecordedSettlementIdentity>>>,
    pub(super) selected: Vec<Arc<RequiredWorkMembership>>,
    pub(super) keys: Option<Arc<[FactPostingKey]>>,
}
pub(super) fn mark(
    observed: &MarkState,
    keys: Option<Arc<[FactPostingKey]>>,
    commit: CommitId,
    context: &PublicationCompanionPreflight<'_>,
) -> Result<MarkedDelivery, CompanionPreflightStop> {
    context.checkpoint()?;
    let mut state = observed.clone();
    let mut counts = WorthQueryNativeMarkingWork::default();
    let applied = affected::apply(&mut state, keys.as_deref(), commit, &mut counts)?;
    let mut selected = Vec::new();
    for identity in &applied.affected {
        context.checkpoint()?;
        if let Some(membership) = state
            .settlements
            .get(identity)
            .and_then(|row| row.work_membership.as_ref())
        {
            selected.push(Arc::clone(membership));
        }
    }
    let affected = (crate::domain_computation::execution_observation::is_installed()
        && matches!(
            applied.report.precision,
            WorthQueryNativeMarkingPrecision::Exact(_)
        ))
    .then_some(applied.affected);
    context.checkpoint()?;
    Ok(MarkedDelivery {
        state,
        report: applied.report,
        affected,
        selected,
        keys,
    })
}
