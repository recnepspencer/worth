mod binding;
mod limits;
mod protocol;

pub use binding::ApplicationInboundOccurrenceBinding;
pub use limits::ApplicationInboundOccurrenceLimits;
pub use protocol::ApplicationInboundOccurrenceProtocol;

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

    use super::*;

    fn limits() -> ApplicationInboundOccurrenceLimits {
        ApplicationInboundOccurrenceLimits {
            maximum_envelope_bytes: NonZeroU64::new(1024).unwrap(),
            maximum_verifier_work: NonZeroU64::new(1024).unwrap(),
            maximum_payload_bytes: NonZeroU64::new(256).unwrap(),
            maximum_outstanding_dispatch_provenance: NonZeroU64::new(8).unwrap(),
            maximum_accepted_occurrences: NonZeroU64::new(8).unwrap(),
            maximum_accepted_bytes: NonZeroU64::new(2048).unwrap(),
            maximum_concurrent_publications: NonZeroU64::new(2).unwrap(),
            maximum_discovery_work: NonZeroU64::new(16).unwrap(),
            replay_window_milliseconds: NonZeroU64::new(1000).unwrap(),
            maximum_cleanup_work: NonZeroU64::new(16).unwrap(),
        }
    }

    #[test]
    fn binding_rejects_unusable_source_or_payload_capacity() {
        let protocol = ApplicationInboundOccurrenceProtocol::new(
            BoundaryProtocolIdentity::new("test.inbound"),
            BoundaryProtocolVersion::new(1),
        );
        assert!(
            ApplicationInboundOccurrenceBinding::<()>::new(protocol.clone(), "rail", limits())
                .is_some()
        );
        assert!(
            ApplicationInboundOccurrenceBinding::<()>::new(protocol.clone(), " ", limits())
                .is_none()
        );
        let mut too_large = limits();
        too_large.maximum_payload_bytes = NonZeroU64::new(4096).unwrap();
        assert!(
            ApplicationInboundOccurrenceBinding::<()>::new(protocol, "rail", too_large).is_none()
        );
    }
}
