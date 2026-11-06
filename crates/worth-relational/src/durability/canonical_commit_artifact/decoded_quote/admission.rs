//! Admit each actual retained owner and the additional decode peak first.
use super::*;
use crate::history::data::CanonicalEnvelopeReadmissionMemory;
use worth_execution::{ExecutionImmutableBytes, ExecutionMemoryReservation};

impl DecodedQuote {
    pub(in super::super) fn admit(
        self,
        request: &ExecutionRequest<'_, '_>,
        input: Option<ExecutionImmutableBytes>,
    ) -> Result<
        (
            CanonicalEnvelopeReadmissionMemory,
            Option<ExecutionMemoryReservation>,
        ),
        Refusal,
    > {
        let transient = self
            .envelope
            .transient
            .checked_add(self.semantic.transient)
            .and_then(|bytes| bytes.checked_add(self.association.transient))
            .and_then(|bytes| bytes.checked_add(self.origin.transient))
            .and_then(|bytes| bytes.checked_add(self.cascades.transient))
            .and_then(|bytes| bytes.checked_add(self.selection.transient))
            .and_then(|bytes| bytes.checked_add(self.index_delta.transient))
            .and_then(|bytes| bytes.checked_add(self.index_writes.transient))
            .ok_or_else(malformed)?;
        let transient = reserve(request, transient)?;
        // The original fixed reservation precedes both its own private control
        // Arc and the envelope Arc built by positioned admission later.
        let control = reserve(request, self.control)?.map(|memory| {
            std::sync::Arc::new(
                crate::history::data::CanonicalEnvelopeControlAllocation::from_reserved(memory),
            )
        });
        let memory = CanonicalEnvelopeReadmissionMemory {
            input,
            envelope: reserve(request, self.envelope.retained)?,
            semantic: reserve(request, self.semantic.retained)?,
            association: reserve(request, self.association.retained)?,
            index_delta: reserve(request, self.index_delta.retained)?,
            index_writes: reserve(request, self.index_writes.retained)?,
            origin: reserve(request, self.origin.retained)?,
            cascades: reserve_shared(request, self.cascades.retained)?,
            selection: reserve_shared(request, self.selection.retained)?,
            merge: None,
            schema: None,
            strategy: None,
            validation: None,
            codec_declarations: None,
            control,
        };
        Ok((memory, transient))
    }
}
fn reserve(
    request: &ExecutionRequest<'_, '_>,
    bytes: u64,
) -> Result<Option<ExecutionMemoryReservation>, Refusal> {
    if bytes == 0 {
        return Ok(None);
    }
    request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))
}

/// The artifact and checkpoint entries use the same actual leaf-transfer phase.
pub(in super::super) fn admit_canonical(
    mut quote: DecodedQuote,
    canonical: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    input: Option<ExecutionImmutableBytes>,
    additional_envelope_bytes: u64,
) -> Result<
    (
        CanonicalEnvelopeReadmissionMemory,
        Option<ExecutionMemoryReservation>,
    ),
    Refusal,
> {
    let (schema, schema_bytes) = super::schema::prepare_canonical_readmission(canonical, request)?;
    let (validation, validation_bytes) =
        super::validation::prepare_canonical_readmission(canonical, request)?;
    let (codec, codec_bytes) =
        super::codec_retirements::prepare_canonical_readmission(canonical, request)?;
    let (strategy, strategy_bytes) =
        super::strategy::prepare_canonical_readmission(canonical, request)?;
    quote.envelope.retained = quote
        .envelope
        .retained
        .checked_sub(schema_bytes)
        .and_then(|n| n.checked_sub(codec_bytes))
        .and_then(|n| n.checked_sub(strategy_bytes))
        .and_then(|n| n.checked_add(additional_envelope_bytes))
        .ok_or_else(malformed)?;
    quote.semantic.retained = quote
        .semantic
        .retained
        .checked_sub(validation_bytes)
        .ok_or_else(malformed)?;
    let (mut memory, transient) = quote.admit(request, input)?;
    memory.schema = Some(schema);
    memory.strategy = strategy;
    memory.validation = Some(validation);
    memory.codec_declarations = Some(codec);
    memory.merge = super::merge::prepare_canonical_readmission(canonical, request)?;
    Ok((memory, transient))
}

fn reserve_shared(
    request: &ExecutionRequest<'_, '_>,
    bytes: u64,
) -> Result<Option<std::sync::Arc<ExecutionMemoryReservation>>, Refusal> {
    if bytes == 0 {
        return Ok(None);
    }
    let total = bytes
        .checked_add(shared_root::<ExecutionMemoryReservation>()?.retained)
        .ok_or_else(malformed)?;
    Ok(reserve(request, total)?.map(std::sync::Arc::new))
}
