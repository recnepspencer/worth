//! Bounded process-local rail sender custody for completed consequences.

mod ack;
mod configuration;
mod encoding;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

use crate::completion_wire::{RailCustodyAckPosture, SignedRailCompletion, CUSTODY_ACK_V1_BYTES};
use crate::protocol::correlation::RailCorrelation;
use crate::protocol::payload::RailEffectPayload;

pub use configuration::{
    RailCompletionDeliveryConfiguration, RailCompletionDeliveryConfigurationDenial,
};
use encoding::sign_completed_effect;

/// Counts expose unresolved work without disclosing notice or credentials.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RailCompletionDeliveryPosture {
    pub reserved: usize,
    pub pending: usize,
    pub exhausted: usize,
    /// Signed authoritative denials observed during this process lifetime.
    pub permanently_denied: u64,
}

struct Obligation {
    signed: SignedRailCompletion,
    attempts: u32,
    next_attempt: Instant,
    deadline: Instant,
    exhausted: bool,
    in_flight: bool,
}

enum SenderTurn {
    Due(RailCorrelation, SignedRailCompletion),
    WaitUntil(Instant),
    Idle,
}

#[derive(Default)]
struct Custody {
    reserved: usize,
    obligations: HashMap<RailCorrelation, Obligation>,
    permanently_denied: u64,
}

pub(super) struct CompletionDelivery {
    configuration: RailCompletionDeliveryConfiguration,
    custody: Mutex<Custody>,
    wake: Notify,
    client: reqwest::Client,
}

