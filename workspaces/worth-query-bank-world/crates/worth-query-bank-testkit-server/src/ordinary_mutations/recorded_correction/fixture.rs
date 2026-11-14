pub use super::*;
pub use crate::facade::support::{
    block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};
pub use bank_domain::{
    model::{AccountName, BankSnapshotVersion, EmployeeAssignmentId, EmployeeRole, InstitutionId},
    proposals::BankSnapshotBuilder,
    schema::AccountStatus,
};
pub use bank_server::{BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed};

pub struct World {
    pub native: TestIdentityWorld,
    pub owner: bank_server::BankAuthenticatedPrincipal,
    pub recipient: bank_server::BankAuthenticatedPrincipal,
    pub teller: bank_server::BankAuthenticatedPrincipal,
    pub source: AccountId,
    pub destination: AccountId,
    pub recipient_id: BankPrincipalId,
}

impl World {
    pub fn new(institution_number: u64) -> Self {
        let institution = InstitutionId::new(institution_number).unwrap();
        let ids = [1, 2, 3].map(|id| BankPrincipalId::new(id).unwrap());
        let identities = ["sender", "recipient", "teller"]
            .map(|name| DynamicIdentity::new(&format!("correction-{institution_number}-{name}")));
        let [source, destination] = [10, 11].map(|id| AccountId::new(id).unwrap());
        let seed = seeded_world(institution, &identities, false);
        let native = runtime(seed);
        let [owner, recipient, teller] = identities.map(|identity| {
            block_on(native.runtime.authenticate_with(
                &native.authentication,
                CausalCredential::for_identity(&identity),
                &request_scope(),
            ))
            .unwrap()
        });
        let scope = request_scope();
        let request = native.runtime.request(&teller, &scope);
        let funding = native
            .runtime
            .mutate(mutations::apply_opening_funding(ApplyOpeningFunding {
                institution,
                account: source,
                amount: Money::from_minor(100).unwrap(),
            }))
            .as_principal(&teller)
            .controls(BankMutationControls::new(scope.clone(), key("opening")))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("funding: {cause:?}"));
        let WorthQuerySourceOnlyAdvance::Completed(done) = funding.advance(&request, None) else {
            panic!("ordinary opening funding");
        };
        assert!(done.close(&request).is_ok());
        let world = Self {
            native,
            owner,
            recipient,
            teller,
            source,
            destination,
            recipient_id: ids[1],
        };
        world.grant_view("initial-view");
        world
    }

    pub fn grant_view(&self, command: &str) -> bank_domain::model::AccountAuthorizationId {
        let scope = request_scope();
        let request = self.native.runtime.request(&self.owner, &scope);
        let operation = self
            .native
            .runtime
            .mutate(mutations::grant_account_access(
                bank_domain::schema::GrantAccountAuthorization {
                    account: self.source,
                    principal: BankPrincipalId::new(3).unwrap(),
                    role: bank_domain::model::CustomerRole::Viewer,
                },
            ))
            .as_principal(&self.owner)
            .controls(BankMutationControls::new(scope.clone(), key(command)))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("view grant: {cause:?}"));
        let WorthQuerySourceOnlyAdvance::Completed(done) = operation.advance(&request) else {
            panic!("ordinary view grant completion");
        };
        let WorthQuerySourceCompletion::Fresh { result, .. } = done.evidence() else {
            panic!("fresh view grant");
        };
        let authorization = result.authorization;
        assert!(done.close(&request).is_ok());
        authorization
    }

    pub fn revoke_view(&self) {
        let authorization = self
            .native
            .runtime
            .query(queries::account_authorized_users(self.source))
            .as_principal(&self.owner)
            .controls(read_controls())
            .execute()
            .unwrap()
            .into_rows()[0]
            .users()
            .iter()
            .find(|user| user.principal() == BankPrincipalId::new(3).unwrap())
            .unwrap()
            .authorization();
        let scope = request_scope();
        let request = self.native.runtime.request(&self.owner, &scope);
        let operation = self
            .native
            .runtime
            .mutate(mutations::revoke_account_access(
                bank_domain::schema::RevokeAccountAuthorization {
                    account: self.source,
                    authorization,
                },
            ))
            .as_principal(&self.owner)
            .controls(BankMutationControls::new(scope.clone(), key("revoke-view")))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("view revocation: {cause:?}"));
        let WorthQuerySourceOnlyAdvance::Completed(done) = operation.advance(&request) else {
            panic!("ordinary view revocation completion");
        };
        assert!(done.close(&request).is_ok());
    }

    pub fn send(
        &self,
        amount: i64,
        command: &str,
    ) -> (WorthQueryRecordedCorrectionSource, JournalEntryId) {
        let scope = request_scope();
        let request = self.native.runtime.request(&self.owner, &scope);
        let source = self
            .native
            .runtime
            .mutate(mutations::send_money(SendMoney {
                from: self.source,
                recipient: self.recipient_id,
                amount: Money::from_minor(amount).unwrap(),
            }))
            .as_principal(&self.owner)
            .controls(BankMutationControls::new(scope.clone(), key(command)))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("send: {cause:?}"));
        let WorthQuerySourceOnlyAdvance::Completed(done) = source.advance(&request, None) else {
            panic!("ordinary source completion");
        };
        let WorthQuerySourceCompletion::Fresh {
            receipt, result, ..
        } = done.evidence()
        else {
            panic!("fresh send result");
        };
        let coordinates = receipt.correction_source();
        let journal = result.journal;
        assert!(done.close(&request).is_ok());
        (coordinates, journal)
    }

    pub fn balances(&self) -> (i64, i64) {
        let balance = |account, principal| {
            self.native
                .runtime
                .query(queries::account_summary(account))
                .as_principal(principal)
                .controls(read_controls())
                .execute()
                .unwrap()
                .into_rows()[0]
                .current_balance()
                .minor_units()
        };
        (
            balance(self.source, &self.owner),
            balance(self.destination, &self.recipient),
        )
    }

    pub fn activity(&self, account: AccountId) -> Vec<bank_domain::reads::AccountActivityItem> {
        self.native
            .runtime
            .account_activity(account)
            .as_principal(if account == self.source {
                &self.owner
            } else {
                &self.recipient
            })
            .execute(read_controls())
            .unwrap()
            .rows()[0]
            .entries()
            .to_vec()
    }
}

