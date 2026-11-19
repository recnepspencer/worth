#![allow(
    clippy::result_large_err,
    reason = "A refused Home open hands back the original request owner without allocating on failure"
)]
//! One Directory Bank Home location reopened across fresh processes of the same world.
use super::*;

pub struct DirectoryWorld {
    location: std::path::PathBuf,
    identities: [DynamicIdentity; 3],
}
impl DirectoryWorld {
    pub fn new() -> Self {
        let identities = ["sender", "recipient", "teller"].map(DynamicIdentity::new);
        let location =
            std::env::temp_dir().join(format!("bank-correction-{}", identities[0].subject));
        Self {
            location,
            identities,
        }
    }
    pub fn open(&self) -> BankHome {
        let request = bank_server::bank_application_home(
            worth_query_host::facade::WorthQueryApplicationHomeLocation::Directory(
                self.location.clone(),
            ),
            seeded_world(InstitutionId::new(91).unwrap(), &self.identities, true),
        )
        .unwrap();
        worth_query_host::facade::application_contribution::with_bootstrap_advancement(
            home_policy(),
            |phase| request.open_in_advancement(&phase),
        )
        .unwrap()
        .unwrap_or_else(|cause| panic!("Bank Home open: {cause:?}"))
    }
    pub fn authenticate(&self, home: &BankHome) -> [bank_server::BankAuthenticatedPrincipal; 3] {
        use crate::facade::support::{
            authentication_configuration, CausalBankAuthenticationAdapter,
        };
        let bank = bank_server::BankHomeRuntime::attach(home.runtime()).unwrap();
        let authentication = bank
            .admit_authentication_adapter(
                authentication_configuration(),
                CausalBankAuthenticationAdapter,
            )
            .unwrap();
        std::array::from_fn(|i| {
            block_on(bank.authenticate_with(
                &authentication,
                CausalCredential::for_identity(&self.identities[i]),
                &request_scope(),
            ))
            .unwrap()
        })
    }
    pub fn send(
        home: &BankHome,
        owner: &bank_server::BankAuthenticatedPrincipal,
        amount: i64,
        command: &str,
    ) -> Sent {
        use bank_server::{BankApplication, SendMoneyAction};
        let scope = request_scope();
        let bank = bank_server::BankHomeRuntime::attach(home.runtime()).unwrap();
        let request = bank.request(owner, &scope);
        let operation = request
            .mutate(SendMoney {
                from: AccountId::new(10).unwrap(),
                recipient: BankPrincipalId::new(2).unwrap(),
                amount: Money::from_minor(amount).unwrap(),
            })
            .without_source()
            .idempotency_owned(key(command))
            .unwrap_or_else(|(_, _, cause)| panic!("send capture: {cause:?}"))
            .retain_durable_operation::<BankApplication, SendMoneyAction>(&request, home.runtime())
            .unwrap_or_else(|(_, cause)| panic!("send owner: {cause:?}"));
        match operation.advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Completed(done) => done,
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
                panic!("send refused: {denial:?}")
            }
            WorthQuerySourceOnlyAdvance::Progress(operation) => {
                panic!("send progress: {:?}", operation.posture())
            }
            WorthQuerySourceOnlyAdvance::Terminal(operation) => {
                panic!("send terminal: {:?}", operation.native_outcome())
            }
        }
    }
    pub fn balances(
        home: &BankHome,
        principals: &[bank_server::BankAuthenticatedPrincipal; 3],
    ) -> (i64, i64) {
        let bank = bank_server::BankHomeRuntime::attach(home.runtime()).unwrap();
        let balance = |account, principal: &bank_server::BankAuthenticatedPrincipal| {
            bank.request(principal, &request_scope())
                .query(bank_domain::queries::account_summary(account))
                .execute()
                .unwrap()
                .into_rows()[0]
                .current_balance()
                .minor_units()
        };
        (
            balance(AccountId::new(10).unwrap(), &principals[0]),
            balance(AccountId::new(11).unwrap(), &principals[1]),
        )
    }
}

impl Default for DirectoryWorld {
    fn default() -> Self {
        Self::new()
    }
}
