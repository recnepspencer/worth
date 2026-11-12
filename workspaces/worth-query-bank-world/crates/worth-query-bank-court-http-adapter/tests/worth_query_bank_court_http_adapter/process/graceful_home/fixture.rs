//! Actual Directory Homes share only external rail custody and immutable fixture inputs.
use worth_query_host::facade::application_entry::{WorthQueryApplicationMutationOutcome,WorthQueryApplicationRequestExt};
use worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase;
use bank_domain::estate::EstateAction;
use super::{callback_capture::CallbackCapture,operation::*,program::RailHomeProgram,schema::*};
use std::sync::{Arc,atomic::AtomicUsize};
use std::time::Duration;
use bank_domain::{estate::{DeathNoticeId,EstateCaseId,EstateDeathNotificationRequest},model::BankPrincipalId};
use bank_external_rail::{RailCompletionDeliveryConfiguration,RailProcessHandle};
use bank_http_adapter::BankRailCompletionServerInstallation;
use ed25519_dalek::SigningKey;
use worth_query_decl::facade::{application_program::*,application_schema::ApplicationSchema};
use worth_query_host::facade::{application_installation::*,primary_graph::*,runtime::WorthQueryApplicationQueryResourceProfile,WorthQueryApplicationHome,WorthQueryApplicationHomeRequest,WorthQueryApplicationHomeLocation};
pub use worth_query_testkit_host::facade::host::{checkpoint_capture,application_runtime_surface as identity};
pub use worth_query_bank_testkit_server::facade::estate_operations::external_effect_dispatch::rail_transport::{BankEstateRailTransport,FaultScript,LedgerStatus,rail_binary_path};
pub use worth_query_bank_testkit_server::facade::support::request_scope;
pub type Home = WorthQueryApplicationHome<RailHomeSchema, RailHomeProgram>;
pub type Admission =
    WorthQueryAdmittedApplicationOperation<RailHomeSchema, NotifyRail, EstateAction, Account>;
type Encoded = worth_query_decl::facade::application_operation::ApplicationEncodedInput<
    bank_domain::schema::EstateActionInputBinding,
