//! Principal lifecycle effects are ordinary declared graph mutations.
//! Native retention alone decides when a retired slot can be reused.
use super::super::{
    schema::*,
    workflow::{
        WorkflowAuthoringGrant, WorkflowDocumentOwner, WorkflowGrantGrantee, WorkflowGrantGrantor,
        WorkflowGrantIdentityField,
    },
};
use super::{binding::ActorLifecycleBinding, declaration::*};
use worth_query_host::facade::{
    declaration::application_operation::ApplicationCandidateRequirements,
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget,
    },
};
#[path = "handler/candidate.rs"]
mod candidate;

pub struct Targets {
    document: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
    mapping: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, ExternalMapping>,
    grants: Vec<WorthQueryInvariantMutationTarget<DocumentRetentionSchema, WorkflowAuthoringGrant>>,
    actor: Option<WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Principal>>,
}
pub struct ActorLifecycleHandler;
impl OperationHandler<DocumentRetentionSchema, ActorLifecycleBinding> for ActorLifecycleHandler {
    fn decide(
        &self,
        input: &ActorLifecycleInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, ActorLifecycleBinding>,
    ) -> HandlerResult<Targets, ActorLifecycleDenial> {
        match observe(input, reader) {
            Ok(Some(targets)) => HandlerResult::Completed(targets),
            Ok(None) => HandlerResult::DomainDenied(ActorLifecycleDenial::WrongState),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn candidate_requirements(
        &self,
        _: &ActorLifecycleInput,
        _: &Targets,
    ) -> ApplicationCandidateRequirements {
        <ActorLifecycleBinding as worth_query_decl::facade::application_operation::ApplicationMutationBinding<DocumentRetentionSchema>>::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &ActorLifecycleInput,
        targets: Targets,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, ActorLifecycleBinding>,
    ) -> HandlerResult<ActorLifecycleChanged, ActorLifecycleDenial> {
        match candidate::apply(input, targets, writer) {
            Ok(()) => HandlerResult::Completed(ActorLifecycleChanged),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
fn observe(
    input: &ActorLifecycleInput,
    reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, ActorLifecycleBinding>,
) -> Result<Option<Targets>, HandlerExecutionDenial> {
    let actor = reader.resolve_optional_entity(PrincipalIdentityField::reference(), 1_u64)?;
    if input.restore == actor.is_some() {
        return Ok(None);
    }
    let document =
        reader.resolve_entity(DocumentIdentityField::reference(), input.identity.clone())?;
    let mapping = reader.resolve_entity(
        ExternalIdentityField::reference(),
        super::super::operator_identity::operator_identity(),
    )?;
    // The mapping replacement must carry the actual observed field preimage
    // through native effect lowering, not just the mapping's entity identity.
    let expected_status = if input.restore {
        worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus::Disabled
    } else {
        worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus::Enabled
    };
    if reader.field(&mapping, MappingStatusField::reference())? != Some(expected_status) {
        return Ok(None);
    }
    let mut grants = Vec::new();
    for key in [
        "workflow-authoring-grant",
        "workflow-instance-start-grant",
        "workflow-advance-grant",
        "workflow-approval-grant",
    ] {
        let grant =
            reader.resolve_entity(WorkflowGrantIdentityField::reference(), key.to_owned())?;
        if let Some(actor) = &actor {
            reader
                .reader()
                .require_decision_relation(WorkflowGrantGrantor::reference(), actor, &grant)
                .map_err(HandlerExecutionDenial::new)?;
            reader
                .reader()
                .require_decision_relation(WorkflowGrantGrantee::reference(), actor, &grant)
                .map_err(HandlerExecutionDenial::new)?;
        }
        grants.push(reader.mutation_target(&grant)?);
    }
    if let Some(actor) = &actor {
        reader
            .reader()
            .require_decision_relation(MappingTarget::reference(), &mapping, actor)
            .map_err(HandlerExecutionDenial::new)?;
        reader
            .reader()
            .require_decision_relation(WorkflowDocumentOwner::reference(), actor, &document)
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(Some(Targets {
        document: reader.mutation_target(&document)?,
        mapping: reader.mutation_target(&mapping)?,
        grants,
        actor: actor
            .map(|actor| reader.mutation_target(&actor))
            .transpose()?,
    }))
}
