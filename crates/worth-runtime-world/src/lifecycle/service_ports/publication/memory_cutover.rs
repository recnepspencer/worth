//! Caller-specific result packaging around the original concrete owner path.
use super::*;
use crate::lifecycle::PreparedWorldRelationalCutover;

fn with_handoff<R, O>(
    handoff: impl for<'action, 'native, 'policy, 'request> FnOnce(
        PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request>,
    ) -> R,
    execute: impl FnOnce(&mut RelationalCutoverHandoff<'_>) -> O,
) -> (Option<R>, O) {
    let mut handoff = Some(handoff);
    let mut result = None;
    let outcome = execute(&mut |cutover| {
        result = Some(handoff.take().expect("one original Native handoff")(
            cutover,
        ));
    });
    (result, outcome)
}

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Lend the original prepared Native cutover before its World guard. This
    /// entrance requires a new Native publication with a current-read input.
    /// Caller status and the actual publication are returned separately.
    pub fn execute_without_signal_with_memory_cutover<R>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithoutSignal,
        cancellation: &RuntimeWorldCancellationToken,
        handoff: impl for<'action, 'native, 'policy, 'request> FnOnce(
            PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request>,
        ) -> R,
    ) -> (Option<R>, RuntimeWorldPublicationOutcome) {
        with_handoff(handoff, |handoff| {
            self.execute_without_signal_cutover(execution, prepared, cancellation, Some(handoff))
        })
    }

    /// Run the same guarded Native handoff, then the original Signal leg and
    /// product movement after the callback ends. A caller error cannot erase a
    /// genuine Native effect. An unused token performs nothing.
    pub fn execute_with_signal_with_memory_cutover<F, R>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithSignal,
        context: &mut Ctx,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
        handoff: impl for<'action, 'native, 'policy, 'request> FnOnce(
            PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request>,
        ) -> R,
    ) -> (Option<R>, RuntimeWorldPublicationOutcome)
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        with_handoff(handoff, |handoff| {
            self.execute_with_signal_cutover(
                execution,
                prepared,
                context,
                cancellation,
                apply,
                Some(handoff),
            )
        })
    }
}

impl RuntimeWorldPublicationPort<(), (), (), (), ()> {
    /// Publish the original Bridge definition with a scoped Native handoff.
    /// Native preflight precedes the callback; Bridge/Signal work, activation,
    /// settlement and product movement follow it on the original path.
    pub fn publish_bridge_conditional_definition_with_memory_cutover<R>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithSignal,
        bridge_prepared: worth_runtime_bridge::facade::BridgePreparedConditionalInstallationExtension,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        cancellation: &RuntimeWorldCancellationToken,
        handoff: impl for<'action, 'native, 'policy, 'request> FnOnce(
            PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request>,
        ) -> R,
    ) -> (
        Option<R>,
        crate::publication::RuntimeWorldConditionalDefinitionPublicationOutcome,
    ) {
        with_handoff(handoff, |handoff| {
            self.publish_bridge_conditional_definition_cutover(
                execution,
                prepared,
                bridge_prepared,
                bridge,
                cancellation,
                Some(handoff),
            )
        })
    }
}