impl CompletionDelivery {
    pub(super) fn new(
        configuration: RailCompletionDeliveryConfiguration,
    ) -> Result<Arc<Self>, RailCompletionDeliveryConfigurationDenial> {
        configuration.validate()?;
        let mut builder = reqwest::Client::builder()
            .tls_built_in_root_certs(false)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_millis(configuration.contact_timeout_millis));
        if let Some(pem) = &configuration.bank_tls_root_pem {
            let certificate = reqwest::Certificate::from_pem(pem.as_bytes())
                .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust)?;
            builder = builder.add_root_certificate(certificate);
        }
        let client = builder
            .build()
            .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidDestination)?;
        Ok(Arc::new(Self {
            configuration,
            custody: Mutex::new(Custody::default()),
            wake: Notify::new(),
            client,
        }))
    }

    pub(super) fn reserve(
        self: &Arc<Self>,
    ) -> Result<Option<DeliveryReservation>, DeliveryRecordDenial> {
        let issued = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| DeliveryRecordDenial::ClockUnavailable)?
            .as_secs();
        issued
            .checked_add(self.configuration.validity_seconds)
            .ok_or(DeliveryRecordDenial::ClockUnavailable)?;
        let mut custody = self.lock();
        if custody.reserved + custody.obligations.len() >= self.configuration.maximum_pending {
            return Ok(None);
        }
        custody.reserved += 1;
        Ok(Some(DeliveryReservation {
            owner: Arc::clone(self),
            active: true,
            issued,
        }))
    }

    pub(super) fn admits_envelope(
        &self,
        correlation: &RailCorrelation,
        payload: &RailEffectPayload,
    ) -> bool {
        let protocol = payload.protocol_identity().as_str().len();
        let family = correlation.family().len();
        let token = correlation.token().len();
        let bytes = payload.bytes().len();
        if protocol > u16::MAX as usize
            || payload.protocol_version().get() > u16::MAX.into()
            || family > u16::MAX as usize
            || token > u16::MAX as usize
            || bytes > u32::MAX as usize
        {
            return false;
        }
        let fixed = 16 + 2 + 2 + 8 + 32 + 8 + 8 + 2 + 2 + 2 + 2 + 4 + 64;
        fixed
            + self.configuration.audience.len()
            + self.configuration.source.len()
            + protocol
            + family
            + token
            + bytes
            <= crate::completion_wire::MAXIMUM_ENVELOPE_BYTES
    }

    pub(super) fn posture(&self) -> RailCompletionDeliveryPosture {
        let custody = self.lock();
        RailCompletionDeliveryPosture {
            reserved: custody.reserved,
            pending: custody.obligations.len(),
            exhausted: custody
                .obligations
                .values()
                .filter(|obligation| obligation.exhausted)
                .count(),
            permanently_denied: custody.permanently_denied,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Custody> {
        self.custody
            .lock()
            .expect("rail delivery custody mutex is never poisoned")
    }

    fn record_completed(
        &self,
        reservation: &mut DeliveryReservation,
        correlation: RailCorrelation,
        payload: &RailEffectPayload,
    ) -> Result<(), DeliveryRecordDenial> {
        let signed = sign_completed_effect(
            &self.configuration,
            &correlation,
            payload,
            reservation.issued,
        )
        .map_err(|_| DeliveryRecordDenial::InvalidPayload)?;
        let mut custody = self.lock();
        if custody.obligations.contains_key(&correlation) {
            return Err(DeliveryRecordDenial::Duplicate);
        }
        custody.reserved -= 1;
        reservation.active = false;
        custody.obligations.insert(
            correlation,
            Obligation {
                signed,
                attempts: 0,
                next_attempt: Instant::now(),
                deadline: Instant::now() + Duration::from_secs(self.configuration.validity_seconds),
                exhausted: false,
                in_flight: false,
            },
        );
        self.wake.notify_one();
        Ok(())
    }

    /// Sends one due obligation at a time. This is an explicit concurrency
    /// ceiling of one; slow peers cannot multiply outgoing contacts.
    pub(super) async fn run(self: Arc<Self>) {
        loop {
            match self.next_turn() {
                SenderTurn::Due(correlation, signed) => {
                    let result = self.contact(&signed).await;
                    self.finish_attempt(&correlation, result);
                }
                SenderTurn::WaitUntil(instant) => tokio::select! {
                    () = self.wake.notified() => {},
                    () = tokio::time::sleep_until(instant.into()) => {},
                },
                SenderTurn::Idle => self.wake.notified().await,
            }
        }
    }

    fn next_turn(&self) -> SenderTurn {
        let now = Instant::now();
        let mut custody = self.lock();
        let mut next_wake: Option<Instant> = None;
        for (correlation, obligation) in &mut custody.obligations {
            if !obligation.exhausted && now >= obligation.deadline {
                obligation.exhausted = true;
            }
            if !obligation.exhausted && !obligation.in_flight && obligation.next_attempt <= now {
                obligation.in_flight = true;
                obligation.attempts += 1;
                return SenderTurn::Due(correlation.clone(), obligation.signed.clone());
            }
            if !obligation.exhausted && !obligation.in_flight {
                let wake = obligation.next_attempt.min(obligation.deadline);
                next_wake = Some(next_wake.map_or(wake, |current| current.min(wake)));
            }
        }
        next_wake.map_or(SenderTurn::Idle, SenderTurn::WaitUntil)
    }

    async fn contact(&self, signed: &SignedRailCompletion) -> Option<RailCustodyAckPosture> {
        let mut response = self
            .client
            .post(&self.configuration.callback_url)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(signed.bytes().to_vec())
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        if response
            .content_length()
            .is_some_and(|length| length != CUSTODY_ACK_V1_BYTES as u64)
        {
            return None;
        }
        let mut bytes = Vec::with_capacity(CUSTODY_ACK_V1_BYTES);
        while let Some(chunk) = response.chunk().await.ok()? {
            if chunk.len() > CUSTODY_ACK_V1_BYTES - bytes.len() {
                return None;
            }
            bytes.extend_from_slice(&chunk);
        }
        ack::verify_ack(&bytes, signed, &self.configuration.bank_ack_verifying_key)
    }

    fn finish_attempt(&self, correlation: &RailCorrelation, result: Option<RailCustodyAckPosture>) {
        let mut custody = self.lock();
        let Some(obligation) = custody.obligations.get_mut(correlation) else {
            return;
        };
        if let Some(posture) = result {
            custody.obligations.remove(correlation);
            if posture == RailCustodyAckPosture::PermanentDenied {
                custody.permanently_denied = custody.permanently_denied.saturating_add(1);
            }
            return;
        }
        obligation.in_flight = false;
        if obligation.attempts >= self.configuration.maximum_attempts
            || Instant::now() >= obligation.deadline
        {
            obligation.exhausted = true;
            return;
        }
        let shift = obligation.attempts.saturating_sub(1).min(8);
        let delay = self
            .configuration
            .retry_delay_millis
            .saturating_mul(1 << shift);
        obligation.next_attempt = Instant::now() + Duration::from_millis(delay);
    }
}

/// Capacity reserved before rail consequence application. Dropping an unused
/// reservation returns it; only the completed-effect owner may consume it.
pub(super) struct DeliveryReservation {
    owner: Arc<CompletionDelivery>,
    active: bool,
    issued: u64,
}

impl DeliveryReservation {
    pub(super) fn record_completed(
        &mut self,
        correlation: RailCorrelation,
        payload: &RailEffectPayload,
    ) -> Result<(), DeliveryRecordDenial> {
        let owner = Arc::clone(&self.owner);
        owner.record_completed(self, correlation, payload)
    }
}

impl Drop for DeliveryReservation {
    fn drop(&mut self) {
        if self.active {
            self.owner.lock().reserved -= 1;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DeliveryRecordDenial {
    ClockUnavailable,
    InvalidPayload,
    Duplicate,
}
