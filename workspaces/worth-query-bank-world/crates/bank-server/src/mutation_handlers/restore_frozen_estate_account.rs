use bank_domain::schema::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};

pub(crate) struct RestoreFrozenEstateAccountHandler;

#[derive(Debug)]
enum RestorationDenial {
    PriorStatusNotOpen,
    AccountRelationChanged,
    AccountNotFrozen,
}
impl std::fmt::Display for RestorationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::PriorStatusNotOpen => "recorded freeze status must be open",
            Self::AccountRelationChanged => "estate account relation changed since the freeze",
            Self::AccountNotFrozen => "estate account is no longer frozen",
        })
    }
}
impl std::error::Error for RestorationDenial {}

impl OperationHandler<BankSchema, RestoreFrozenEstateAccountBinding>
    for RestoreFrozenEstateAccountHandler
{
    fn decide(
        &self,
        input: &RestoreFrozenEstateAccount,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, RestoreFrozenEstateAccountBinding>,
    ) -> HandlerResult<RestoreFrozenEstateAccount, EstateMutationDenial> {
        if input.prior_status != AccountStatus::Open {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                RestorationDenial::PriorStatusNotOpen,
            ));
        }
        let estate = reader.scope().clone();
        let projection = reader.reader();
        let relations =
            match projection.decision_relations_from(EstateAccount::reference(), &estate) {
                Ok(relations) => relations,
                Err(denial) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
                }
            };
        let [relation] = relations.as_slice() else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                RestorationDenial::AccountRelationChanged,
            ));
        };
        let account = relation.to().clone();
        match projection.decision_field(&account, AccountIdentity::reference()) {
            Ok(Some(identity)) if identity == input.account => {}
            Ok(_) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                    RestorationDenial::AccountRelationChanged,
                ))
            }
            Err(denial) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
            }
        }
        match projection.decision_field(&account, Status::reference()) {
            Ok(Some(AccountStatus::Frozen)) => HandlerResult::Completed(*input),
            Ok(_) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                RestorationDenial::AccountNotFrozen,
            )),
            Err(denial) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial)),
        }
    }

    fn build_candidate(
        &self,
        _: &RestoreFrozenEstateAccount,
        decision: RestoreFrozenEstateAccount,
        candidate: &mut CandidateWriter<'_, BankSchema, RestoreFrozenEstateAccountBinding>,
    ) -> HandlerResult<EstateMutationResult, EstateMutationDenial> {
        let account = match candidate.resolve_entity(AccountIdentity::reference(), decision.account)
        {
            Ok(account) => account,
            Err(denial) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial))
            }
        };
        if let Err(denial) =
            candidate.write_field(&account, Status::reference(), decision.prior_status)
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial));
        }
        HandlerResult::Completed(EstateMutationResult {
            estate: decision.estate,
        })
    }
}
