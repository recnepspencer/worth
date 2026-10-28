//! This provider owns its issued attempts; frame fields alone are insufficient.
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_decl::facade::{
    application_resource::ApplicationResourceBinding,
    application_schema::{
        ApplicationScalarValueBinding, ApplicationSchema, StringApplicationValueBinding,
        U64ApplicationValueBinding,
    },
    worth_query_portable_type,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;
use worth_query_host::facade::{domain::*, primary_graph::*};

pub struct ProviderRole;
worth_query_portable_type!(ProviderRole => "certification.resource-provider.v1");

pub struct Clock;
impl WorthQueryNamedClock for Clock {
    const PORTABLE_IDENTITY: &'static str = "capability-retention.resource-clock.v1";
}
#[derive(Default)]
pub struct ClockSource {
    nanos: AtomicU64,
    sequence: AtomicU64,
}
impl ClockSource {
    pub fn milliseconds(&self, value: u64) {
        self.nanos.store(value * 1_000_000, Ordering::SeqCst);
    }
    pub fn observations(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }
}
impl WorthQueryNamedClockSource<Clock> for ClockSource {
    const SEMANTIC_IDENTITY: &'static str = "capability-retention.resource-clock-source.v1";
    fn source_identity(&self) -> WorthQueryClockSourceIdentity {
        WorthQueryClockSourceIdentity::declare("capability-resource-clock").unwrap()
    }
    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity {
        WorthQueryClockTimelineIdentity::declare("capability-resource-process").unwrap()
    }
    fn observe(&self) -> Result<WorthQueryNamedClockReading<Clock>, WorthQueryNamedClockFailure> {
        Ok(WorthQueryNamedClockReading::new(
            self.sequence.fetch_add(1, Ordering::SeqCst),
            WorthQueryClockCoordinate::from_nanoseconds(self.nanos.load(Ordering::SeqCst)),
        ))
    }
}

struct Issued {
    attempt: WorthQueryResourceProviderAttempt,
    writer: WorthQueryResourceIngressWriter,
    token: u64,
    input: u64,
    closed: bool,
}
#[derive(Default)]
pub struct ProducerState {
    issued: Mutex<Vec<Issued>>,
    next_token: AtomicU64,
    starts: AtomicUsize,
    verifies: AtomicUsize,
    closes: AtomicUsize,
    pub cleanup_pending: AtomicBool,
    pub cancel_on_start: Mutex<Option<WorthQueryCancellationSource>>,
    pub external_response_lost: AtomicBool,
}
impl ProducerState {
    pub fn counts(&self) -> (usize, usize, usize) {
        (
            self.starts.load(Ordering::SeqCst),
            self.verifies.load(Ordering::SeqCst),
            self.closes.load(Ordering::SeqCst),
        )
    }
    pub fn send(
        &self,
        occurrence: usize,
        value: u64,
    ) -> Result<(), WorthQueryResourceIngressDenial> {
        let issued = self.issued.lock().unwrap();
        let mut frame = [0_u8; 17];
        frame[..8].copy_from_slice(&issued[occurrence].token.to_le_bytes());
        frame[9..].copy_from_slice(&value.to_le_bytes());
        issued[occurrence].writer.write(&frame)
    }
    pub fn send_domain_failure(
        &self,
        occurrence: usize,
    ) -> Result<(), WorthQueryResourceIngressDenial> {
        let issued = self.issued.lock().unwrap();
        let mut frame = [0_u8; 17];
        frame[..8].copy_from_slice(&issued[occurrence].token.to_le_bytes());
        frame[8] = 2;
        issued[occurrence].writer.write(&frame)
    }
    pub fn send_indeterminate(
        &self,
        occurrence: usize,
    ) -> Result<(), WorthQueryResourceIngressDenial> {
        let issued = self.issued.lock().unwrap();
        let mut frame = [0_u8; 17];
        frame[..8].copy_from_slice(&issued[occurrence].token.to_le_bytes());
        frame[8] = 1;
        issued[occurrence].writer.write(&frame)
    }
    pub fn send_forged(&self, occurrence: usize) -> Result<(), WorthQueryResourceIngressDenial> {
        self.issued.lock().unwrap()[occurrence]
            .writer
            .write(&[0_u8; 17])
    }
    pub fn input(&self, occurrence: usize) -> u64 {
        self.issued.lock().unwrap()[occurrence].input
    }
}
pub struct Producer(pub Arc<ProducerState>);
impl<Schema, Binding> WorthQueryApplicationResourceProvider<Schema, Binding> for Producer
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<
        Schema,
        InputBinding = U64ApplicationValueBinding,
        ResultBinding = U64ApplicationValueBinding,
        FailureBinding = StringApplicationValueBinding,
        Provider = ProviderRole,
    >,
{
    type Role = ProviderRole;
    fn start(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
        input: &u64,
        writer: WorthQueryResourceIngressWriter,
    ) -> Result<(), WorthQueryResourceProviderFailure> {
        let token = self.0.next_token.fetch_add(1, Ordering::SeqCst) + 1;
        self.0.issued.lock().unwrap().push(Issued {
            attempt: attempt.clone(),
            writer,
            token,
            input: *input,
            closed: false,
        });
        self.0.starts.fetch_add(1, Ordering::SeqCst);
        if let Some(cancellation) = self.0.cancel_on_start.lock().unwrap().take() {
            cancellation.cancel();
        }
        Ok(())
    }
    fn verify(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
        frame: &[u8],
    ) -> Result<WorthQueryResourceVerifiedFrame, WorthQueryResourceVerificationFailure> {
        self.0.verifies.fetch_add(1, Ordering::SeqCst);
        let issued = self.0.issued.lock().unwrap();
        let issued = issued
            .iter()
            .find(|value| value.attempt.correlation() == attempt.correlation())
            .ok_or(WorthQueryResourceVerificationFailure::ForeignAttempt)?;
        if issued.closed || frame.len() != 17 || frame[..8] != issued.token.to_le_bytes() {
            return Err(WorthQueryResourceVerificationFailure::Unauthenticated);
        }
        if frame[8] == 1 && self.0.external_response_lost.load(Ordering::SeqCst) {
            return Ok(WorthQueryResourceVerifiedFrame {
                kind: WorthQueryResourceFrameKind::EffectsIndeterminate,
                value: StringApplicationValueBinding::encode(&"lost-response".to_owned()).unwrap(),
            });
        }
        if frame[8] == 2 {
            return Ok(WorthQueryResourceVerifiedFrame {
                kind: WorthQueryResourceFrameKind::DomainFailure,
                value: StringApplicationValueBinding::encode(&"provider-domain-failure".to_owned())
                    .unwrap(),
            });
        }
        if frame[8] != 0 {
            return Err(WorthQueryResourceVerificationFailure::InvalidFrame);
        }
        let value = u64::from_le_bytes(frame[9..].try_into().unwrap());
        Ok(WorthQueryResourceVerifiedFrame {
            kind: WorthQueryResourceFrameKind::Result,
            value: U64ApplicationValueBinding::encode(&value).unwrap(),
        })
    }
    fn close(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
    ) -> WorthQueryResourceCleanupPosture {
        self.0.closes.fetch_add(1, Ordering::SeqCst);
        if self.0.cleanup_pending.load(Ordering::SeqCst) {
            return WorthQueryResourceCleanupPosture::Pending;
        }
        if let Some(issued) = self
            .0
            .issued
            .lock()
            .unwrap()
            .iter_mut()
            .find(|value| value.attempt.correlation() == attempt.correlation())
        {
            issued.closed = true;
        }
        WorthQueryResourceCleanupPosture::Closed
    }
}
