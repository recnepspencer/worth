//! Bounded passive receipts for the existing producer input-cutoff path.
use std::cell::Cell;
mod settlement;
pub(in crate::domain_computation::primary_graph) use settlement::settlement_decline;
pub use settlement::{
    WorthQueryInputCutoffCandidateForTest, WorthQueryInputCutoffNativeBasisForTest,
    WorthQueryInputCutoffNoRowReasonForTest, WorthQueryInputCutoffSettlementForTest,
    WorthQueryInputCutoffSettlementPostureForTest,
};

/// The existing gate reached immediately before a cutoff's result.
/// These observations never authorize reuse or change a comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInputCutoffGateForTest {
    ExecutionRequest,
    ComputationPrior,
    PreparedKey,
    PreparedContext,
    Candidate,
    RegistryCustody,
    PredecessorHandoff,
    NativeBasis,
    PriorKey,
    CompletedContext,
    OutputWitness,
    InputIdentity,
    InstalledEdition,
    SelectedMembership,
    DecisionContext,
    RestoreFence,
    ComparableFacts,
    HandlerPrefixBoundary,
    SettlementCurrentness,
    MatchedConsumer,
    MatchedRoots,
    OwnEvidence,
    PendingRoots,
    HandlerPrefix,
    HandlerPrefixAllowance,
    ConsumedOutputs,
    OutputCurrentness,
    SourceConsumption,
    PublicationSelection,
    PublicationBasis,
    PublicationSource,
    PublicationPrefix,
    PublicationConsumedOutputs,
    PublicationOutput,
    PublicationApply,
}

/// Eligibility and actual publication are separate receipts. A dropped or
/// stopped preparation never reports either as successful.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInputCutoffOutcomeForTest {
    Fresh,
    Stopped,
    Eligible,
    Published,
}

/// One invocation's first decisive gate, or the last gate of its success.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInputCutoffForTest {
    pub producer: &'static str,
    pub gate: WorthQueryInputCutoffGateForTest,
    pub outcome: WorthQueryInputCutoffOutcomeForTest,
    /// The original no-row answer at SettlementCurrentness, when observed.
    /// None never certifies a clean row or successful mark re-establishment.
    pub settlement_decline: Option<WorthQueryInputCutoffSettlementForTest>,
}

thread_local! {
    static CURRENT: Cell<Option<WorthQueryInputCutoffForTest>> = const { Cell::new(None) };
}

pub(in crate::domain_computation::primary_graph) use WorthQueryInputCutoffGateForTest as Gate;
pub(in crate::domain_computation::primary_graph) use WorthQueryInputCutoffOutcomeForTest as Outcome;

/// Only the existing owned execution listener enables this passive scope.
/// Nested invocations restore the enclosing receipt instead of overwriting it.
pub(in crate::domain_computation::primary_graph) struct Scope {
    prior: Option<WorthQueryInputCutoffForTest>,
    installed: bool,
}

impl Scope {
    pub(in crate::domain_computation::primary_graph) fn begin(
        producer: &'static str,
        gate: Gate,
    ) -> Self {
        let installed = super::execution_observation::is_installed();
        let prior = if installed {
            CURRENT.replace(Some(WorthQueryInputCutoffForTest {
                producer,
                gate,
                outcome: Outcome::Stopped,
                settlement_decline: None,
            }))
        } else {
            None
        };
        Self { prior, installed }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if self.installed
            && let Some(receipt) = CURRENT.replace(self.prior)
        {
            super::record(|observed| {
                match observed
                    .input_cutoffs
                    .iter_mut()
                    .find(|slot| slot.is_none())
                {
                    Some(slot) => *slot = Some(receipt),
                    None => super::add(&mut observed.input_cutoffs_omitted, 1),
                }
            });
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn reach(gate: Gate) {
    if !super::execution_observation::is_installed() {
        return;
    }
    CURRENT.with(|current| {
        if let Some(mut receipt) = current.get() {
            receipt.gate = gate;
            current.set(Some(receipt));
        }
    });
}

pub(in crate::domain_computation::primary_graph) fn finish(outcome: Outcome) {
    if !super::execution_observation::is_installed() {
        return;
    }
    CURRENT.with(|current| {
        if let Some(mut receipt) = current.get() {
            receipt.outcome = outcome;
            current.set(Some(receipt));
        }
    });
}

#[cfg(test)]
mod tests;
