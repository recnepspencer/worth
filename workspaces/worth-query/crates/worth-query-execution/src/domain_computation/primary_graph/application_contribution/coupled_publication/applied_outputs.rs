//! Prerequisite lookup borrows only writers appended by this actual assembly.
use super::*;
use crate::domain_computation::primary_graph::{
    invariant_projection::{
        WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind as Kind,
    },
    provider::AppliedOutputParticipants,
};
impl<S: ApplicationSchema> AppliedOutputParticipants for CoupledApplicationAssembly<'_, '_, '_, S> {
    fn visit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        family: &str,
        source: worth_relational::facade::identity::EntityId,
        consume: &mut dyn FnMut(
            crate::domain_computation::WorthQueryProviderSessionView<'_>,
        ) -> Result<(), WorthQueryCurrentOutputDenial>,
    ) -> Result<(), WorthQueryCurrentOutputDenial> {
        for writer in self.writers.borrow().iter() {
            // This is the actual candidate roster, never the provider's live population.
            // Charge narrowing itself; no hidden provider-wide prefilter or second index.
            execution
                .run_native_with_checkpoint(|context| {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context,
                        (family.len() + writer.meaning.family.len() + 1) as u64,
                    )
                })
                .map_err(|cause| {
                    WorthQueryCurrentOutputDenial::new(Kind::ExecutionRequest(cause.into()), family)
                })?
                .0
                .map_err(|cause| {
                    WorthQueryCurrentOutputDenial::new(Kind::ExecutionRequest(cause.into()), family)
                })?;
            if writer.meaning.source_scope != source || writer.meaning.family != family {
                continue;
            }
            let Some(CoupledWriterStage::Inspected(writer)) = writer.stage.as_ref() else {
                return Err(WorthQueryCurrentOutputDenial::new(
                    Kind::OutputUnavailable,
                    family,
                ));
            };
            consume(writer.session())?;
        }
        Ok(())
    }
}
