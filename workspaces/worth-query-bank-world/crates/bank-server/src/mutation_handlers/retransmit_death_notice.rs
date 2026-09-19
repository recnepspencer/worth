use bank_domain::{
    estate::EstateAction,
    schema::{
        BankSchema, EstateDeathNotificationEffect, EstateMutationDenial, EstateMutationResult,
        RetransmitEstateDeathNoticeMutationBinding,
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

pub(crate) struct RetransmitEstateDeathNoticeHandler;

impl OperationHandler<BankSchema, RetransmitEstateDeathNoticeMutationBinding>
    for RetransmitEstateDeathNoticeHandler
{
    fn decide(
        &self,
        input: &EstateAction,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            BankSchema,
            RetransmitEstateDeathNoticeMutationBinding,
        >,
    ) -> HandlerResult<EstateAction, EstateMutationDenial> {
        let command =
            match crate::estate_progression::retransmit_death_notice::retransmit_command(*input) {
                Ok(command) => command,
                Err(_) => return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant),
            };
        let estate = reader.scope().clone();
        match crate::estate_progression::retransmit_death_notice::project_retransmit(
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
        RetransmitEstateDeathNoticeMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &EstateAction,
        decision: EstateAction,
        candidate: &mut CandidateWriter<'_, BankSchema, RetransmitEstateDeathNoticeMutationBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let EstateAction::RetransmitDeathNotice {
            estate,
            notice,
            subject,
        } = decision
        else {
            return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant);
        };
        if let Err(denial) = candidate.emit_external(
            EstateDeathNotificationEffect::reference(),
            bank_domain::estate::EstateDeathNotificationRequest::new(estate, notice, subject),
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        HandlerResult::Completed(EstateMutationResult { estate })
    }
}
