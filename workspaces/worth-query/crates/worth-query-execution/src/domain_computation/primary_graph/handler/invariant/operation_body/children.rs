//! Allocate bounded custody before candidate-created handles exist; no authority is fabricated.
use super::{
    ApplicationMutationBinding, ApplicationSchema, DecisionReader, HandlerExecutionDenial,
};
use crate::domain_computation::primary_graph::{
    operation_body::PreparedBodyChildren, WorthQueryAdvancementDenial,
    WorthQueryApplicationEffectEntity, WorthQueryManagedComputationResourceDenial,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_child_custody(
        &self,
        body: &ApplicationOperationBodyProgram,
    ) -> Result<Option<PreparedBodyChildren<S>>, HandlerExecutionDenial> {
        let count = body
            .creation()
            .into_iter()
            .flat_map(|c| c.creation_roots())
            .map(|c| c.children().len())
            .sum::<usize>();
        if count == 0 {
            return Ok(None);
        }
        let bytes = count
            .checked_mul(size_of::<WorthQueryApplicationEffectEntity<S, ()>>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| {
                HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))
            })?;
        let memory = self.execution.reserve(bytes).map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
        })?;
        Ok(Some(PreparedBodyChildren {
            targets: Vec::with_capacity(count),
            _memory: memory,
        }))
    }
}
