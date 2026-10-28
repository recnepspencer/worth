use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedProgress, SignalOwnedAsyncManagedReport,
    SignalOwnedAsyncManagedWakeCleanup,
};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn revalidate_owned_async_request_handoff(
        &mut self,
        node: ResourceNodeId,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        if intent.node() != node {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let capture = self.captures_telemetry();
        let digest = self
            .derived
            .resource
            .descriptor_for_node(node)
            .expect("admitted source retains its descriptor")
            .revalidation_decision_plan()
            .decision_digest()
            .clone();
        let prepared = match self
            .derived
            .resource
            .prepare_explicit_resource_revalidation(
                intent,
                digest,
                capture.then_some(&mut self.derived.telemetry.resource),
            ) {
            Ok(prepared) => prepared,
            Err(report) => {
                return Ok(self.replacement_handoff(
                    SignalOwnedAsyncManagedProgress::Denied,
                    SignalOwnedAsyncManagedReport::Revalidation(report),
                    None,
                    None,
                    None,
                ));
            }
        };
        let prior_timeout = self.derived.resource.active_timeout_wake_for_node(node);
        let tick = self.derived.temporal.clock_basis().current_tick();
        let timeout = self.schedule_owned_async_timeout(node, tick)?;
        let report = self.derived.resource.admit_prepared_resource_revalidation(
            prepared,
            self.branch_id(),
            tick,
            timeout.map(|(_, admission)| admission),
            capture.then_some(&mut self.derived.telemetry.resource),
        );
        let actual = report
            .admitted_revalidation()
            .expect("the prepared native revalidation kernel installs or coalesces an occurrence");
        let admitted = actual.admitted_request();
        let progress = if actual.coalescing().is_some() {
            SignalOwnedAsyncManagedProgress::Coalesced
        } else {
            SignalOwnedAsyncManagedProgress::Replaced
        };
        let (cleanup, refusal) = match self
            .retire_optional_wake(prior_timeout, TemporalWakeRetirementReason::Superseded)
        {
            Ok(()) => (None, None),
            Err(cause) => (
                prior_timeout.map(|wake| SignalOwnedAsyncManagedWakeCleanup::Retirement {
                    wake,
                    reason: TemporalWakeRetirementReason::Superseded,
                }),
                Some(cause),
            ),
        };
        Ok(self.replacement_handoff(
            progress,
            SignalOwnedAsyncManagedReport::Revalidation(report),
            Some(admitted),
            cleanup,
            refusal,
        ))
    }
}
