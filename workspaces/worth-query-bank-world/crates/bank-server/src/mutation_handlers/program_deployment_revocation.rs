//! Authenticated issuer revocation through the ordinary production candidate owner.
use bank_domain::schema::*;
use worth_query_host::facade::{
    declaration::application_operation::{
        ApplicationCandidateRequirements, ApplicationMutationBinding,
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget,
    },
};
pub struct ProgramDeploymentRevocationHandler;

impl OperationHandler<BankSchema, ProgramDeploymentRevocationBinding>
    for ProgramDeploymentRevocationHandler
{
    fn decide(
        &self,
        input: &ProgramDeploymentRevocationInput,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, ProgramDeploymentRevocationBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BankSchema, ProgramDeploymentGrant>,
        ProgramDeploymentRevocationDenial,
    > {
        let grant = match reader.resolve_entity(
            ProgramDeploymentGrantIdentity::reference(),
            input.grant_identity.clone(),
        ) {
            Ok(grant) => grant,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&grant, ProgramDeploymentGrantStatus::reference()) {
            Ok(Some(current)) if current == "revoked" => {
                return HandlerResult::DomainDenied(ProgramDeploymentRevocationDenial::Unchanged)
            }
            Ok(_) => {}
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&grant) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &ProgramDeploymentRevocationInput,
        _: &WorthQueryInvariantMutationTarget<BankSchema, ProgramDeploymentGrant>,
    ) -> ApplicationCandidateRequirements {
        ProgramDeploymentRevocationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _input: &ProgramDeploymentRevocationInput,
        target: WorthQueryInvariantMutationTarget<BankSchema, ProgramDeploymentGrant>,
        writer: &mut CandidateWriter<'_, BankSchema, ProgramDeploymentRevocationBinding>,
    ) -> HandlerResult<ProgramDeploymentRevocationChanged, ProgramDeploymentRevocationDenial> {
        let grant = match writer.projected_entity(&target) {
            Ok(grant) => grant,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.write_field(
            &grant,
            ProgramDeploymentGrantStatus::reference(),
            "revoked".to_owned(),
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(ProgramDeploymentRevocationChanged)
    }
}
