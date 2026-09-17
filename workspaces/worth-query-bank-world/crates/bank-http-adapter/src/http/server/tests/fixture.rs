use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant, SystemTime};

use bank_domain::estate::{DeathNoticeStatus, EstateCaseId};
use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, EmployeeAssignmentId,
    EmployeeRole, InstitutionId, Money,
};
use bank_domain::proposals::{
    BankIdempotencyKey, BankOperationScopeBinding, BankOperationScopeEntityBinding,
    BankOperationScopeSchemaBinding, BankProposalEngine, BankSnapshot, BankSnapshotBuilder,
};
use bank_domain::schema::{AccountStatus, ApplyOpeningFunding, Deposit};
use bank_server::{
    queries, BankAuthenticatedPrincipal, BankAuthenticationBoundary,
    BankAuthenticationConfiguration, BankEmployeeAssignmentSeed, BankIdentityRuntime,
    BankPrincipalSeed, BankReadControls, BankWorldSeed,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryRequestScope, WorthQueryValidatedExternalPrincipal,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::primary_graph::WorthQueryRuntimeTimeSource;

use super::super::super::protocol::{BankHttpCredential, BankHttpDenial};
use super::super::authentication::BankHttpApplicationAuthenticator;

const TEST_ISSUER: &str = "https://bank-http.test.invalid";
const TEST_SUBJECT: &str = "tcp-owner";
const TEST_AUDIENCE: &str = "bank-http-test";
const TEST_METHOD: &str = "causal-http-test";

mod estate;
mod held_authentication;
use estate::estate_world;
pub(super) use held_authentication::held_authentication_application;

pub(super) struct CausalHttpApplication {
    pub(super) runtime: BankIdentityRuntime,
    authentication: BankAuthenticationBoundary<CausalAuthenticationAdapter>,
}

impl BankHttpApplicationAuthenticator for CausalHttpApplication {
    fn runtime(&self) -> &BankIdentityRuntime {
        &self.runtime
    }

    fn authenticate<'a>(
        &'a self,
        credential: BankHttpCredential,
        scope: &'a WorthQueryRequestScope,
    ) -> Pin<Box<dyn Future<Output = Result<BankAuthenticatedPrincipal, BankHttpDenial>> + Send + 'a>>
    {
        Box::pin(async move {
            let serialized =
                serde_json::to_value(credential).expect("test credential should serialize");
            let subject = serialized
                .get("access_token")
                .and_then(serde_json::Value::as_str)
                .filter(|value| *value != "test-only")
                .unwrap_or(TEST_SUBJECT);
            self.runtime
                .authenticate_with(
                    &self.authentication,
                    CausalCredential {
                        identity: external_identity_for(subject),
                    },
                    scope,
                )
                .await
                .map_err(|_| panic!("causal HTTP credential should authenticate"))
        })
    }
}

struct CausalAuthenticationAdapter;

struct CausalCredential {
    identity: WorthQueryExternalPrincipalIdentity,
}

impl CausalHttpApplication {
    pub(super) async fn authenticate(
        &self,
        scope: &WorthQueryRequestScope,
    ) -> BankAuthenticatedPrincipal {
        self.runtime
            .authenticate_with(
                &self.authentication,
                CausalCredential {
                    identity: external_identity(),
                },
                scope,
            )
            .await
            .expect("fixture principal should authenticate")
    }

    pub(super) async fn death_notice_status(&self, estate: EstateCaseId) -> DeathNoticeStatus {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(5),
            cancellation.token(),
        );
        let principal = self.authenticate(&scope).await;
        self.runtime
            .query(queries::estate_case(estate))
            .as_principal(&principal)
            .controls(BankReadControls::current(scope, 16, 20_000).unwrap())
            .execute()
            .expect("fixture specialist should read estate status")
            .rows()[0]
            .death_notice()
            .status()
    }
}

impl WorthQueryAuthenticationAdapter for CausalAuthenticationAdapter {
    type Credential = CausalCredential;

