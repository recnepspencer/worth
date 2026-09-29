use std::num::NonZeroU64;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

use super::identity;
use crate::application_schema::{
    ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
    ApplicationSchemaMember,
};

#[test]
fn inbound_source_protocol_and_capacity_change_schema_identity() {
    let one = NonZeroU64::new(1).unwrap();
    let base = ApplicationSchemaMember::OperationInboundOccurrence {
        operation: "Operation".to_owned(),
        effect: "ExternalEffect".to_owned(),
        protocol: ApplicationInboundOccurrenceProtocol::new(
            BoundaryProtocolIdentity::new("test.inbound"),
            BoundaryProtocolVersion::new(1),
        ),
        source_identity: "rail".to_owned(),
        limits: ApplicationInboundOccurrenceLimits {
            maximum_envelope_bytes: one,
            maximum_payload_bytes: one,
            maximum_outstanding_dispatch_provenance: one,
            maximum_accepted_occurrences: one,
            maximum_accepted_bytes: one,
            maximum_concurrent_publications: one,
            maximum_discovery_work: one,
            replay_window_milliseconds: one,
            maximum_cleanup_work: one,
        },
    };
    let baseline = identity(std::slice::from_ref(&base));
    let mut changed_source = base.clone();
    let ApplicationSchemaMember::OperationInboundOccurrence {
        source_identity, ..
    } = &mut changed_source
    else {
        unreachable!()
    };
    *source_identity = "other-rail".to_owned();
    assert_ne!(identity(&[changed_source]), baseline);
    let mut changed_limit = base;
    let ApplicationSchemaMember::OperationInboundOccurrence { limits, .. } = &mut changed_limit
    else {
        unreachable!()
    };
    limits.maximum_outstanding_dispatch_provenance = NonZeroU64::new(2).unwrap();
    assert_ne!(identity(&[changed_limit]), baseline);
}
