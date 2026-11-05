//! Preserve the actual participant refusal, including any cleanup custody.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation::primary_graph) fn coupled_producer(
        denial: WorthQueryOutputDemandDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage: WorthQueryApplicationCommitDenialStage::InvariantExecution,
            detail: Some(denial.subject().to_owned().into()),
            cause: Some(Box::new(denial_cause::DenialCause::CoupledProducer(denial))),
        }
    }
    /// Original owner producer's typed refusal; no effect was published.
    pub fn coupled_producer_denial(&self) -> Option<&WorthQueryOutputDemandDenial> {
        match self.cause.as_deref() {
            Some(denial_cause::DenialCause::CoupledProducer(denial)) => Some(denial),
            _ => None,
        }
    }
}
