//! The host-supplied port Query dispatches an external effect through.
//!
//! Query owns no network vocabulary and no rail. It owns only the honest set
//! of things a caller can observe from one attempt, and every one of those
//! observations except `Completed` leaves the external effect unresolved.

use super::outbox::WorthQueryDispatchOutboxRecord;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily;

/// What one dispatch attempt observed. `Completed` is the only variant that
/// may become an `ExternalCompletion` posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalTransportOutcome {
    /// The external owner reported the effect complete.
    Completed,
    /// The external owner reported receipt and nothing further.
    Acknowledged,
    /// A second acknowledgement arrived for one attempt.
    DuplicateAcknowledgement,
    /// The external owner read the payload and refused it. The effect did not
    /// happen; re-sending the same bytes cannot make it happen.
    Rejected,
    /// The owner recognized the protocol family but rejected its exact
    /// produced version under its independently declared compatibility policy.
    UnsupportedProtocolVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    /// The caller's deadline elapsed before any further answer.
    TimedOut,
    /// The external owner could not be reached at all.
    Disconnected,
    /// The request left; the answer never came back. Whether the external
    /// owner acted is unknown and must stay unknown.
    LostResponse,
}

/// One durably co-committed external effect, offered to the transport.
///
/// Every field is read off the `WorthQueryDispatchOutboxRecord` that committed
/// inside the operation's own transaction, which the runtime derived from the
/// installed contract and the admitted typed emission. The payload travels
/// because an external owner that receives only a correlation cannot decode
/// what the effect *means* — it would have to look the meaning up elsewhere or
/// assume it, which is the runtime's derivation being re-done by the party
/// least able to do it (Q8.25-C3).
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryExternalDispatchRequest<'a> {
    correlation_family: &'a WorthQueryExternalEffectCorrelationFamily,
    correlation_token: &'a [u8; 32],
    /// The declared effect these bytes project from.
    effect: &'a str,
    /// Stable protocol identity of the bytes, independent of Rust type names.
    protocol_identity: &'a BoundaryProtocolIdentity,
    protocol_version: BoundaryProtocolVersion,
    /// The bound that wire protocol declared; the bytes never exceed it.
    maximum_payload_bytes: u64,
    /// Exactly the bytes the outbox co-committed, unaltered.
    payload: &'a [u8],
}

/// A host-installed external-effect transport.
///
/// Implementations live outside Query and carry the concrete protocol. Query
/// never learns the endpoint, the encoding, or the external owner's identity.
pub trait WorthQueryExternalEffectTransport: Send + Sync + 'static {
    fn dispatch(
        &self,
        request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome;
}

impl<'a> WorthQueryExternalDispatchRequest<'a> {
    /// The sole constructor: one durably co-committed record, taken whole.
    ///
    /// There is no parameter through which an effect name, a payload type, a
    /// byte bound, or payload bytes could be named separately, so a request
    /// that disagrees with what committed is unrepresentable rather than
    /// discouraged.
    pub(crate) fn for_record(record: &'a WorthQueryDispatchOutboxRecord) -> Self {
        Self {
            correlation_family: record.correlation_family(),
            correlation_token: record.correlation().bytes(),
            effect: record.effect(),
            protocol_identity: record.protocol_identity(),
            protocol_version: record.protocol_version(),
            maximum_payload_bytes: record.maximum_payload_bytes(),
            payload: record.payload(),
        }
    }

    pub const fn correlation_family(&self) -> &'a WorthQueryExternalEffectCorrelationFamily {
        self.correlation_family
    }

    pub const fn correlation_token(&self) -> &'a [u8; 32] {
        self.correlation_token
    }

    pub const fn effect(&self) -> &'a str {
        self.effect
    }

    pub const fn protocol_identity(&self) -> &'a BoundaryProtocolIdentity {
        self.protocol_identity
    }

    pub const fn protocol_version(&self) -> BoundaryProtocolVersion {
        self.protocol_version
    }

    pub const fn maximum_payload_bytes(&self) -> u64 {
        self.maximum_payload_bytes
    }

    pub const fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
    use worth_query_declaration::facade::application_schema::ApplicationExternalEffectProtocol;
    use worth_query_installation::facade::{
        InstalledExternalEffectContract, WorthQueryExternalEffectCorrelationFamily,
    };

    use super::super::{
        derive_external_effect_correlation_identity, ExternalEffectCorrelationBasis,
        WorthQueryDispatchOutboxRecord,
    };
    use super::WorthQueryExternalDispatchRequest;

    #[test]
    fn request_is_an_exact_borrowed_projection_of_one_outbox_record() {
        let correlation =
            derive_external_effect_correlation_identity(ExternalEffectCorrelationBasis {
                correlation_family: WorthQueryExternalEffectCorrelationFamily::new(
                    "transport-mapping",
                )
                .unwrap(),
                operation_slot: "notify-death",
                operation_version: 7,
                outcome_identity: 81,
                idempotency_key: &[0xA5; 32],
                branch: "mapping-branch",
            })
            .expect("the fixture correlation derives");
        let contract = InstalledExternalEffectContract::Declared {
            correlation_family: WorthQueryExternalEffectCorrelationFamily::new("transport-mapping")
                .unwrap(),
            effect: "EstateDeathNotificationEffect".to_owned(),
            rust_payload_type: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "worth.query.test.moved-payload.v1",
            ),
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::new("bank.estate.death-notification"),
                BoundaryProtocolVersion::new(1),
            ),
            maximum_payload_bytes: 24,
            inbound: None,
        };
        let record = WorthQueryDispatchOutboxRecord::from_installed_contract(
            correlation,
            &contract,
            vec![0x11, 0x22, 0x33],
            81,
        )
        .expect("the contract declares an external effect");

        let request = WorthQueryExternalDispatchRequest::for_record(&record);

        assert_eq!(request.correlation_family(), record.correlation_family());
        assert_eq!(request.correlation_token(), record.correlation().bytes());
        assert_eq!(request.effect(), record.effect());
        assert_eq!(request.protocol_identity(), record.protocol_identity());
        assert_eq!(request.protocol_version(), record.protocol_version());
        assert_eq!(
            request.maximum_payload_bytes(),
            record.maximum_payload_bytes()
        );
        assert_eq!(request.payload(), record.payload());
    }
}
