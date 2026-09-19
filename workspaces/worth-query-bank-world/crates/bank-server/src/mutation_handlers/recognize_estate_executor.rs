use bank_domain::{
    estate::EstateAction,
    schema::{
        BankSchema, EstateCaseIdentityField, EstateExecutor, EstateMutationDenial,
        EstateMutationResult, PrincipalIdentityField, RecognizeEstateExecutorMutationBinding,
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

pub(crate) struct RecognizeEstateExecutorHandler;

impl OperationHandler<BankSchema, RecognizeEstateExecutorMutationBinding>
    for RecognizeEstateExecutorHandler
{
    fn decide(
        &self,
        input: &EstateAction,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, RecognizeEstateExecutorMutationBinding>,
    ) -> HandlerResult<EstateAction, EstateMutationDenial> {
        let command =
            match crate::estate_progression::recognize_executor::recognition_command(*input) {
                Ok(command) => command,
                Err(_) => return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant),
            };
        let estate = reader.scope().clone();
        match crate::estate_progression::recognize_executor::project_recognition(
            reader.reader(),
            &estate,
            command,
        ) {
            Ok(()) => HandlerResult::Completed(*input),
            Err(denial) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial)),
        }
    }

    fn candidate_requirements(
        &self,
        _: &EstateAction,
        _: &EstateAction,
    ) -> ApplicationCandidateRequirements {
        RecognizeEstateExecutorMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &EstateAction,
        decision: EstateAction,
        candidate: &mut CandidateWriter<'_, BankSchema, RecognizeEstateExecutorMutationBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let EstateAction::RecognizeExecutor {
            estate, executor, ..
        } = decision
        else {
            return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant);
        };
        let estate_entity =
            match candidate.resolve_entity(EstateCaseIdentityField::reference(), estate) {
                Ok(entity) => entity,
                Err(denial) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
                }
            };
        let executor_entity =
            match candidate.resolve_entity(PrincipalIdentityField::reference(), executor) {
                Ok(entity) => entity,
                Err(denial) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
                }
            };
        if let Err(denial) = candidate.link(
            EstateExecutor::reference(),
            format!("estate-executor:{}:{}", estate.get(), executor.get()),
            &executor_entity,
            &estate_entity,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        HandlerResult::Completed(EstateMutationResult { estate })
    }
}
