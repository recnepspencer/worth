//! Emit a physical member frame using Durability's original-request encoder.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
pub use worth_relational_storage::internal_port::storage::data::member_frame::locator;
use worth_relational_storage::internal_port::storage::data::member_frame::MAGIC;
pub(crate) use worth_relational_storage::internal_port::storage::data::member_frame::{
    decode, encode, locator_for_request, locator_in_stage, MemberKind,
};

/// Admit and emit the same complete frame through the original request.
pub(crate) fn encode_for_request(
    kind: MemberKind,
    value: &impl serde::Serialize,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<
    (
        RelationalDurableMemberLocator,
        worth_execution::ExecutionImmutableBytes,
    ),
    contracts::durability::data::recovery_stop::RelationalRecoveryStop,
> {
    let mut prefix = [0_u8; MAGIC.len() + 3];
    prefix[..MAGIC.len()].copy_from_slice(MAGIC);
    prefix[MAGIC.len()..MAGIC.len() + 2].copy_from_slice(&1_u16.to_be_bytes());
    prefix[MAGIC.len() + 2] = kind as u8;
    let bytes = contracts::durability::artifact_payload_emission::request::encode_named(
        &prefix, value, request,
    )?;
    Ok((
        locator_for_request(request, bytes.bytes())
            .map_err(contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request)?,
        bytes,
    ))
}
