//! Real Home, external contact and exact-body trusted callback boundary.
use super::{
    operation::*,
    program::Program,
    schema::{self, Account, Schema},
};
use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime},
};
use worth_query_decl::facade::{
    application_operation::ApplicationEncodedInput,
    application_program::ApplicationProgramAuthoring,
    application_schema::{ApplicationExternalEffectBinding, ApplicationSchema},
};
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryAdvancementPhase,
    },
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    primary_graph::*,
    runtime::{ExecutionAllocationPolicy, WorthQueryApplicationQueryResourceProfile},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::{
    application_runtime_surface as identity,
    dispatch_home::{
        self as dispatch,
        notify::{BalanceNoticeBinding, BalanceValueBinding},
    },
};

pub type Home = WorthQueryApplicationHome<Schema, Program>;
type Admission = WorthQueryAdmittedApplicationOperation<Schema, NoticeOperation, u64, Account>;
pub struct Owners {
    pub location: std::path::PathBuf,
    pub handlers: Arc<AtomicUsize>,
    pub contacts: Arc<AtomicUsize>,
    pub payloads: Arc<Mutex<Vec<Vec<u8>>>>,
}
impl Owners {
    pub fn new() -> Self {
        Self {
            location: dispatch::location(),
            handlers: Arc::new(AtomicUsize::new(0)),
            contacts: Arc::new(AtomicUsize::new(0)),
            payloads: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn open(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        initialize: bool,
        supplier: Supplier,
    ) -> Home {
        let declaration = Schema::declaration().unwrap();
        let request = WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(self.location.clone()),
            ApplicationProgramAuthoring::<Schema, Program>::begin()
                .with_schema(&declaration)
                .validated_program()
                .unwrap(),
            WorthQueryApplicationProgramRoster::new(),
            declaration,
            (supplier,),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            WorthQueryApplicationLimits::new(
                identity::world_resources(),
                WorthQueryApplicationQueryResourceProfile::default(),
                SignalConditionalEvaluationBudget::development(),
            ),
            move |_, graph, installed| {
                assert!(initialize, "reopen must not reseed");
                schema::seed(graph, installed)
            },
        );
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| panic!("real original Home: {refused:?}"));
        let operation = home
            .runtime()
            .runtime()
            .installed_schema()
            .installed_operation(NoticeOperation::reference())
            .unwrap();
        assert!(
            operation.body().is_none(),
            "this court must use the actual supplier path"
        );
        assert!(operation
            .contracts()
            .emissions()
            .emissions()
            .iter()
            .any(|emission| emission.effect()
                == operation.contracts().external_effect().effect().unwrap()));
        home.runtime()
            .runtime()
            .install_external_effect_transport(Arc::new(Rail(
                self.contacts.clone(),
                self.payloads.clone(),
            )))
            .unwrap();
        home
    }
}
struct Rail(Arc<AtomicUsize>, Arc<Mutex<Vec<Vec<u8>>>>);
impl WorthQueryExternalEffectTransport for Rail {
    fn dispatch(
        &self,
        request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        assert_eq!(request.payload().len(), 8);
        self.1.lock().unwrap().push(request.payload().to_vec());
        self.0.fetch_add(1, Ordering::SeqCst);
        WorthQueryExternalTransportOutcome::Acknowledged
    }
}
pub fn close(home: Home, execution: &worth_execution::ExecutionRequest<'_, '_>) {
    home.close(execution)
        .unwrap_or_else(|refused| panic!("graceful original close: {refused:?}"))
        .reclaim();
}
pub fn emit(home: &Home, key: u64) -> WorthQueryApplicationCommitReceipt {
    let (_, scope) = super::super::super::pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(Notice(key))
        .without_source()
        .idempotency(&key)
        .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("actual declaration must commit: {outcome:?}")
    };
    assert_eq!(result, key);
    assert_eq!(
        receipt.dispatch_outbox().unwrap().payload(),
        key.to_be_bytes()
    );
    receipt
}
pub fn admitted<R>(
    home: &Home,
    key: u64,
    body: impl for<'step> FnOnce(
        &WorthQueryAdvancementPhase<'step>,
        &Admission,
        WorthQueryApplicationIdempotencyBinding,
        &ApplicationEncodedInput<BalanceValueBinding>,
    ) -> R,
) -> R {
    let (_, scope) = super::super::super::pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(Notice(key))
        .without_source()
        .idempotency(&key)
        .with_recovery_in_program(runtime, body)
        .unwrap()
}
pub fn recover(
    home: &Home,
    key: u64,
) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
    admitted(home, key, |phase, admission, binding, input| {
        home.runtime()
            .runtime()
            .mint_admitted_recovery_handle(phase, admission, binding, input)
    })
}
pub fn coordinates(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryCorrectionSourceCoordinates {
    WorthQueryRecordedCorrectionSource::from_untrusted_parts(
        receipt
            .committed_product_publication()
            .composite_commit()
            .clone(),
        receipt.commit_reference().clone(),
    )
    .coordinates()
}

/// This trusted adapter authenticates one exact external envelope; it never issues completion.
pub struct Callback {
    body: Vec<u8>,
    claims: WorthQueryInboundOccurrenceClaims,
}
impl Callback {
    pub fn from_original(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        let record = receipt.dispatch_outbox().unwrap();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let body = format!("original-notice:{}", record.outcome_identity()).into_bytes();
        let text = std::str::from_utf8(&body).unwrap();
        let signed_meaning_digest = super::super::source_controls::signed_meaning(text);
        Self {
            body,
            claims: WorthQueryInboundOccurrenceClaims {
                audience: "compensation-original".into(),
                source_identity: SOURCE.into(),
                key_epoch: 1,
                message_identity: [57; 32],
                signed_meaning_digest,
                issued_at_unix_seconds: now,
                expires_at_unix_seconds: now + Duration::from_secs(60).as_secs(),
                protocol_identity: BalanceNoticeBinding::PROTOCOL.identity().clone(),
                protocol_version: BalanceNoticeBinding::PROTOCOL.version(),
                correlation_family: FAMILY.into(),
                correlation_token: *record.correlation().bytes(),
                payload: record.payload().to_vec(),
            },
        }
    }
    pub fn complete(self, home: &Home) {
        let body = self.body.clone();
        let operation = home
            .runtime()
            .runtime()
            .installed_schema()
            .installed_operation(NoticeOperation::reference())
            .unwrap();
        let handle = home
            .runtime()
            .runtime()
            .install_inbound_occurrence_verifier(&operation, Arc::new(self))
            .unwrap();
        let (_, scope) = super::super::super::pending_support::scope();
        let received = home
            .runtime()
            .runtime()
            .receive_inbound_occurrence(&handle, &body, &scope)
            .unwrap();
        assert_eq!(
            received.posture(),
            WorthQueryInboundReceiptPosture::Performed,
            "pending: {:?}, preparation: {:?}",
            received.pending_reason(),
            received.preparation_denial()
        );
        drop(received);
        home.runtime()
            .runtime()
            .maintain_inbound_occurrences(&handle, std::num::NonZeroUsize::new(16).unwrap(), &scope)
            .unwrap();
    }
}
impl WorthQueryInboundOccurrenceVerifier for Callback {
    fn audience(&self) -> &str {
        &self.claims.audience
    }
    fn source_identity(&self) -> &str {
        &self.claims.source_identity
    }
    fn protocol_identity(&self) -> &worth_foundational::facade::BoundaryProtocolIdentity {
        &self.claims.protocol_identity
    }
    fn protocol_version(&self) -> worth_foundational::facade::BoundaryProtocolVersion {
        self.claims.protocol_version
    }
    fn verify(
        &self,
        envelope: &[u8],
        now: u64,
        _: NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        if envelope != self.body {
            return Err(WorthQueryInboundVerificationDenial::AuthenticationFailed);
        }
        if now < self.claims.issued_at_unix_seconds || now > self.claims.expires_at_unix_seconds {
            return Err(WorthQueryInboundVerificationDenial::Expired);
        }
        Ok(self.claims.clone())
    }
}
