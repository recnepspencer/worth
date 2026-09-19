use bank_domain::{
    estate::{EstateAction, EstateCaseStatus},
    schema::{
        BankSchema, EstateCaseIdentityField, EstateCaseStatusField, EstateExecutor,
        EstateMutationDenial, EstateMutationResult, PrincipalIdentityField,
        ReleaseEstateMutationBinding,
    },
};
use worth_query_host::facade::{
    declaration::application_operation::{
        ApplicationCandidateRequirements, ApplicationMutationBinding,
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    },
};

pub(crate) struct ReleaseEstateHandler;

impl OperationHandler<BankSchema, ReleaseEstateMutationBinding> for ReleaseEstateHandler {
    fn decide(
        &self,
        input: &EstateAction,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, ReleaseEstateMutationBinding>,
    ) -> HandlerResult<EstateAction, EstateMutationDenial> {
        let command = match crate::estate_progression::release_estate::release_command(*input) {
            Ok(command) => command,
            Err(_) => return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant),
        };
        let estate = reader.scope().clone();
        match crate::estate_progression::release_estate::project_release_readiness(
            reader.reader(),
            &estate,
            command,
        ) {
            Ok(()) => {}
            Err(denial) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
            }
        };
        let executor =
            match reader.resolve_entity(PrincipalIdentityField::reference(), command.executor) {
                Ok(executor) => executor,
                Err(denial) => return HandlerResult::ExecutionDenied(denial),
            };
        let relations = match reader.relations_from(EstateExecutor::reference(), &executor) {
            Ok(relations) => relations,
            Err(denial) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
            }
        };
        let observed = relations
            .iter()
            .filter(|relation| relation.to() == &estate)
            .count();
        if observed != 1 {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                crate::BankEstateReleaseProjectionDenial::ExecutorRelationCardinality { observed },
            ));
        }
        HandlerResult::Completed(*input)
    }

    fn candidate_requirements(
        &self,
        _: &EstateAction,
        _: &EstateAction,
    ) -> ApplicationCandidateRequirements {
        ReleaseEstateMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &EstateAction,
        decision: EstateAction,
        candidate: &mut CandidateWriter<'_, BankSchema, ReleaseEstateMutationBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let EstateAction::ReleaseEstate { estate, .. } = decision else {
            return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant);
        };
        let estate_entity =
            match candidate.resolve_entity(EstateCaseIdentityField::reference(), estate) {
                Ok(entity) => entity,
                Err(denial) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
                }
            };
        if let Err(denial) = candidate.write_field(
            &estate_entity,
            EstateCaseStatusField::reference(),
            EstateCaseStatus::Released,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        HandlerResult::Completed(EstateMutationResult { estate })
    }
}
