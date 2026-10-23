use super::*;
use crate::support::{block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld};
use bank_domain::{
    model::{AccountName, BankSnapshotVersion, EmployeeAssignmentId, EmployeeRole, InstitutionId},
    proposals::BankSnapshotBuilder,
    schema::AccountStatus,
};
use bank_server::{BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed};

pub(super) struct World {
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
        let source = AccountId::new(10).unwrap();
        let destination = AccountId::new(11).unwrap();
        let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
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
            )
            .build()
            .unwrap();
        let mut seed = BankWorldSeed::new(snapshot).employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(1).unwrap(),
            institution,
            ids[2],
            EmployeeRole::Teller,
        ));
        for (id, identity) in ids.into_iter().zip(&identities) {
            seed = seed.principal(BankPrincipalSeed::enabled(id, identity.external()));
        }
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
