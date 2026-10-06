//! The stop a caller meets when its advance was refused required custody.
//!
//! A caller refused custody does not wait holding it: the refreshes it
//! carried end here, so no other demand's advance is refused for what a
//! stopped caller keeps. The refusal stays retryable while a later advance
//! can find the room. One of the ended refreshes had published: its row
//! stays Ready on its own, and the predecessor Ready it kept for this wave
//! is freed, so the next advance needs less at once. Or another demand holds
//! custody its advance or close moves: the registry answers that. Or the
//! caller has yet to settle and its row holds the cached Ready it joined:
//! that Ready is released, and the next advance computes the row Fresh
//! without keeping a stale Ready beside its successor. With none of these,
//! the next advance meets the same custody and the same refusal, so the
//! stop is terminal for this caller. It ends the caller's advance,
//! not the rows: retention is the caller's budget, so the rows stay
//! claimable for any demand that fits. A demand refused at its start holds
//! no row yet, so every open demand is another one.

use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// The stop `demand`'s caller meets for `stop`. What the request's work
    /// cannot establish leaves the stop as the row gave it.
    pub(super) fn caller_custody_stop<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        stop: WorthQueryOutputDemandDenial,
        admission: &mut InvalidationEditAdmission,
    ) -> WorthQueryOutputDemandDenial
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        if stop.kind() != WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
            || stop.recovery_posture() != WorthQueryOutputDemandRecoveryPosture::Retryable
        {
            return stop;
        }
        let Some(interest) = demand.interest.as_ref() else {
            return stop;
        };
        match demand
            .required_continuations
            .end_all(&self.output_demands, admission)
        {
            Ok(false) => {}
            Ok(true) | Err(_) => return stop,
        }
        match self
            .output_demands
            .another_demand_holds_custody(interest.key(), admission)
        {
            Ok(false) => {}
            Ok(true) | Err(_) => return stop,
        }
        if !demand.settled {
            match self
                .output_demands
                .release_joined_ready(interest, admission)
            {
                Ok(false) => {}
                Ok(true) | Err(_) => return stop,
            }
        }
        stop.with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Terminal)
    }

    /// The stop the caller of a demand refused at its start meets for `stop`.
    pub(super) fn starting_custody_stop(
        &self,
        stop: WorthQueryOutputDemandDenial,
        admission: &mut InvalidationEditAdmission,
    ) -> WorthQueryOutputDemandDenial {
        if stop.kind() != WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
            || stop.recovery_posture() != WorthQueryOutputDemandRecoveryPosture::Retryable
        {
            return stop;
        }
        match self.output_demands.a_demand_holds_custody(admission) {
            Ok(false) => {
                stop.with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Terminal)
            }
            Ok(true) | Err(_) => stop,
        }
    }
}
