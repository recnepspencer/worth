use bank_domain::{
    estate::{DeathNoticeStatus, EstateAction},
    schema::{
        BankSchema, DeathNoticeIdentityField, DeathNoticeStatusField,
        EstateDeathNotificationEffect, EstateMutationDenial, EstateMutationResult,
        NotifyEstateDeathMutationBinding,
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

pub(crate) struct NotifyEstateDeathHandler;

impl OperationHandler<BankSchema, NotifyEstateDeathMutationBinding> for NotifyEstateDeathHandler {
    fn decide(
        &self,
        input: &EstateAction,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, NotifyEstateDeathMutationBinding>,
    ) -> HandlerResult<EstateAction, EstateMutationDenial> {
        let command = match crate::estate_progression::notify_death::notification_command(*input) {
            Ok(command) => command,
            Err(_) => return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant),
        };
        let estate = reader.scope().clone();
        match crate::estate_progression::notify_death::project_notification(
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
        NotifyEstateDeathMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &EstateAction,
        decision: EstateAction,
        candidate: &mut CandidateWriter<'_, BankSchema, NotifyEstateDeathMutationBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let command = match crate::estate_progression::notify_death::notification_command(decision)
        {
            Ok(command) => command,
            Err(_) => return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant),
        };
        let notice =
            match candidate.resolve_entity(DeathNoticeIdentityField::reference(), command.notice) {
                Ok(notice) => notice,
                Err(denial) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
                }
            };
        if let Err(denial) = candidate.write_field(
            &notice,
            DeathNoticeStatusField::reference(),
            DeathNoticeStatus::NotificationRequested,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        if let Err(denial) = candidate.emit_external(
            EstateDeathNotificationEffect::reference(),
            bank_domain::estate::EstateDeathNotificationRequest::new(
                command.estate,
                command.notice,
                command.subject,
            ),
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        HandlerResult::Completed(EstateMutationResult {
            estate: command.estate,
        })
    }
}
