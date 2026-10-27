//! The operation really writes the field whose original preimage it retains.
use super::declaration::{MonitorBinding, MonitorInput};
use crate::document_retention_model::{
    retention_entry::{DocumentRetentionWritten, SetRetentionDenial},
    schema::*,
};
use worth_query_host::facade::primary_graph::*;

pub struct Handler;
impl OperationHandler<DocumentRetentionSchema, MonitorBinding> for Handler {
    fn decide(
        &self,
        input: &MonitorInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, MonitorBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        SetRetentionDenial,
    > {
        let mut observe = || {
            let document = reader
                .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())?;
            reader.field(&document, DocumentRetentionField::reference())?;
            reader.mutation_target(&document)
        };
        match observe() {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &MonitorInput,
        target: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, MonitorBinding>,
    ) -> HandlerResult<DocumentRetentionWritten, SetRetentionDenial> {
        let mut apply = || -> Result<(), HandlerExecutionDenial> {
            let document = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(
                    &document,
                    DocumentRetentionField::reference(),
                    input.retention_days,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .emit_external(
                    crate::effect::ArchiveEffect::reference(),
                    SetRetentionInput {
                        identity: input.identity.clone(),
                        retention_days: input.retention_days,
                    },
                )
                .map_err(HandlerExecutionDenial::new)
        };
        match apply() {
            Ok(()) => HandlerResult::Completed(DocumentRetentionWritten {
                retention_days: input.retention_days,
            }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
