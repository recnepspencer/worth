pub use std::time::{Duration, SystemTime};

pub use bank_server::{
    BankApprovalAuthenticationConfiguration, BankAuthenticatedPrincipal,
    BankAuthenticationConfiguration, BankIdentityRuntime, BankPrincipalAdmissionError,
    BankPrincipalSeed, BankWorldSeed,
};
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
};
pub use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

worth_query_host::facade::worth_query_authentication_audience!(pub BankServerIdentityTestAudience);
worth_query_host::facade::worth_query_authentication_method!(pub CausalBankServerTestMethod);
worth_query_host::facade::worth_query_principal_issuer!(pub BankServerTestIssuer);

pub struct DynamicIdentity {
    pub subject: String,
}

impl DynamicIdentity {
    /// Equal workload recipes include equal authenticated principal seed bytes.
    /// Each installed runtime still admits its own real authentication owner.
    pub fn from_recipe(scenario: &str) -> Self {
        Self {
            subject: format!("bank-recipe:{scenario}"),
        }
    }

    pub fn new(scenario: &str) -> Self {
        let unique = format!(
            "{}-{}",
            std::process::id(),
            worth_query_host::facade::runtime::time::system_time_sampler()
                .system_time()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock should follow the Unix epoch")
                .as_nanos()
        );
        Self {
            subject: format!("{scenario}-{unique}"),
        }
    }

    pub fn external(&self) -> WorthQueryExternalPrincipalIdentity {
        WorthQueryExternalPrincipalIdentity::new(BankServerTestIssuer, &self.subject)
            .expect("dynamic external identity should admit")
    }
}

pub struct TestIdentityWorld {
    pub runtime: BankIdentityRuntime,
}

impl TestIdentityWorld {
    /// Authenticates `identity` as this process's local principal and
    /// resolves it to its installed Bank principal, on the calling thread.
    pub fn authenticate(
        &self,
        identity: &DynamicIdentity,
        scope: &WorthQueryRequestScope,
    ) -> Result<BankAuthenticatedPrincipal, BankPrincipalAdmissionError> {
        let authentication = self
            .runtime
            .admit_authentication_adapter(authentication_configuration(), local_principal(identity))
            .expect("the local process principal should admit");
        self.runtime.authenticate_local(&authentication, scope)
    }
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

pub fn test_world(runtime: BankIdentityRuntime) -> TestIdentityWorld {
    TestIdentityWorld { runtime }
}

pub fn authentication_configuration() -> BankAuthenticationConfiguration {
    BankAuthenticationConfiguration::new(
        WorthQueryAuthenticationAudience::of(BankServerIdentityTestAudience),
        WorthQueryAuthenticationMethod::of(CausalBankServerTestMethod),
    )
}

pub fn request_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Some(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        cancellation.token(),
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
}

/// The Host's local process principal acting as `identity`, admitted with the
/// test audience and method.
pub fn local_principal(identity: &DynamicIdentity) -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        identity.external(),
        BankServerIdentityTestAudience,
        CausalBankServerTestMethod,
        Duration::from_secs(60),
    )
    .expect("a finite local principal lifetime should admit")
}
