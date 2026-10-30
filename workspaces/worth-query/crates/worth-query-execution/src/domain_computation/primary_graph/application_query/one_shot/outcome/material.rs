//! Admission at the actual retained-source completion allocation sites.
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation::primary_graph::application_query) struct MaterialCompletion<
    'a,
    'request,
    'authority,
> {
    execution: &'a ExecutionRequest<'request, 'authority>,
    allocation: &'a Arc<SourceFactsAllocation>,
}

impl<'a, 'request, 'authority> MaterialCompletion<'a, 'request, 'authority> {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        execution: &'a ExecutionRequest<'request, 'authority>,
        allocation: &'a Arc<SourceFactsAllocation>,
    ) -> Self {
        Self {
            execution,
            allocation,
        }
    }
    pub(super) fn bytes(&self, bytes: u64) -> Result<(), Cause> {
        self.allocation.claim(self.execution, bytes)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn work(
        &self,
        units: u64,
    ) -> Result<(), Cause> {
        let mut result = None;
        let outcome = self.execution.run_native_with_checkpoint(|context| {
            result = Some(if context.has_accounting() {
                context.checkpoint(units)
            } else {
                context.poll()
            });
        });
        match (outcome, result) {
            (Ok(_), Some(result)) => result.map_err(Into::into),
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(cause.into()),
            (Err(cause), _) => Err(cause.into()),
            (Ok(_), None) => unreachable!("completed source metadata checkpoint"),
        }
    }
    pub(super) fn slots<T>(&self, count: usize) -> Result<(), Cause> {
        self.allocation.claim_slice::<T>(self.execution, count)
    }
    pub(super) fn original_metadata(
        &self,
        query: &str,
        branch: &worth_relational::facade::history::BranchId,
    ) -> Result<(), Cause> {
        self.arc::<super::super::super::observed_source::WorthQueryRetainedObservedParameters>()?;
        self.arc::<String>()?;
        self.text(query)?;
        self.arc::<worth_relational::facade::history::BranchId>()?;
        self.text(&branch.0)
    }
    pub(super) fn source_headers<Query>(
        &self,
        count: usize,
    ) -> Result<Vec<super::super::super::WorthQueryObservedSource<Query>>, Cause> {
        self.slots::<super::super::super::WorthQueryObservedSource<Query>>(count)?;
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(count)
            .map_err(|_| Cause::AllocationFailed)?;
        Ok(sources)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn arc<T>(
        &self,
    ) -> Result<(), Cause> {
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<T>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        self.bytes(bytes)
    }
    pub(super) fn text(&self, text: &str) -> Result<(), Cause> {
        let bytes = u64::try_from(text.len()).map_err(|_| Cause::ChargedBytesOverflow)?;
        self.bytes(bytes)?;
        self.work(bytes)
    }
    pub(super) fn affinity<S, P, I>(
        &self,
        principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
            S,
            P,
            I,
        >,
    ) -> Result<(), Cause> {
        use worth_query_admission::facade::authenticated_principal::WorthQueryPrincipalAttribute;
        let attributes = principal.attributes();
        self.work(u64::try_from(attributes.len()).map_err(|_| Cause::WorkCounterOverflow)?)?;
        self.slots::<WorthQueryPrincipalAttribute>(attributes.len())?;
        self.text(principal.external_identity().issuer())?;
        self.text(principal.external_identity().subject())?;
        for attribute in attributes {
            self.text(attribute.key())?;
            self.text(attribute.value())?;
        }
        Ok(())
    }
    pub(super) fn basis(
        &self,
        basis: &super::super::super::WorthQueryApplicationBasisIdentity,
    ) -> Result<(), Cause> {
        let bytes = basis
            .retained_clone_heap_bytes()
            .ok_or(Cause::ChargedBytesOverflow)?;
        self.bytes(bytes)?;
        self.work(bytes)
    }
}

pub(in crate::domain_computation::primary_graph::application_query) fn retain_scope<S, E>(
    execution: &ExecutionRequest<'_, '_>,
    scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<S, E>,
    query: &str,
) -> Result<Arc<crate::domain_computation::primary_graph::entity_resolution::WorthQueryObservedScopeSelector>, super::WorthQueryApplicationOneShotDenial>{
    use super::WorthQueryApplicationOneShotDenialKind as Kind;
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    scope
        .retain_scope_selector_funded(execution)
        .map_err(|cause| {
            super::denial(
                match cause {
                    Ok(cause) => Kind::ExecutionRequest(cause),
                    Err(Control::Cancelled) => Kind::Cancelled,
                    Err(Control::TimedOut) => Kind::DeadlineExceeded,
                },
                query,
                query,
            )
        })
}
