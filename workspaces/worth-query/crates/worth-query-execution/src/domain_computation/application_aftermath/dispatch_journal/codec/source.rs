//! One concrete source-coordinate grammar for accepted and transport records.
use super::*;
pub(super) fn encode(output: &mut Vec<u8>, source: &QueryInboundDispatchSource) {
    for number in [
        source.world_slot,
        source.world_commit,
        source.world_attempt,
        source.incarnation,
        source.native_commit,
        source.native_version,
    ] {
        output.extend_from_slice(&number.to_le_bytes());
    }
    super::inbound::text(output, &source.native_branch);
    for number in source.entity {
        output.extend_from_slice(&number.to_le_bytes());
    }
}
pub(super) fn decode(
    reader: &mut Reader<'_>,
) -> Result<QueryInboundDispatchSource, QueryDispatchJournalDenial> {
    Ok(QueryInboundDispatchSource {
        world_slot: reader.number()?,
        world_commit: reader.number()?,
        world_attempt: reader.number()?,
        incarnation: reader.number()?,
        native_commit: reader.number()?,
        native_version: reader.number()?,
        native_branch: reader.text()?,
        entity: [reader.number()?, reader.number()?, reader.number()?],
    })
}
