//! A Home supplies an already attached durable Signal owner, never a companion graph.
use super::*;
use crate::lifecycle::RuntimeWorldAuthoredSignalAttachment;
impl<B, R, U, C, X>
    RuntimeWorldOwnerBuilder<B, R, MissingRuntimeWorldInput, MissingRuntimeWorldInput, U, C, X>
{
    pub fn with_authored_signal<D, I, E, Ctx, T>(
        self,
        signal: Arc<RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>>,
    ) -> RuntimeWorldOwnerBuilder<
        B,
        R,
        Arc<RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>>,
        MissingRuntimeWorldInput,
        U,
        C,
        X,
    >
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock: self.clock,
            execution_authority: self.execution_authority,
            execution_policy: self.execution_policy,
        }
    }
}
