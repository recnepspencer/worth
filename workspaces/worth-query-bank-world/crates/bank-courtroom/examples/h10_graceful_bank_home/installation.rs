#![allow(
    clippy::result_large_err,
    reason = "The journey drives the real Home open, whose refusal hands back its original owner"
)]
//! One real assessed Bank installation; snapshots are inputs, never request authority.
use bank_domain::{model::*, proposals::*, schema::*};
use bank_server::{
    BankAssessedPaymentsApplication, BankAuthenticatedPrincipal, BankHomeRuntime,
    BankPrincipalSeed, BankWorldSeed,
};
use worth_query_bank_testkit_server::facade::{
    estate_operations::freeze_account::fixture::{self as estate, DynamicIdentity},
    ordinary_reads::fixture::proposals::{binding, create_business, fund, grant, key},
    recorded_correction::fixture::home_policy,
    support::{authentication_configuration, local_principal, request_scope},
};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement, WorthQueryApplicationHomeLocation,
    WorthQueryWorkflowApplicationHome,
};

pub(super) type Home = WorthQueryWorkflowApplicationHome<
    BankSchema,
    ApprovedBusinessPaymentWorkflow,
    BankAssessedPaymentsApplication,
>;
pub(super) const BUSINESS: BusinessId = BusinessId::new(40).unwrap();
pub(super) const RECIPIENT: BankPrincipalId = estate::FOREIGN_OWNER;
pub(super) const NOTIFICATION_ESTATE: bank_domain::estate::EstateCaseId =
    bank_domain::estate::EstateCaseId::new(50).unwrap();
pub(super) const NOTIFICATION_NOTICE: bank_domain::estate::DeathNoticeId =
    bank_domain::estate::DeathNoticeId::new(51).unwrap();

