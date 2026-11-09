use super::super::{schema::*, workflow::WorkflowDocumentOwner};
use super::declaration::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryInvariantMutationTarget,
};

pub struct Targets {
    context: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
    owner: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Principal>,
    current: Option<WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>>,
}
pub struct ModelLifecycleHandler;
impl OperationHandler<DocumentRetentionSchema, ModelLifecycleBinding> for ModelLifecycleHandler {
    fn decide(
        &self,
        input: &ModelLifecycleInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, ModelLifecycleBinding>,
    ) -> HandlerResult<Targets, ModelLifecycleDenial> {
        match observe(input, reader) {
            Ok(Some(targets)) => HandlerResult::Completed(targets),
            Ok(None) => HandlerResult::DomainDenied(ModelLifecycleDenial::WrongState),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &ModelLifecycleInput,
        targets: Targets,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, ModelLifecycleBinding>,
    ) -> HandlerResult<ModelLifecycleChanged, ModelLifecycleDenial> {
        match apply(input, targets, writer) {
            Ok(()) => HandlerResult::Completed(ModelLifecycleChanged),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

fn observe(
    input: &ModelLifecycleInput,
    reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, ModelLifecycleBinding>,
) -> Result<Option<Targets>, HandlerExecutionDenial> {
    let current = reader.resolve_optional_entity(
        DocumentIdentityField::at("DocumentIdentityField"),
        input.identity.clone(),
    )?;
    if input.restore == current.is_some() || input.context == input.identity {
        return Ok(None);
    }
    if let Some(current) = &current {
        reader
            .reader()
            .require_decision_entity(current, Document::reference())
            .map_err(HandlerExecutionDenial::new)?;
    }
    let context = reader.resolve_entity(
        DocumentIdentityField::at("DocumentIdentityField"),
        input.context.clone(),
    )?;
    let owner =
        reader.resolve_entity(PrincipalIdentityField::at("PrincipalIdentityField"), 1_u64)?;
    reader
        .reader()
        .require_decision_entity(&context, Document::reference())
        .map_err(HandlerExecutionDenial::new)?;
    reader
        .reader()
        .require_decision_entity(&owner, Principal::reference())
        .map_err(HandlerExecutionDenial::new)?;
    if let Some(current) = &current {
        reader
            .reader()
            .require_decision_relation(WorkflowDocumentOwner::reference(), &owner, current)
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(Some(Targets {
        context: reader.mutation_target(&context)?,
        owner: reader.mutation_target(&owner)?,
        current: current
            .map(|current| reader.mutation_target(&current))
            .transpose()?,
    }))
}

fn apply(
    input: &ModelLifecycleInput,
    targets: Targets,
    writer: &mut CandidateWriter<'_, DocumentRetentionSchema, ModelLifecycleBinding>,
) -> Result<(), HandlerExecutionDenial> {
    if let Some(current) = targets.current {
        let current = writer
            .projected_entity(&current)
            .map_err(HandlerExecutionDenial::new)?;
        let owner = writer
            .projected_entity(&targets.owner)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .unlink(WorkflowDocumentOwner::reference(), &owner, &current)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .delete_entity(Document::reference(), &current)
            .map_err(HandlerExecutionDenial::new)?;
    } else {
        let context = writer
            .projected_entity(&targets.context)
            .map_err(HandlerExecutionDenial::new)?;
        let owner = writer
            .projected_entity(&targets.owner)
            .map_err(HandlerExecutionDenial::new)?;
        let created = writer
            .create_entity_in_context(
                &context,
                Document::reference(),
                WorthQueryApplicationEntityKey::new("recreated-document-model")
                    .map_err(HandlerExecutionDenial::new)?,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(
                &created,
                DocumentIdentityField::at("DocumentIdentityField"),
                input.identity.clone(),
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(
                &created,
                DocumentRetentionField::at("DocumentRetentionField"),
                7_u64,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                WorkflowDocumentOwner::reference(),
                "recreated-document-owner",
                &owner,
                &created,
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}
