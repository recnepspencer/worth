use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, InstitutionId, Money,
};
use bank_domain::proposals::{
    BankIdempotencyKey, BankOperationScopeBinding, BankOperationScopeEntityBinding,
    BankOperationScopeSchemaBinding, BankProposalEngine, BankSnapshot, BankSnapshotBuilder,
};
use bank_domain::schema::{AccountStatus, ApplyOpeningFunding, Deposit};
use bank_server::{BankPrincipalSeed, BankWorldSeed};
use serde::Deserialize;
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

mod estate;
pub use estate::{
    BankHttpProcessEstateAftermathWorld, BankHttpProcessEstateElevationWorld,
    BankHttpProcessEstateWorld,
};

#[derive(Deserialize)]
pub struct BankHttpProcessWorld {
    pub institution: u64,
    pub institution_cash_account: String,
    pub participants: Vec<BankHttpProcessParticipant>,
    pub estate: Option<BankHttpProcessEstateWorld>,
}

#[derive(Deserialize)]
pub struct BankHttpProcessParticipant {
    pub principal: u64,
    pub external_subject: String,
    pub account: BankHttpProcessAccount,
}

#[derive(Deserialize)]
pub struct BankHttpProcessAccount {
    pub identity: String,
    pub display_name: String,
    pub status: BankHttpProcessAccountStatus,
    #[serde(default)]
    pub activity_amounts_minor: Vec<i64>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpProcessAccountStatus {
    Open,
    Frozen,
    Closed,
}

impl BankHttpProcessWorld {
    pub(super) fn build(self) -> Result<BankWorldSeed, ()> {
        let institution = InstitutionId::new(self.institution).ok_or(())?;
        let cash = AccountId::parse_canonical_text(&self.institution_cash_account).ok_or(())?;
        let mut snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).ok_or(())?)
            .institution(institution)
            .institution_cash_account(cash, institution);
        let mut seeds = Vec::with_capacity(self.participants.len());
        let mut funding = Vec::with_capacity(self.participants.len());
        for participant in self.participants {
            let principal = BankPrincipalId::new(participant.principal).ok_or(())?;
            let account =
                AccountId::parse_canonical_text(&participant.account.identity).ok_or(())?;
            let name = AccountName::new(participant.account.display_name).map_err(|_| ())?;
            snapshot = snapshot.principal(principal).personal_account(
                account,
                institution,
                principal,
                name,
                participant.account.status.into_domain(),
            );
            funding.push((account, participant.account.activity_amounts_minor));
            let external = WorthQueryExternalPrincipalIdentity::new(
                crate::AuthentikIssuer,
                participant.external_subject,
            )
            .map_err(|_| ())?;
            seeds.push(BankPrincipalSeed::enabled(principal, external));
        }
        let mut snapshot = snapshot.build().map_err(|_| ())?;
        for (account_ordinal, (account, amounts)) in funding.into_iter().enumerate() {
            for (activity_ordinal, amount) in amounts.into_iter().enumerate() {
                snapshot = apply_activity(
                    snapshot,
                    SeedActivity {
                        institution,
                        account,
                        account_ordinal,
                        activity_ordinal,
                        amount,
                    },
                )?;
            }
        }
        let mut seed = seeds
            .into_iter()
            .fold(BankWorldSeed::new(snapshot), BankWorldSeed::principal);
        if let Some(estate) = self.estate {
            let installed = estate.build(institution)?;
            seed = installed
                .employees
                .into_iter()
                .fold(seed, BankWorldSeed::employee)
                .estate(installed.world);
        }
        Ok(seed)
    }
}

struct SeedActivity {
    institution: InstitutionId,
    account: AccountId,
    account_ordinal: usize,
    activity_ordinal: usize,
    amount: i64,
}

fn apply_activity(snapshot: BankSnapshot, activity: SeedActivity) -> Result<BankSnapshot, ()> {
    let amount = Money::from_minor(activity.amount).map_err(|_| ())?;
    let key = BankIdempotencyKey::new(format!(
        "process-seed-{}-{}",
        activity.account_ordinal, activity.activity_ordinal
    ))
    .map_err(|_| ())?;
    let input = ApplyOpeningFunding {
        institution: activity.institution,
        account: activity.account,
        amount,
    };
    let proposal = if activity.activity_ordinal == 0 {
        BankProposalEngine::prepare_opening_funding(
            &snapshot,
            operation_binding(activity.account, activity.activity_ordinal),
            &key,
            &input,
        )
    } else {
        BankProposalEngine::prepare_deposit(
            &snapshot,
            operation_binding(activity.account, activity.activity_ordinal),
            &key,
            &Deposit {
                institution: activity.institution,
                account: activity.account,
                amount,
            },
        )
    }
    .map_err(|_| ())?;
    Ok(proposal.proposed_snapshot().clone())
}

fn operation_binding(account: AccountId, ordinal: usize) -> BankOperationScopeBinding {
    BankOperationScopeBinding::new(
        1,
        BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
        "bank-http-process-seed",
        BankOperationScopeEntityBinding::new(0, 1, 1),
        BankOperationScopeEntityBinding::new(
            0,
            account.canonical_text().len() as u64 + ordinal as u64,
            1,
        ),
    )
}

impl BankHttpProcessAccountStatus {
    const fn into_domain(self) -> AccountStatus {
        match self {
            Self::Open => AccountStatus::Open,
            Self::Frozen => AccountStatus::Frozen,
            Self::Closed => AccountStatus::Closed,
        }
    }
}