pub(super) struct Inputs {
    pub identities: [DynamicIdentity; 3],
    pub payment: PaymentId,
    pub deferred_payment: PaymentId,
    snapshot: BankSnapshot,
}
impl Inputs {
    pub fn from_recipe(recipe: &str) -> Self {
        let identities = ["deceased", "specialist", "recipient"]
            .map(|role| DynamicIdentity::from_recipe(&format!("{recipe}/{role}")));
        let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
            .institution(estate::INSTITUTION)
            .business(BUSINESS)
            .principal(estate::DECEASED)
            .principal(estate::SPECIALIST)
            .principal(RECIPIENT)
            .institution_cash_account(AccountId::new(80).unwrap(), estate::INSTITUTION)
            .personal_account(
                estate::ESTATE_ACCOUNT,
                estate::INSTITUTION,
                estate::DECEASED,
                AccountName::new("Estate Operating").unwrap(),
                AccountStatus::Open,
            )
            .personal_account(
                estate::FOREIGN_ACCOUNT,
                estate::INSTITUTION,
                RECIPIENT,
                AccountName::new("Recipient").unwrap(),
                AccountStatus::Open,
            )
            .build()
            .unwrap();
        // These existing proposal functions prepare initial data only. All commands below
        // are admitted by the installed Home and freshly authenticated principals.
        let snapshot = create_business(snapshot, estate::INSTITUTION, BUSINESS);
        let account = snapshot.business_account(BUSINESS).unwrap();
        let snapshot = fund(
            snapshot,
            estate::INSTITUTION,
            account,
            20_000,
            "h10-opening",
        );
        let snapshot = grant(
            snapshot,
            account,
            estate::DECEASED,
            CustomerRole::Initiator,
            "h10-initiator",
        );
        let snapshot = grant(
            snapshot,
            account,
            estate::SPECIALIST,
            CustomerRole::Approver,
            "h10-approver",
        );
        let proposal = BankProposalEngine::prepare_initiate_business_payment(
            &snapshot,
            binding(8),
            &key("h10-pending-payment"),
            estate::DECEASED,
            &InitiateBusinessPayment {
                business: BUSINESS,
                from: account,
                recipient: RECIPIENT,
                amount: Money::from_minor(2_500).unwrap(),
            },
        )
        .unwrap();
        let snapshot = proposal.proposed_snapshot().clone();
        let payment = snapshot.payments().next().unwrap().id();
        let proposal = BankProposalEngine::prepare_initiate_business_payment(
            &snapshot,
            binding(9),
            &key("h10-deferred-payment"),
            estate::DECEASED,
            &InitiateBusinessPayment {
                business: BUSINESS,
                from: account,
                recipient: RECIPIENT,
                amount: Money::from_minor(1_000).unwrap(),
            },
        )
        .unwrap();
        let snapshot = proposal.proposed_snapshot().clone();
        let deferred_payment = snapshot
            .payments()
            .find(|row| row.id() != payment)
            .unwrap()
            .id();
        Self {
            identities,
            payment,
            deferred_payment,
            snapshot,
        }
    }
    pub fn open(&self, path: &std::path::Path) -> Home {
        let seed = self
            .identities
            .iter()
            .zip([estate::DECEASED, estate::SPECIALIST, RECIPIENT])
            .fold(
                BankWorldSeed::new(self.snapshot.clone())
                    .business_owner(bank_server::BankBusinessOwnerSeed::new(
                        BUSINESS,
                        estate::DECEASED,
                    ))
                    .employee(bank_server::BankEmployeeAssignmentSeed::new(
                        estate::ASSIGNMENT,
                        estate::INSTITUTION,
                        estate::SPECIALIST,
                        EmployeeRole::EstateSpecialist,
                    ))
                    .estate(notification_estate_inputs()),
                |seed, (identity, principal)| {
                    seed.principal(BankPrincipalSeed::enabled(principal, identity.external()))
                },
            );
        let recipe = bank_server::bank_assessed_payments_home(
            WorthQueryApplicationHomeLocation::Directory(path.to_owned()),
            seed,
        )
        .unwrap();
        let home =
            with_bootstrap_advancement(home_policy(), |phase| recipe.open_in_advancement(&phase))
                .unwrap()
                .unwrap();
        let schema = home.runtime().runtime().installed_schema();
        let authentication = bank_server::install_approval_authentication(
            schema,
            bank_server::BankApprovalAuthenticationConfiguration::denying(),
        )
        .unwrap();
        let workflow =
            bank_server::install_payment_workflow_spec(schema, home.runtime().installed_program())
                .unwrap();
        home.retain_workflow_spec(workflow, authentication.signing_owner())
            .unwrap()
    }
    pub fn authenticate(&self, home: &Home, ordinal: usize) -> BankAuthenticatedPrincipal {
        let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
        let authentication = bank
            .admit_authentication_adapter(
                authentication_configuration(),
                local_principal(&self.identities[ordinal]),
            )
            .unwrap();
        bank.authenticate_local(&authentication, &request_scope())
            .unwrap()
    }
}
fn notification_estate_inputs() -> bank_domain::estate::BankEstateWorld {
    use bank_domain::estate::*;
    use worth_query_bank_testkit_server::facade::estate_operations::notify_death::fixture::{
        estate_case, notification_grant,
    };
    let mut grant = notification_grant(
        CapabilityGrantId::new(52).unwrap(),
        NOTIFICATION_ESTATE,
        estate::DECEASED,
        None,
    );
    // The existing notification fixture uses a different input principal roster.
    // This is an initial grant record, admitted only by the actual installed capability policy.
    grant.grantee = estate::SPECIALIST;
    estate::estate_world(estate::ESTATE_ACCOUNT)
        .with_death_notice(EstateDeathNotice {
            id: NOTIFICATION_NOTICE,
            subject: estate::DECEASED,
            status: DeathNoticeStatus::Reported,
        })
        .with_case(estate_case(
            NOTIFICATION_ESTATE,
            estate::ESTATE_ACCOUNT,
            estate::DECEASED,
            NOTIFICATION_NOTICE,
        ))
        .with_estate_assignment(NOTIFICATION_ESTATE, estate::ASSIGNMENT)
        .with_grant(grant)
}
pub(super) fn checkpoint(home: &Home) -> u64 {
    with_bootstrap_advancement(home_policy(), |phase| {
        home.checkpoint(&phase.execution_request())
    })
    .unwrap()
    .unwrap()
    .generation()
}
pub(super) fn checkpoint_covered_journals(home: &Home) -> Vec<(String, u64)> {
    with_bootstrap_advancement(home_policy(), |phase| {
        home.inspect_checkpoint(&phase.execution_request())
    })
    .unwrap()
    .unwrap()
    .journals
    .into_iter()
    .map(|journal| (journal.name.to_owned(), journal.covered))
    .collect()
}
pub(super) fn close(home: Home) {
    let closed = with_bootstrap_advancement(home_policy(), |phase| {
        home.close(&phase.execution_request())
    })
    .unwrap()
    .unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
}
