//! Where a row stands, read without beginning it.

use super::{
    DemandRecord, DemandState, ReadyCompletion, WorthQueryOutputAdvancement,
    WorthQueryOutputCheckpoint, WorthQueryOutputDemandInterest, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// The stage of a demand's row. Reading it claims nothing and moves nothing,
/// so a demand can decide what it does before it begins the row.
pub(in crate::domain_computation::primary_graph) enum OutputRowStage {
    /// Another call is scheduling, executing or advancing the row.
    Driven,
    Stopped(WorthQueryOutputDemandDenial),
    Ready(ReadyCompletion),
    /// A real committed output still owes delivery or readiness; no call holds it.
    Published,
    /// Admitted or scheduled, with no committed output: no call holds it.
    BeforeReady,
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn row_stage(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> OutputRowStage {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get(&interest.key)
            .expect("live demand interest retains its owner record");
        stage_of(record)
    }

    /// The required driver observes the stage under its original request meter.
    pub(in crate::domain_computation::primary_graph) fn row_stage_admitted(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<OutputRowStage, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(&interest.key, admission)?;
        let record = state
            .records
            .get(&interest.key)
            .expect("live demand interest retains its owner record");
        Ok(stage_of(record))
    }
}

fn stage_of(record: &DemandRecord) -> OutputRowStage {
    match &record.state {
        DemandState::Admitted | DemandState::Scheduled => OutputRowStage::BeforeReady,
        DemandState::Scheduling | DemandState::Running => OutputRowStage::Driven,
        DemandState::Failed(denial) => OutputRowStage::Stopped(denial.clone()),
        DemandState::Output(output) => match &output.advancement {
            WorthQueryOutputAdvancement::Claimed(_) => OutputRowStage::Driven,
            WorthQueryOutputAdvancement::Stopped { denial, .. } => {
                OutputRowStage::Stopped(denial.clone())
            }
            WorthQueryOutputAdvancement::Idle => match &output.checkpoint {
                Some(WorthQueryOutputCheckpoint::Ready(completion)) => {
                    OutputRowStage::Ready(completion.clone())
                }
                Some(
                    WorthQueryOutputCheckpoint::Published { .. }
                    | WorthQueryOutputCheckpoint::Delivered { .. },
                ) => OutputRowStage::Published,
                None => OutputRowStage::BeforeReady,
            },
        },
    }
}
