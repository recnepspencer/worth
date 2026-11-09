//! Immutable original root delivery is separate from final Ready authority.
use super::*;
use std::sync::Arc;

pub(super) enum OriginalProgramResult<Q> {
    Disabled,
    Awaiting,
    Retained {
        result: Arc<WorthQueryOutputDemandSettlement>,
        source: crate::domain_computation::primary_graph::WorthQueryObservedSource<Q>,
    },
}
impl<
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        D: WorthQueryApplicationOutputDemand<S>,
    > WorthQueryAdmittedProgramOutput<S, P, D>
{
    pub(super) fn retain_original_result(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if matches!(self.original_result, OriginalProgramResult::Awaiting)
            && let Some(original) = self
                .admitted
                .observe_own_completed_publication(execution, runtime)?
        {
            self.original_result = OriginalProgramResult::Retained {
                result: original,
                source: self.admitted.observed_source().clone(),
            };
        }
        Ok(())
    }

    /// Immutable original evidence only. It supplies no parent, Current,
    /// fulfillment or cleanup authority; those continue to use final Ready.
    pub fn original_root_result(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Option<(
        Arc<WorthQueryOutputDemandSettlement>,
        crate::domain_computation::primary_graph::WorthQueryObservedSource<SourceQuery<S, D>>,
    )> {
        match &self.original_result {
            OriginalProgramResult::Retained { result, source } => {
                Some((Arc::clone(result), source.clone()))
            }
            _ => None,
        }
    }
}
