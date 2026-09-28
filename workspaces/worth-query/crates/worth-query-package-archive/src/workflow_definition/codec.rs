//! WQWD bytes: a header naming the spec, identity, limits and start, then
//! the definition's nodes in identity order and its connections. Connection
//! order is not canonical in the draft: the authored definition canonicalizes
//! it, so a reordered draft keeps its content identity.

use std::time::Duration;

use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowComponentLimits, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowSpec, ValidatedWorkflowDefinition,
};

use super::records::{self, MINIMUM_CONNECTION_BYTES, MINIMUM_NODE_BYTES};
use super::WorthQueryUntrustedWorkflowDefinitionDraft;
use crate::application_program::require_byte_budget;
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::compatibility::{
    WorthQueryPackageArchiveCompatibilityProfile, WorthQueryPackageArchiveProtocolLayer,
};
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::limits::WorthQueryPackageArchiveLimits;

const MAGIC: &[u8; 4] = b"WQWD";

/// Current deterministic workflow definition draft protocol. Version 2
/// carries expression-backed conditions.
pub const WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION: u16 = 2;

/// The oldest draft protocol still read. Its conditions readmit as migrated
/// expressions; nothing writes it.
pub const WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_OLDEST_READABLE_VERSION: u16 = 1;

/// Encodes one validated definition's authored meaning. Only the draft
/// travels: nothing about instances, approvals or publication is read.
pub fn encode_workflow_definition_draft<Spec>(
    definition: &ValidatedWorkflowDefinition<Spec>,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<Vec<u8>, Denial>
where
    Spec: ApplicationWorkflowSpec,
{
    let limits = limits.narrowed();
    let mut output = BinaryOutput::with_capacity(256);
    output.raw_bytes(MAGIC);
    output.u16(WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION);
    records::text(&mut output, Spec::IDENTITY.as_str())?;
    records::text(&mut output, definition.identity().as_str())?;
    encode_limits(&mut output, definition.limits())?;
    records::text(&mut output, definition.start().as_str())?;
    output.u32(count(definition.nodes().len())?);
    for node in definition.nodes() {
        records::encode_node(&mut output, node)?;
    }
    output.u32(count(definition.connections().len())?);
    for connection in definition.connections() {
        records::encode_connection(&mut output, connection)?;
    }
    let bytes = output.into_bytes();
    require_byte_budget(bytes.len(), limits)?;
    Ok(bytes)
}

/// Decodes bounded bytes into an untrusted draft. Every count is checked
/// against the draft's own limits and the remaining bytes before anything is
/// allocated for it.
pub fn decode_workflow_definition_draft(
    bytes: &[u8],
    limits: WorthQueryPackageArchiveLimits,
) -> Result<WorthQueryUntrustedWorkflowDefinitionDraft, Denial> {
    let limits = limits.narrowed();
    require_byte_budget(bytes.len(), limits)?;
    let mut input = BinaryInput::new(bytes);
    if &input.array::<4>()? != MAGIC {
        return Err(Denial::new(Kind::InvalidMagic));
    }
    let version = input.u16()?;
    WorthQueryPackageArchiveCompatibilityProfile::CURRENT
        .admit(
            WorthQueryPackageArchiveProtocolLayer::WorkflowDefinitionDraft,
            version,
        )
        .map_err(|compatibility| {
            Denial::incompatible(
                Kind::UnsupportedWorkflowDefinitionDraftVersion,
                compatibility,
            )
        })?;
    let spec = records::nonempty_text(&mut input)?;
    let identity = records::nonempty_text(&mut input)?;
    let definition_limits = decode_limits(&mut input)?;
    let start = records::nonempty_text(&mut input)?;
    let declared_nodes = input.u32()?;
    let node_count = bounded_count(
        &input,
        declared_nodes,
        definition_limits.maximum_nodes(),
        MINIMUM_NODE_BYTES,
    )?;
    let mut nodes = Vec::with_capacity(node_count);
    for _ in 0..node_count {
        let node = records::decode_node(&mut input, version)?;
        if nodes
            .last()
            .is_some_and(|previous: &super::DraftNode| previous.identity >= node.identity)
        {
            return Err(Denial::new(Kind::NonCanonicalRecordSequence));
        }
        nodes.push(node);
    }
    let declared_connections = input.u32()?;
    let connection_count = bounded_count(
        &input,
        declared_connections,
        definition_limits.maximum_connections(),
        MINIMUM_CONNECTION_BYTES,
    )?;
    let mut connections = Vec::with_capacity(connection_count);
    for _ in 0..connection_count {
        connections.push(records::decode_connection(&mut input)?);
    }
    if !input.is_finished() {
        return Err(Denial::new(Kind::TrailingBytes));
    }
    Ok(WorthQueryUntrustedWorkflowDefinitionDraft {
        spec,
        identity,
        limits: definition_limits,
        start,
        nodes: nodes.into_boxed_slice(),
        connections: connections.into_boxed_slice(),
    })
}

fn encode_limits(
    output: &mut BinaryOutput,
    limits: ApplicationWorkflowDefinitionLimits,
) -> Result<(), Denial> {
    let components = limits.component_limits();
    output.u16(limits.maximum_nodes());
    output.u16(limits.maximum_connections());
    output.u16(limits.maximum_effects());
    output.u16(components.maximum_occurrences());
    output.raw_bytes(&[components.maximum_depth()]);
    output.u32(components.maximum_node_provenance());
    output.u32(components.maximum_connection_provenance());
    output.u32(components.maximum_port_provenance());
    output.u32(limits.maximum_canonical_bytes());
    // Zero declares no deadline; a declared one is never zero.
    let deadline = limits
        .total_deadline()
        .map_or(Ok(0), |deadline| u64::try_from(deadline.as_millis()))
        .map_err(|_| Denial::new(Kind::NumericWidthExceeded))?;
    output.u64(deadline);
    Ok(())
}

fn decode_limits(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationWorkflowDefinitionLimits, Denial> {
    let nodes = input.u16()?;
    let connections = input.u16()?;
    let effects = input.u16()?;
    let components = ApplicationWorkflowComponentLimits::new(
        input.u16()?,
        input.u8()?,
        input.u32()?,
        input.u32()?,
        input.u32()?,
    )
    .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
    let canonical_bytes = input.u32()?;
    let deadline = input.u64()?;
    let limits = ApplicationWorkflowDefinitionLimits::new(
        nodes,
        connections,
        effects,
        components,
        canonical_bytes,
    )
    .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
    match deadline {
        0 => Ok(limits),
        milliseconds => limits
            .with_total_deadline(Duration::from_millis(milliseconds))
            .ok_or_else(|| Denial::new(Kind::InvalidRecordShape)),
    }
}

fn count(length: usize) -> Result<u32, Denial> {
    u32::try_from(length).map_err(|_| Denial::new(Kind::NestedEntryBudgetExceeded))
}

/// A count within the draft's declared maximum and within what the
/// remaining bytes could hold at the smallest record size.
fn bounded_count(
    input: &BinaryInput<'_>,
    count: u32,
    maximum: u16,
    minimum_record_bytes: usize,
) -> Result<usize, Denial> {
    if count > u32::from(maximum) {
        return Err(Denial::new(Kind::NestedEntryBudgetExceeded));
    }
    let count = usize::try_from(count).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?;
    if count.saturating_mul(minimum_record_bytes) > input.remaining_len() {
        return Err(Denial::new(Kind::Truncated));
    }
    Ok(count)
}
