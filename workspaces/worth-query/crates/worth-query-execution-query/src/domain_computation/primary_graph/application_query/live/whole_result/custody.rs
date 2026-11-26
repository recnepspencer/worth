//! The runtime a live lease reads through: borrowed from its caller, or held by
//! shared custody so the lease can live beside the runtime's owner.
use std::sync::Arc;

use crate::domain_computation::managed_run::WorthQueryManagedLowerExecutionBasis;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

use super::WorthQueryApplicationWholeResultLiveLease;

/// An installed allocation that keeps its runtime alive for a shared live lease.
///
/// Implemented by the installed program runtime only; holding one counts as a
/// runtime keepalive, so the Home refuses a clean close until it is released.
pub trait WorthQuerySharedLiveRuntime<Schema>: Send + Sync {
    fn live_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema>;
}

pub(crate) enum LiveRuntimeCustody<'runtime, Schema> {
    Borrowed(&'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>),
    Shared(Arc<dyn WorthQuerySharedLiveRuntime<Schema>>),
}

impl<Schema> std::ops::Deref for LiveRuntimeCustody<'_, Schema> {
    type Target = WorthQueryPrimaryGraphApplicationRuntime<Schema>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(runtime) => runtime,
            Self::Shared(backing) => backing.live_runtime(),
        }
    }
}

/// Finalizes the managed basis as abandoned when a lease drops unterminated.
///
/// Declared after the lease's subscription, footprint, opening product and
/// initial snapshot, so those release before the basis finalizes.
pub(crate) struct LiveBasisSlot(pub(crate) Option<WorthQueryManagedLowerExecutionBasis>);

impl std::ops::Deref for LiveBasisSlot {
    type Target = Option<WorthQueryManagedLowerExecutionBasis>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LiveBasisSlot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for LiveBasisSlot {
    fn drop(&mut self) {
        if let Some(basis) = self.0.take() {
            let _ = basis
                .bridge
                .finalize(BridgeExecutionBasisTerminalDisposition::Abandoned);
        }
    }
}

impl<'runtime, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryApplicationWholeResultLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    /// Moves this open lease onto shared custody of its own runtime.
    ///
    /// `backing` must hold exactly the runtime the lease was opened on;
    /// otherwise the lease returns unchanged.
    #[allow(clippy::type_complexity)]
    pub fn into_shared(
        self,
        backing: Arc<dyn WorthQuerySharedLiveRuntime<Schema>>,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveLease<
            'static,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        Self,
    > {
        if !std::ptr::eq(backing.live_runtime(), &*self.runtime) {
            return Err(self);
        }
        let Self {
            opening_product,
            subscription,
            footprint,
            initial,
            basis,
            runtime: _,
            termination_failed,
            query,
            scope,
            parameters,
            controls,
            opening_branch,
            opening_incarnation,
            governance,
            read_completion,
            cursor,
            _principal,
            _thread_affinity,
        } = self;
        Ok(WorthQueryApplicationWholeResultLiveLease {
            opening_product,
            subscription,
            footprint,
            initial,
            basis,
            runtime: LiveRuntimeCustody::Shared(backing),
            termination_failed,
            query,
            scope,
            parameters,
            controls,
            opening_branch,
            opening_incarnation,
            governance,
            read_completion,
            cursor,
            _principal,
            _thread_affinity,
        })
    }

    /// Whether this lease reads through exactly `runtime`.
    pub fn belongs_to(&self, runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>) -> bool {
        std::ptr::eq(runtime, &*self.runtime)
    }
}
