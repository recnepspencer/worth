pub use bank_domain::model::{
    AccountAuthorizationId, AccountId, AccountName, BankPrincipalId, BankSnapshotVersion,
    CustomerRole, InstitutionId,
};
pub use bank_domain::proposals::{BankAccountAuthorization, BankSnapshotBuilder};
pub use bank_domain::schema::AccountStatus;
pub use bank_server::{BankAuthenticatedPrincipal, BankPrincipalSeed, BankWorldSeed};

pub use crate::facade::server::support::{
    block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

pub struct CanonicalScaleFixture {
    pub world: TestIdentityWorld,
    pub baseline_actor: DynamicIdentity,
    pub expanded_actor: DynamicIdentity,
    pub baseline_account: AccountId,
    pub expanded_account: AccountId,
}

impl CanonicalScaleFixture {
    pub fn authenticate_baseline(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(&self.baseline_actor)
    }

    pub fn authenticate_expanded(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(&self.expanded_actor)
    }

    pub fn authenticate(&self, identity: &DynamicIdentity) -> BankAuthenticatedPrincipal {
        let request = crate::facade::server::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(identity),
            &request,
        ))
        .expect("canonical scale actor should authenticate")
    }
}

pub fn canonical_scale_world() -> CanonicalScaleFixture {
    const EXPANDED_ACCOUNT_COUNT: usize = 192;

    let institution = id(InstitutionId::new, 1);
    let baseline_actor_id = id(BankPrincipalId::new, 1);
    let expanded_actor_id = id(BankPrincipalId::new, 2);
    let baseline_account = id(AccountId::new, 1_000);
    let expanded_account = id(AccountId::new, 2_000);
    let baseline_actor = DynamicIdentity::new("canonical-scale-baseline-actor");
    let expanded_actor = DynamicIdentity::new("canonical-scale-expanded-actor");
    let mut owners = Vec::with_capacity(EXPANDED_ACCOUNT_COUNT + 1);
    let mut builder = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(institution)
        .principal(baseline_actor_id)
        .principal(expanded_actor_id)
        .institution_cash_account(id(AccountId::new, 100), institution);

    for ordinal in 0..=EXPANDED_ACCOUNT_COUNT {
        let owner_id = id(BankPrincipalId::new, u64::try_from(ordinal).unwrap() + 100);
        let owner_identity = DynamicIdentity::new(&format!("canonical-scale-owner-{ordinal}"));
        let account = if ordinal == 0 {
            baseline_account
        } else {
            id(AccountId::new, u64::try_from(ordinal).unwrap() + 1_999)
        };
        let authorized_principal = if ordinal == 0 {
            baseline_actor_id
        } else {
            expanded_actor_id
        };
        builder = builder
            .principal(owner_id)
            .personal_account(
                account,
                institution,
                owner_id,
                AccountName::new(format!("Scale account {ordinal}")).unwrap(),
                AccountStatus::Open,
            )
            .projected_authorization(BankAccountAuthorization::from_projection(
                id(
                    AccountAuthorizationId::new,
                    u64::try_from(ordinal).unwrap() + 1,
                ),
                account,
                authorized_principal,
                CustomerRole::Viewer,
            ));
        owners.push((owner_id, owner_identity));
    }

    let mut seed = BankWorldSeed::new(builder.build().expect("scale world should build"))
        .principal(BankPrincipalSeed::enabled(
            baseline_actor_id,
            baseline_actor.external(),
        ))
        .principal(BankPrincipalSeed::enabled(
            expanded_actor_id,
            expanded_actor.external(),
        ));
    for (owner_id, owner_identity) in &owners {
        seed = seed.principal(BankPrincipalSeed::enabled(
            *owner_id,
            owner_identity.external(),
        ));
    }

    CanonicalScaleFixture {
        world: runtime(seed),
        baseline_actor,
        expanded_actor,
        baseline_account,
        expanded_account,
    }
}

pub fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).unwrap()
}
