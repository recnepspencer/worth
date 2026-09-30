//! Portable wait fields; typed marker identity is resolved from installed schema provenance.

use std::num::NonZeroU64;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowAwaitInbound, ApplicationWorkflowInboundWait,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
};

use super::{nonempty_text, text};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::workflow_definition::DraftInbound;

pub(super) fn encode(
    output: &mut BinaryOutput,
    awaited: &ApplicationWorkflowAwaitInbound,
) -> Result<(), Denial> {
    let inbound = awaited.inbound();
    text(output, awaited.origin().as_str())?;
    text(output, inbound.effect())?;
    text(output, inbound.protocol().identity().as_str())?;
    output.u32(inbound.protocol().version().get());
    text(output, inbound.source_identity())?;
    let limits = inbound.limits();
    for value in [
        limits.maximum_envelope_bytes,
        limits.maximum_verifier_work,
        limits.maximum_payload_bytes,
        limits.maximum_outstanding_dispatch_provenance,
        limits.maximum_accepted_occurrences,
        limits.maximum_accepted_bytes,
        limits.maximum_concurrent_publications,
        limits.maximum_discovery_work,
        limits.replay_window_milliseconds,
        limits.maximum_cleanup_work,
    ] {
        output.u64(value.get());
    }
    output.raw_bytes(&[match awaited.wait() {
        ApplicationWorkflowInboundWait::UntilInstanceDeadline => 0,
    }]);
    Ok(())
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<DraftInbound, Denial> {
    let origin = nonempty_text(input)?;
    let effect = nonempty_text(input)?;
    let identity = BoundaryProtocolIdentity::parse(nonempty_text(input)?)
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let version = BoundaryProtocolVersion::try_new(input.u32()?)
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let source = nonempty_text(input)?;
    let mut number =
        || NonZeroU64::new(input.u64()?).ok_or_else(|| Denial::new(Kind::InvalidRecordShape));
    let limits = ApplicationInboundOccurrenceLimits {
        maximum_envelope_bytes: number()?,
        maximum_verifier_work: number()?,
        maximum_payload_bytes: number()?,
        maximum_outstanding_dispatch_provenance: number()?,
        maximum_accepted_occurrences: number()?,
        maximum_accepted_bytes: number()?,
        maximum_concurrent_publications: number()?,
        maximum_discovery_work: number()?,
        replay_window_milliseconds: number()?,
        maximum_cleanup_work: number()?,
    };
    if !limits.accommodates_payload() || input.u8()? != 0 {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    Ok(DraftInbound {
        origin,
        effect,
        protocol: ApplicationInboundOccurrenceProtocol::new(identity, version),
        source,
        limits,
    })
}
