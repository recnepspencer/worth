use bank_domain::{
    estate::EstateAction,
    schema::{
        AccountIdentity, AccountStatus, BankSchema, EstateMutationDenial, EstateMutationResult,
        FreezeEstateAccountMutationBinding, Status,
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

pub(crate) struct FreezeEstateAccountHandler;

impl OperationHandler<BankSchema, FreezeEstateAccountMutationBinding>
    for FreezeEstateAccountHandler
{
    fn decide(
        &self,
        input: &EstateAction,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, FreezeEstateAccountMutationBinding>,
    ) -> HandlerResult<EstateAction, EstateMutationDenial> {
        let account =
            match crate::estate_progression::freeze_account::freeze_command_account(*input) {
                Ok(account) => account,
                Err(_) => return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant),
            };
        let estate = reader.scope().clone();
        match crate::estate_progression::freeze_account::project_freeze_account(
            reader.reader(),
            &estate,
            account,
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
        FreezeEstateAccountMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &EstateAction,
        decision: EstateAction,
        candidate: &mut CandidateWriter<'_, BankSchema, FreezeEstateAccountMutationBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let EstateAction::FreezeAccount { estate, account } = decision else {
            return HandlerResult::DomainDenied(EstateMutationDenial::InputVariant);
        };
        let account = match candidate.resolve_entity(AccountIdentity::reference(), account) {
            Ok(account) => account,
            Err(denial) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
            }
        };
        if let Err(denial) =
            candidate.write_field(&account, Status::reference(), AccountStatus::Frozen)
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        HandlerResult::Completed(EstateMutationResult { estate })
    }
}
