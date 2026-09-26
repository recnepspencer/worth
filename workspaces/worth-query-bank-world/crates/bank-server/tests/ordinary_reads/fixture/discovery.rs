use bank_domain::model::{
    AccountAuthorizationId, AccountId, AccountName, BankPrincipalId, BankSnapshotVersion,
    CustomerRole, InstitutionId,
};
use bank_domain::proposals::{BankAccountAuthorization, BankSnapshotBuilder};
use bank_domain::schema::AccountStatus;
use bank_server::{BankAuthenticatedPrincipal, BankPrincipalSeed, BankWorldSeed};

use super::id;
use crate::support::{
    block_on, request_scope, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

pub(crate) struct AccountDiscoveryFixture {
    pub world: TestIdentityWorld,
    actor: DynamicIdentity,
}

impl AccountDiscoveryFixture {
    pub fn authenticate(&self) -> BankAuthenticatedPrincipal {
        let request = request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.actor),
            &request,
        ))
        .expect("discovery actor should authenticate")
    }
}

pub(crate) fn over_budget_discovery_world(
    scenario: &str,
    authorized_accounts: usize,
) -> AccountDiscoveryFixture {
    over_budget_discovery_world_with_role(scenario, authorized_accounts, CustomerRole::Viewer)
}

pub(crate) fn over_budget_discovery_world_with_role(
    scenario: &str,
    authorized_accounts: usize,
    role: CustomerRole,
) -> AccountDiscoveryFixture {
    let actor = DynamicIdentity::new(&format!("{scenario}-actor"));
    let owner_identities = (0..authorized_accounts)
        .map(|ordinal| DynamicIdentity::new(&format!("{scenario}-owner-{ordinal}")))
        .collect::<Vec<_>>();
    let institution = id(InstitutionId::new, 1);
    let actor_id = id(BankPrincipalId::new, 1);
    let mut builder = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(institution)
        .principal(actor_id)
        .institution_cash_account(id(AccountId::new, 100), institution);
    for ordinal in 0..authorized_accounts {
        let owner = id(BankPrincipalId::new, u64::try_from(ordinal).unwrap() + 2);
        let account = id(AccountId::new, u64::try_from(ordinal).unwrap() + 1_000);
        builder = builder
            .principal(owner)
            .personal_account(
                account,
                institution,
                owner,
                AccountName::new(format!("Authorized {ordinal}")).unwrap(),
                AccountStatus::Open,
            )
            .projected_authorization(BankAccountAuthorization::from_projection(
                id(
                    AccountAuthorizationId::new,
                    u64::try_from(ordinal).unwrap() + 1,
                ),
                account,
                actor_id,
                role,
            ));
    }
    let mut seed = BankWorldSeed::new(builder.build().expect("discovery world should build"))
        .principal(BankPrincipalSeed::enabled(actor_id, actor.external()));
    for (ordinal, identity) in owner_identities.iter().enumerate() {
        seed = seed.principal(BankPrincipalSeed::enabled(
            id(BankPrincipalId::new, u64::try_from(ordinal).unwrap() + 2),
            identity.external(),
        ));
    }
    AccountDiscoveryFixture {
        world: runtime(seed),
        actor,
    }
}
