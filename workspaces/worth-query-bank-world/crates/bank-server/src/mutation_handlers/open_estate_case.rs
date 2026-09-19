use bank_domain::{
    estate::{EstateAction, EstateCaseStatus},
    schema::{
        BankSchema, EstateCaseIdentityField, EstateCaseStatusField, EstateMutationDenial,
        EstateMutationResult, OpenEstateCaseMutationBinding,
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

pub(crate) struct OpenEstateCaseHandler;

impl OperationHandler<BankSchema, OpenEstateCaseMutationBinding> for OpenEstateCaseHandler {
    fn decide(
        &self,
        input: &EstateAction,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, OpenEstateCaseMutationBinding>,
    ) -> HandlerResult<EstateAction, EstateMutationDenial> {
        let command =
            match crate::estate_progression::open_estate_case::case_opening_command(*input) {
                Ok(command) => command,
                Err(_) => return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant),
            };
        let estate = reader.scope().clone();
        match crate::estate_progression::open_estate_case::project_case_opening(
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
        OpenEstateCaseMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &EstateAction,
        decision: EstateAction,
        candidate: &mut CandidateWriter<'_, BankSchema, OpenEstateCaseMutationBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let EstateAction::OpenEstateCase { estate, .. } = decision else {
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
            EstateCaseStatus::Open,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        HandlerResult::Completed(EstateMutationResult { estate })
    }
}