    fn configuration_identity(&self) -> &str {
        "causal-bank-http-adapter-v1"
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                credential.identity,
                audience(),
                method(),
                now,
                now + Duration::from_secs(60),
                Vec::new(),
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

pub(super) fn application(account: AccountId) -> CausalHttpApplication {
    let runtime = BankIdentityRuntime::install_world(world_seed(account))
        .expect("bank HTTP runtime should install");
    application_from_runtime(runtime)
}

pub(super) fn application_with_authorization_time(
    account: AccountId,
    source: impl WorthQueryRuntimeTimeSource,
) -> CausalHttpApplication {
    let runtime = BankIdentityRuntime::install_world_with_authorization_time_source(
        world_seed(account),
        source,
    )
    .expect("bank HTTP runtime should install with authorization time");
    application_from_runtime(runtime)
}

fn world_seed(account: AccountId) -> BankWorldSeed {
    let principal = BankPrincipalId::new(1).unwrap();
    let institution = InstitutionId::new(1).unwrap();
    let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(institution)
        .principal(principal)
        .principal(BankPrincipalId::new(2).unwrap())
        .principal(BankPrincipalId::new(3).unwrap())
        .principal(BankPrincipalId::new(4).unwrap())
        .principal(BankPrincipalId::new(5).unwrap())
        .principal(BankPrincipalId::new(6).unwrap())
        .institution_cash_account(AccountId::new(1).unwrap(), institution)
        .personal_account(
            account,
            institution,
            principal,
            AccountName::new("Daily").unwrap(),
            AccountStatus::Open,
        )
        .personal_account(
            AccountId::new(101).unwrap(),
            institution,
            BankPrincipalId::new(2).unwrap(),
            AccountName::new("Estate Operating").unwrap(),
            AccountStatus::Open,
        )
        .personal_account(
            AccountId::new(102).unwrap(),
            institution,
            BankPrincipalId::new(3).unwrap(),
            AccountName::new("Beneficiary").unwrap(),
            AccountStatus::Open,
        )
        .build()
        .expect("bank HTTP world should build");
    let snapshot = apply_funding(snapshot, FundingSpec::opening(institution, account, 100));
    let snapshot = apply_funding(snapshot, FundingSpec::deposit(institution, account, 200));
    let snapshot = apply_funding(
        snapshot,
        FundingSpec::estate(institution, AccountId::new(101).unwrap(), 1_000),
    );
    BankWorldSeed::new(snapshot)
        .principal(BankPrincipalSeed::enabled(principal, external_identity()))
        .principal(BankPrincipalSeed::enabled(
            BankPrincipalId::new(2).unwrap(),
            WorthQueryExternalPrincipalIdentity::new(TEST_ISSUER, "estate-subject").unwrap(),
        ))
        .principal(BankPrincipalSeed::enabled(
            BankPrincipalId::new(3).unwrap(),
            WorthQueryExternalPrincipalIdentity::new(TEST_ISSUER, "beneficiary").unwrap(),
        ))
        .principal(BankPrincipalSeed::enabled(
            BankPrincipalId::new(4).unwrap(),
            WorthQueryExternalPrincipalIdentity::new(TEST_ISSUER, "executor").unwrap(),
        ))
        .principal(BankPrincipalSeed::enabled(
            BankPrincipalId::new(5).unwrap(),
            external_identity_for("approver"),
        ))
        .principal(BankPrincipalSeed::enabled(
            BankPrincipalId::new(6).unwrap(),
            external_identity_for("reviewer"),
        ))
        .employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(1).unwrap(),
            institution,
            principal,
            EmployeeRole::Teller,
        ))
        .employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(2).unwrap(),
            institution,
            principal,
            EmployeeRole::EstateSpecialist,
        ))
        .employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(3).unwrap(),
            institution,
            BankPrincipalId::new(5).unwrap(),
            EmployeeRole::EstateSpecialist,
        ))
        .employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(4).unwrap(),
            institution,
            BankPrincipalId::new(6).unwrap(),
            EmployeeRole::Compliance,
        ))
        .estate(estate_world(institution, principal))
}

fn application_from_runtime(runtime: BankIdentityRuntime) -> CausalHttpApplication {
    let authentication = runtime
        .admit_authentication_adapter(
            BankAuthenticationConfiguration::new(audience(), method()),
            CausalAuthenticationAdapter,
        )
        .expect("bank HTTP authentication should install");
    CausalHttpApplication {
        runtime,
        authentication,
    }
}

struct FundingSpec {
    institution: InstitutionId,
    account: AccountId,
    amount: Money<bank_domain::model::USD>,
    key: &'static str,
    opening: bool,
}

impl FundingSpec {
    fn opening(institution: InstitutionId, account: AccountId, amount: i64) -> Self {
        Self::new(institution, account, amount, "http-funding-1", true)
    }

    fn deposit(institution: InstitutionId, account: AccountId, amount: i64) -> Self {
        Self::new(institution, account, amount, "http-funding-2", false)
    }

    fn estate(institution: InstitutionId, account: AccountId, amount: i64) -> Self {
        Self::new(institution, account, amount, "http-estate-funding", true)
    }

    fn new(
        institution: InstitutionId,
        account: AccountId,
        amount: i64,
        key: &'static str,
        opening: bool,
    ) -> Self {
        Self {
            institution,
            account,
            amount: Money::from_minor(amount).unwrap(),
            key,
            opening,
        }
    }
}

fn apply_funding(snapshot: BankSnapshot, spec: FundingSpec) -> BankSnapshot {
    let key = BankIdempotencyKey::new(spec.key).unwrap();
    let proposal = if spec.opening {
        BankProposalEngine::prepare_opening_funding(
            &snapshot,
            operation_binding(spec.account),
            &key,
            &ApplyOpeningFunding {
                institution: spec.institution,
                account: spec.account,
                amount: spec.amount,
            },
        )
    } else {
        BankProposalEngine::prepare_deposit(
            &snapshot,
            operation_binding(spec.account),
            &key,
            &Deposit {
                institution: spec.institution,
                account: spec.account,
                amount: spec.amount,
            },
        )
    };
    proposal
        .expect("HTTP activity funding should prepare")
        .proposed_snapshot()
        .clone()
}

fn operation_binding(account: AccountId) -> BankOperationScopeBinding {
    BankOperationScopeBinding::new(
        1,
        BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
        "bank-http-test-funding",
        BankOperationScopeEntityBinding::new(0, 1, 1),
        BankOperationScopeEntityBinding::new(0, account.canonical_text().len() as u64, 1),
    )
}

pub(super) fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    external_identity_for(TEST_SUBJECT)
}

fn external_identity_for(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(TEST_ISSUER, subject).unwrap()
}

fn audience() -> WorthQueryAuthenticationAudience {
    WorthQueryAuthenticationAudience::new(TEST_AUDIENCE).unwrap()
}

fn method() -> WorthQueryAuthenticationMethod {
    WorthQueryAuthenticationMethod::new(TEST_METHOD).unwrap()
}
