//! Move the verified native basis with a suspended owner publication.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

pub(in crate::domain_computation::primary_graph) enum OwnedInputCutoffDecision {
    Fresh {
        key: PreparedInputReuseKey,
        context: PreparedDecisionReuseContext,
    },
    Reuse(OwnedVerifiedInputCutoff),
}

pub(in crate::domain_computation::primary_graph) struct OwnedVerifiedInputCutoff {
    computation: super::super::super::CurrentComputation,
    candidate: RetainedInputCutoffCandidate,
    selected: PositionedRelationalSnapshot,
    fresh_key: Option<PreparedInputReuseKey>,
    verified_facts: super::super::super::ComparableSourceFacts,
    verified_consumed: Option<std::sync::Arc<[ConsumedOutputEvidence]>>,
}

impl RetainedInputCutoffCandidate {
    pub(in crate::domain_computation::primary_graph) fn verify_for_reuse_owned(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh_key: PreparedInputReuseKey,
        fresh_context: PreparedDecisionReuseContext,
        matched: Option<MatchedRequiredPredecessors<'_>>,
        runtime: &RelationalRuntime,
        basis: PreparedInputCutoffBasis<'_>,
        owner: &SourceInvalidationOwner,
    ) -> Result<OwnedInputCutoffDecision, InputCutoffVerificationStop> {
        match self.verify_for_reuse(
            execution,
            fresh_key,
            fresh_context,
            matched,
            runtime,
            &basis,
            owner,
        )? {
            InputCutoffDecision::Fresh { key, context } => {
                Ok(OwnedInputCutoffDecision::Fresh { key, context })
            }
            InputCutoffDecision::Reuse(verified) => {
                let VerifiedInputCutoff {
                    _computation: computation,
                    candidate,
                    selected: _,
                    fresh_key,
                    verified_facts,
                    verified_consumed,
                } = verified;
                Ok(OwnedInputCutoffDecision::Reuse(OwnedVerifiedInputCutoff {
                    computation,
                    candidate,
                    selected: basis.into_positioned(),
                    fresh_key,
                    verified_facts,
                    verified_consumed,
                }))
            }
        }
    }
}

impl OwnedVerifiedInputCutoff {
    /// Owner application compares the prepared facts again when an earlier
    /// member moved the native head. A selected proof alone cannot authorize
    /// current-only source registration at a different native position.
    pub(in crate::domain_computation::primary_graph) fn recompare_for_publication(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        basis: PreparedInputCutoffBasis<'_>,
        owner: &SourceInvalidationOwner,
        source_facts: &(impl FactSequence + ?Sized),
    ) -> Result<Self, InputCutoffVerificationStop> {
        observation::reach(Gate::PublicationSource);
        let (snapshot, selected) = basis.in_runtime(runtime)?;

        if self.selected == *selected {
            return Ok(self);
        }
        if self.selected.runtime_instance_id() != selected.runtime_instance_id()
            || self.selected.branch_id() != selected.branch_id()
        {
            return Err(InputCutoffVerificationStop::SelectedSourceMismatch);
        }
        observation::reach(Gate::PublicationPrefix);
        let prefix = self
            .candidate
            .completed_handler_fact_count()
            .unwrap_or(self.verified_facts.len());
        if prefix > self.verified_facts.len()
            || !facts_are_current(
                execution,
                self.verified_facts
                    .iter()
                    .take(prefix)
                    .chain(source_facts.iter()),
                runtime,
                snapshot,
            )?
        {
            return Err(InputCutoffVerificationStop::CurrentnessRaced);
        }
        observation::reach(Gate::PublicationConsumedOutputs);
        let current = ConsumedOutputEvidence::verify_many_with_admission(
            execution,
            self.verified_consumed
                .as_deref()
                .unwrap_or_else(|| self.candidate.consumed_outputs()),
            owner,
            runtime,
            snapshot,
            selected,
        )
        .map_err(|stop| match stop {
            stop @ ConsumedOutputVerificationStop::SourceQueryAuthorization(_) => {
                InputCutoffVerificationStop::OutputObservation(stop)
            }
            ConsumedOutputVerificationStop::Execution(cause) => {
                InputCutoffVerificationStop::Execution(cause)
            }
            ConsumedOutputVerificationStop::Unavailable
            | ConsumedOutputVerificationStop::RetryCurrentness(_) => {
                InputCutoffVerificationStop::CurrentnessRaced
            }
            ConsumedOutputVerificationStop::CapacityExhausted(_) => {
                InputCutoffVerificationStop::CapacityExhausted
            }
            ConsumedOutputVerificationStop::WorkExhausted => {
                InputCutoffVerificationStop::WorkExhausted
            }
            ConsumedOutputVerificationStop::PendingUpstream => {
                InputCutoffVerificationStop::PendingUpstream
            }
            ConsumedOutputVerificationStop::Interrupted(event) => {
                InputCutoffVerificationStop::Admission(CompanionPreflightStop::Interrupted(event))
            }
        })?;
        if !matches!(current, ConsumedOutputVerification::Current) {
            return Err(InputCutoffVerificationStop::CurrentnessRaced);
        }
        observation::reach(Gate::PublicationOutput);
        if !self
            .candidate
            .native_output_witness()
            .ok_or(InputCutoffVerificationStop::CurrentnessRaced)?
            .unchanged_in(runtime, snapshot, execution)?
        {
            return Err(InputCutoffVerificationStop::CurrentnessRaced);
        }
        // Only the owner which compared every prepared input, consumed output
        // and retained postcondition can move the exact native registration basis.
        self.selected = basis.into_positioned();
        Ok(self)
    }

    /// The publication receives the original proof and basis, without a new
    /// lookup, a clone of the native position, or another verification charge.
    pub(in crate::domain_computation::primary_graph) fn with_verified<R>(
        self,
        publish: impl for<'basis> FnOnce(VerifiedInputCutoff<'basis>) -> R,
    ) -> R {
        let Self {
            computation,
            candidate,
            selected,
            fresh_key,
            verified_facts,
            verified_consumed,
        } = self;
        publish(VerifiedInputCutoff {
            _computation: computation,
            candidate,
            selected: &selected,
            fresh_key,
            verified_facts,
            verified_consumed,
        })
    }
}