fn seeded_world(
    institution: InstitutionId,
    identities: &[DynamicIdentity; 3],
    projected: bool,
) -> BankWorldSeed {
    let ids = [1, 2, 3].map(|id| BankPrincipalId::new(id).unwrap());
    let [source, destination] = [10, 11].map(|id| AccountId::new(id).unwrap());
    let mut snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(institution)
        .principal(ids[0])
        .principal(ids[1])
        .principal(ids[2])
        .institution_cash_account(AccountId::new(1).unwrap(), institution)
        .personal_account(
            source,
            institution,
            ids[0],
            AccountName::new("Sender").unwrap(),
            AccountStatus::Open,
        )
        .personal_account(
            destination,
            institution,
            ids[1],
            AccountName::new("Recipient").unwrap(),
            AccountStatus::Open,
        );
    if projected {
        use bank_domain::{
            accounting::{BankJournalEntry, BankPosting},
            model::{AccountAuthorizationId, CustomerRole, PostingId, SignedMoney},
            proposals::BankAccountAuthorization,
            schema::PostingPurpose,
        };
        snapshot = snapshot
            .projected_journal(BankJournalEntry::from_projection(
                JournalEntryId::new(15).unwrap(),
                PostingPurpose::OpeningFunding,
                vec![
                    BankPosting::from_projection(
                        PostingId::new(16).unwrap(),
                        AccountId::new(1).unwrap(),
                        SignedMoney::from_minor(-100),
                    ),
                    BankPosting::from_projection(
                        PostingId::new(17).unwrap(),
                        source,
                        SignedMoney::from_minor(100),
                    ),
                ],
                None,
            ))
            .projected_authorization(BankAccountAuthorization::from_projection(
                AccountAuthorizationId::new(18).unwrap(),
                source,
                ids[2],
                CustomerRole::Viewer,
            ));
    }
    let mut seed =
        BankWorldSeed::new(snapshot.build().unwrap()).employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(1).unwrap(),
            institution,
            ids[2],
            EmployeeRole::Teller,
        ));
    for (id, identity) in ids.into_iter().zip(identities) {
        seed = seed.principal(BankPrincipalSeed::enabled(id, identity.external()));
    }
    seed
}

pub type BankHome = worth_query_host::facade::WorthQueryApplicationHome<
    bank_domain::schema::BankSchema,
    bank_server::BankApplication,
>;
pub type Sent = WorthQueryCompletedSourceOperation<
    bank_domain::schema::BankSchema,
    SendMoney,
    bank_server::BankApplication,
    bank_server::SendMoneyAction,
    worth_query_host::facade::declaration::application_program::ApplicationRetainedCommit,
>;

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
            .retain_operation::<BankApplication, SendMoneyAction>(home.runtime())
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
pub fn home_policy() -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(128 * 1024 * 1024),
            Some(128_000_000),
        ),
    )
}
