//! The stock canonical value binding an installed invariant refuses with.
//!
//! `WorthQueryDomainRefusalBinding` is the `RefusalBinding` of an installed
//! invariant whose `Refusal` is a [`WorthQueryDomainRefusal`]. Its decoder
//! re-admits the code and message grammar, so a commit denial decodes only a
//! valid refusal.
use super::WorthQueryDomainRefusal;
use worth_query_declaration::facade::application_schema::ValueDecodeBounds;

/// Wire ceiling: a 64-byte code, a 256-byte message whose every byte needs a
/// JSON escape (512 bytes) and the field framing all fit, with headroom.
const WIRE_MAXIMUM_BYTES: usize = 1024;

worth_query_declaration::worth_query_value_binding!(
    pub WorthQueryDomainRefusalBinding for WorthQueryDomainRefusal {
        identity: "worth.query.domain-refusal.v1", scalar: String,
        canonical: ValueDecodeBounds::new(WIRE_MAXIMUM_BYTES, 256, 8, 4),
        validate: admit_decoded,
    }
);

fn admit_decoded(refusal: &WorthQueryDomainRefusal) -> Result<(), &'static str> {
    refusal
        .admit()
        .map_err(super::WorthQueryDomainRefusalDenial::reason)
}