>;
pub struct Opened {
    pub home: Home,
    pub transport: Arc<BankEstateRailTransport>,
    pub verifier: WorthQueryInboundVerifierHandle,
}
pub struct World {
    location: std::path::PathBuf,
    pub handlers: Arc<AtomicUsize>,
    pub rail: RailProcessHandle,
    pub callback: CallbackCapture,
    pub capture_loop: Arc<BankEstateRailTransport>,
}
impl World {
    pub fn new() -> Self {
        let mut rail = RailProcessHandle::spawn_awaiting_completion_installation(
            rail_binary_path(),
            "127.0.0.1:0",
        )
        .unwrap();
        let capture_loop = Arc::new(BankEstateRailTransport::connected_to(
            rail.local_addr(),
            rail.test_control_addr(),
        ));
        let callback = CallbackCapture::bind(&capture_loop.blocking);
        let config = RailCompletionDeliveryConfiguration::new(
            format!("http://{}/completion", callback.address),
            "bank-home-court".into(),
            "rail-primary".into(),
            1,
            [7; 32],
            SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes(),
            60,
            8,
            1,
            Duration::from_millis(10),
            Duration::from_secs(10),
            None,
        )
        .unwrap();
        rail.install_completion_delivery(&config).unwrap();
        Self {
            location: worth_query_testkit_host::facade::host::dispatch_home::location(),
            handlers: Arc::new(AtomicUsize::new(0)),
            rail,
            callback,
            capture_loop,
        }
    }
    pub fn open(
        &self,
        execution: checkpoint_capture::ExecutionRequest<'_, '_>,
        seed: bool,
    ) -> Opened {
        let declaration = RailHomeSchema::declaration().unwrap();
        let request = WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(self.location.clone()),
            ApplicationProgramAuthoring::<RailHomeSchema, RailHomeProgram>::begin()
                .with_schema(&declaration)
                .validated_program()
                .unwrap(),
            WorthQueryApplicationProgramRoster::new(),
            declaration,
            (self.handlers.clone(),),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            WorthQueryApplicationLimits::new(
                identity::world_resources(),
                WorthQueryApplicationQueryResourceProfile::default(),
                SignalConditionalEvaluationBudget::development(),
            ),
            move |_, graph, installed| {
                assert!(seed, "a reopened Bank rail Home must never initialize");
                super::schema::seed(graph, installed)
            },
        );
        let home =
            checkpoint_capture::with_bootstrap_advancement_from_request(execution, |phase| {
                request.open_in_advancement(&phase)
            })
            .unwrap()
            .unwrap_or_else(|denial| panic!("actual Directory Home open: {denial:?}"));
        let transport = Arc::new(BankEstateRailTransport::connected_to(
            self.rail.local_addr(),
            self.rail.test_control_addr(),
        ));
        home.runtime()
            .runtime()
            .install_external_effect_transport(transport.clone())
            .unwrap();
        let installation = BankRailCompletionServerInstallation::new(
            SigningKey::from_bytes(&[7; 32]).verifying_key().to_bytes(),
            [9; 32],
            "bank-home-court".into(),
            "rail-primary".into(),
            1,
            0,
        )
        .unwrap();
        let op = home
            .runtime()
            .runtime()
            .installed_schema()
            .installed_operation(NotifyRail::reference())
            .unwrap();
        let verifier = home
            .runtime()
            .runtime()
            .install_inbound_occurrence_verifier(&op, installation.estate_verifier())
            .unwrap();
        Opened {
            home,
            transport,
            verifier,
        }
    }
    pub fn finish(self) {
        let Self {
            rail,
            callback,
            capture_loop,
            ..
        } = self;
        let _posture = rail
            .close()
            .expect("external owner reports any deliberately unacknowledged callback work");
        callback.close(&capture_loop.blocking);
    }
}
impl Opened {
    pub fn close(self, execution: &checkpoint_capture::ExecutionRequest<'_, '_>) {
        let Self {
            home,
            transport,
            verifier,
        } = self;
        drop(verifier);
        drop(transport);
        home.close(execution)
            .unwrap_or_else(|denial| panic!("actual Bank rail Home must close: {denial:?}"))
            .reclaim();
    }
    pub fn emit(
        &self,
        value: EstateDeathNotificationRequest,
        key: u64,
    ) -> WorthQueryApplicationCommitReceipt {
        let scope = request_scope();
        let runtime = self.home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let outcome = runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(Notify(action(value)))
            .without_source()
            .idempotency(&key)
            .execute_in_program(
                runtime,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("original Bank payload emission must commit: {outcome:?}")
        };
        receipt
    }
    pub fn admitted<R>(
        &self,
        value: EstateDeathNotificationRequest,
        key: u64,
        body: impl for<'step> FnOnce(
            &WorthQueryAdvancementPhase<'step>,
            &Admission,
            WorthQueryApplicationIdempotencyBinding,
            &Encoded,
        ) -> R,
    ) -> R {
        let scope = request_scope();
        let runtime = self.home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(Notify(action(value)))
            .without_source()
            .idempotency(&key)
            .with_recovery_in_program(runtime, body)
            .unwrap()
    }
    pub fn recover(
        &self,
        value: EstateDeathNotificationRequest,
        key: u64,
    ) -> WorthQueryRecoveryHandle {
        self.admitted(value, key, |phase, admission, binding, input| {
            self.home
                .runtime()
                .runtime()
                .mint_admitted_recovery_handle(phase, admission, binding, input)
        })
        .unwrap()
    }
    pub fn maintain(&self) {
        let report = self
            .home
            .runtime()
            .runtime()
            .maintain_inbound_occurrences(
                &self.verifier,
                std::num::NonZeroUsize::new(8).unwrap(),
                &request_scope(),
            )
            .unwrap();
        assert_eq!(
            report.remaining(),
            0,
            "actual terminal ACK and retained work settle"
        );
    }
}
pub fn notice(id: u64) -> EstateDeathNotificationRequest {
    EstateDeathNotificationRequest::new(
        EstateCaseId::new(id).unwrap(),
        DeathNoticeId::new(id + 1).unwrap(),
        BankPrincipalId::new(id + 2).unwrap(),
    )
}

pub fn action(value: EstateDeathNotificationRequest) -> EstateAction {
    EstateAction::NotifyDeath {
        estate: value.estate(),
        notice: value.notice(),
        subject: value.subject(),
    }
}
