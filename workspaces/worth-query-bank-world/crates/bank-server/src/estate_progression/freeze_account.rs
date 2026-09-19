use bank_domain::{
    estate::EstateAction,
    model::AccountId,
    proposals::BankIdempotencyKey,
    schema::{
        AccountIdentity, AccountStatus, BankSchema, EstateAccount, EstateCase,
        FreezeEstateAccountOperation, Status,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    primary_graph::{
        WorthQueryApplicationOperationInvariantProjectionReader,
        WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
        WorthQueryInvariantProjectionTraversalDenial,
    },
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

#[derive(Debug)]
pub enum BankEstateFreezeProjectionDenial {
    RelationCardinality { expected: usize, observed: usize },
    MissingAccountIdentity,
    RelatedAccountMismatch,
    MissingAccountStatus,
    AccountNotOpen,
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
}

impl BankIdentityRuntime {
    pub fn freeze_estate_account_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let EstateAction::FreezeAccount { estate, account } = action else {
            return Err(BankEstateProgressionDenial::CommandInput(
                "FreezeEstateAccountOperation",
            ));
        };
        super::program_outcome::program_outcome(
            self.request(principal, request)
                .mutate(bank_domain::schema::FreezeEstateAccount::new(
                    estate, account,
                ))
                .idempotency(key)
                .execute_capability_in_program(self.application_program()),
            "FreezeEstateAccountOperation",
            |denial| {
                denial
                    .downcast::<BankEstateFreezeProjectionDenial>()
                    .map(BankEstateProgressionDenial::FreezeProjection)
            },
        )
    }
}

pub(crate) fn freeze_command_account(
    action: EstateAction,
) -> Result<AccountId, BankEstateProgressionDenial> {
    match action {
        EstateAction::FreezeAccount { account, .. } => Ok(account),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "FreezeEstateAccountOperation",
        )),
    }
}

pub(crate) fn project_freeze_account(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        FreezeEstateAccountOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    expected_account: AccountId,
) -> Result<(), BankEstateFreezeProjectionDenial> {
    let relations = reader.decision_relations_from(EstateAccount::reference(), estate)?;
    let [relation] = relations.as_slice() else {
        return Err(BankEstateFreezeProjectionDenial::RelationCardinality {
            expected: 1,
            observed: relations.len(),
        });
    };
    let account = relation.to().clone();
    let observed_account = reader
        .decision_field(&account, AccountIdentity::reference())?
        .ok_or(BankEstateFreezeProjectionDenial::MissingAccountIdentity)?;
    if observed_account != expected_account {
        return Err(BankEstateFreezeProjectionDenial::RelatedAccountMismatch);
    }
    let status = reader
        .decision_field(&account, Status::reference())?
        .ok_or(BankEstateFreezeProjectionDenial::MissingAccountStatus)?;
    if status != AccountStatus::Open {
        return Err(BankEstateFreezeProjectionDenial::AccountNotOpen);
    }
    Ok(())
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankEstateFreezeProjectionDenial {
    fn from(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            denial.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial> for BankEstateFreezeProjectionDenial {
    fn from(denial: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            denial.kind(),
        ))
    }
}

impl std::fmt::Display for BankEstateFreezeProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RelationCardinality { expected, observed } => write!(
                formatter,
                "estate account relation expected {expected} target, observed {observed}"
            ),
            Self::MissingAccountIdentity => {
                write!(formatter, "estate account is missing its typed identity")
            }
            Self::RelatedAccountMismatch => {
                formatter.write_str("freeze command account does not match estate account")
            }
            Self::MissingAccountStatus => {
                write!(formatter, "estate account is missing its current status")
            }
            Self::AccountNotOpen => formatter.write_str("estate account is not open"),
            Self::DecisionPlan(denial) => denial.fmt(formatter),
            Self::Traversal(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankEstateFreezeProjectionDenial {}
