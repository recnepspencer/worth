use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use bank_server::{
    BankApprovalAuthenticationConfiguration, BankAuthenticationBoundary,
    BankAuthenticationConfiguration, BankIdentityRuntime, BankPrincipalSeed, BankWorldSeed,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryRequestScope, WorthQueryValidatedExternalPrincipal,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

const TEST_AUDIENCE: &str = "bank-server-identity-test";
const TEST_METHOD: &str = "causal-bank-server-test";

pub struct DynamicIdentity {
    issuer: String,
    subject: String,
}

impl DynamicIdentity {
    pub fn new(scenario: &str) -> Self {
        let unique = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock should follow the Unix epoch")
                .as_nanos()
        );
        Self {
            issuer: format!("https://issuer.test.invalid/{unique}"),
            subject: format!("{scenario}-{unique}"),
        }
    }

    pub fn external(&self) -> WorthQueryExternalPrincipalIdentity {
        WorthQueryExternalPrincipalIdentity::new(&self.issuer, &self.subject)
            .expect("dynamic external identity should admit")
    }
}

pub struct TestIdentityWorld {
    pub runtime: BankIdentityRuntime,
    pub authentication: BankAuthenticationBoundary<CausalBankAuthenticationAdapter>,
}

pub trait BankTestRuntimeSeed {
    fn install(self) -> Result<BankIdentityRuntime, bank_server::BankIdentityRuntimeBuildError>;
}

impl<const N: usize> BankTestRuntimeSeed for [BankPrincipalSeed; N] {
    fn install(self) -> Result<BankIdentityRuntime, bank_server::BankIdentityRuntimeBuildError> {
        BankIdentityRuntime::install(self)
    }
}

impl BankTestRuntimeSeed for BankWorldSeed {
    fn install(self) -> Result<BankIdentityRuntime, bank_server::BankIdentityRuntimeBuildError> {
        BankIdentityRuntime::install_world(self)
    }
}

pub fn runtime(seed: impl BankTestRuntimeSeed) -> TestIdentityWorld {
    let runtime = seed.install().expect("bank test runtime should install");
    test_world(runtime)
}

#[allow(
    dead_code,
    reason = "shared support is compiled by test targets without the payment workflow court"
)]
pub fn runtime_with_approval_authentication(
    seed: BankWorldSeed,
    approval_authentication: BankApprovalAuthenticationConfiguration,
) -> TestIdentityWorld {
    let runtime = BankIdentityRuntime::install_world_with_approval_authentication(
        seed,
        approval_authentication,
    )
    .expect("bank test runtime with approval authentication should install");
    test_world(runtime)
}

fn test_world(runtime: BankIdentityRuntime) -> TestIdentityWorld {
    let authentication = runtime
        .admit_authentication_adapter(
            authentication_configuration(),
            CausalBankAuthenticationAdapter,
        )
        .expect("causal authentication adapter should admit");
    TestIdentityWorld {
        runtime,
        authentication,
    }
}

pub fn authentication_configuration() -> BankAuthenticationConfiguration {
    BankAuthenticationConfiguration::new(
        WorthQueryAuthenticationAudience::new(TEST_AUDIENCE).expect("test audience should admit"),
        WorthQueryAuthenticationMethod::new(TEST_METHOD).expect("test method should admit"),
    )
}

pub fn request_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    )
}

pub struct CausalCredential {
    identity: WorthQueryExternalPrincipalIdentity,
}

impl CausalCredential {
    pub fn for_identity(identity: &DynamicIdentity) -> Self {
        Self {
            identity: identity.external(),
        }
    }
}

pub struct CausalBankAuthenticationAdapter;

impl WorthQueryAuthenticationAdapter for CausalBankAuthenticationAdapter {
    type Credential = CausalCredential;

    fn configuration_identity(&self) -> &str {
        "bank-server-causal-test-adapter-v1"
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
                WorthQueryAuthenticationAudience::new(TEST_AUDIENCE)
                    .expect("test audience should admit"),
                WorthQueryAuthenticationMethod::new(TEST_METHOD).expect("test method should admit"),
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

pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
