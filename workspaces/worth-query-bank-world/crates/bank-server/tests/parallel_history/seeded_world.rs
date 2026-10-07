//! The same seeded requests and initial funding feed the model and real world.
use super::journal_model::Transfer;
use crate::support::{self, CausalCredential, DynamicIdentity, TestIdentityWorld};
use bank_domain::{model::*, proposals::*, schema::*};
use bank_server::{
    BankAuthenticatedPrincipal, BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed,
};
pub(super) const SEED: u64 = 0x9176_378a_0001;
pub(super) struct World {
    pub world: TestIdentityWorld,
    identities: Vec<DynamicIdentity>,
    pub accounts: [AccountId; 2],
    pub cash: AccountId,
    pub institution: InstitutionId,
    pub funding: i64,
    pub funding_journal: JournalEntryId,
}
pub(super) fn key(command: u64) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("parallel-history-{command}")).unwrap()
}
pub(super) fn principal(number: u64) -> BankPrincipalId {
    BankPrincipalId::new(number).unwrap()
}
impl World {
    pub fn new(seed: u64) -> Self {
        let identities = (1..=4)
            .map(|actor| DynamicIdentity::new(&format!("history-{seed}-{actor}")))
            .collect::<Vec<_>>();
        let institution = InstitutionId::new(1).unwrap();
        let cash = AccountId::new(100).unwrap();
        let mut builder = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
            .institution(institution)
            .institution_cash_account(cash, institution);
        for actor in 1..=4 {
            builder = builder.principal(principal(actor));
        }
        let accounts = [AccountId::new(11).unwrap(), AccountId::new(12).unwrap()];
        for (index, account) in accounts.iter().enumerate() {
            builder = builder.personal_account(
                *account,
                institution,
                principal(index as u64 + 1),
                AccountName::new(format!("Account {index}")).unwrap(),
                AccountStatus::Open,
            );
        }
        // The auditor also has the declared read-only cash-account role so
        // every modeled balance is observable through the client summary door.
        builder = builder.projected_authorization(BankAccountAuthorization::from_projection(
            AccountAuthorizationId::new(1).unwrap(),
            cash,
            principal(4),
            CustomerRole::Viewer,
        ));
        let mut snapshot = builder.build().unwrap();
        let funding = 10_000 + i64::try_from(seed % 1000).unwrap();
        snapshot = BankProposalEngine::prepare_opening_funding(
            &snapshot,
            binding(),
            &key(200),
            &ApplyOpeningFunding {
                institution,
                account: accounts[0],
                amount: Money::from_minor(funding).unwrap(),
            },
        )
        .unwrap()
        .proposed_snapshot()
        .clone();
        let funding_journal = snapshot.journal().last().unwrap().id();
        let mut seed = BankWorldSeed::new(snapshot).employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(1).unwrap(),
            institution,
            principal(4),
            EmployeeRole::Auditor,
        ));
        for (index, identity) in identities.iter().enumerate() {
            seed = seed.principal(BankPrincipalSeed::enabled(
                principal(index as u64 + 1),
                identity.external(),
            ));
        }
        Self {
            world: support::runtime(seed),
            identities,
            accounts,
            cash,
            institution,
            funding,
            funding_journal,
        }
    }
    pub fn authenticate(&self, actor: u64) -> BankAuthenticatedPrincipal {
        support::block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[actor as usize - 1]),
            &support::request_scope(),
        ))
        .unwrap()
    }
    pub fn requests(&self, seed: u64) -> Vec<Transfer> {
        let amount = 1 + i64::try_from((seed >> 16) % 512).unwrap();
        let first = Transfer {
            actor: 1,
            from: 11,
            to: 12,
            amount,
            command: 1,
        };
        vec![
            first.clone(),
            first.clone(),
            Transfer {
                actor: 3,
                command: 2,
                ..first.clone()
            },
            Transfer {
                // Just above the post-transfer balance, but within the original
                // funding: an insufficient-funds check against stale funds admits it.
                amount: self.funding - amount + 1,
                command: 3,
                ..first.clone()
            },
            Transfer {
                amount: amount + 1,
                command: 4,
                ..first
            },
        ]
    }
}
fn binding() -> BankOperationScopeBinding {
    BankOperationScopeBinding::new(
        1,
        BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
        "parallel-history-seed",
        BankOperationScopeEntityBinding::new(0, 1, 1),
        BankOperationScopeEntityBinding::new(0, 1, 1),
    )
}
