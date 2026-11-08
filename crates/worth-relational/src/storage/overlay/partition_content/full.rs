//! One cold Full commitment algorithm; only its actual allocation admission varies.
use super::*;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) type Request<'a, 'request, 'authority> =
    Option<&'a ExecutionRequest<'request, 'authority>>;

pub(super) fn poll(request: Request<'_, '_, '_>) -> Result<(), PartitionContentDigestError> {
    if let Some(request) = request {
        request
            .consult()
            .map_err(|cause| PartitionContentDigestError::Request(cause.into()))?;
    }
    Ok(())
}

/// Charge the separately varying resolved-name work before hashing/comparing.
pub(super) fn charge(
    request: Request<'_, '_, '_>,
    units: u64,
) -> Result<(), PartitionContentDigestError> {
    let Some(request) = request else {
        return Ok(());
    };
    let mut outcome = None;
    let run = request.run_native_with_checkpoint(|context| {
        outcome = Some(crate::execution::request_stage::checkpoint(context, units));
    });
    match (run, outcome) {
        (_, Some(Err(cause))) => Err(PartitionContentDigestError::Request(
            worth_execution::WorkCeilingDenial::Stopped(cause).into(),
        )),
        (Err(cause), _) => Err(PartitionContentDigestError::Request(cause.into())),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed commitment charge executes callback"),
    }
}

pub(super) fn rows_memory(
    count: usize,
    request: Request<'_, '_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, PartitionContentDigestError> {
    let bytes = std::alloc::Layout::array::<(u128, [u8; 32])>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
    request
        .map(|request| {
            request
                .reserve_retained_memory(bytes)
                .map_err(|cause| PartitionContentDigestError::Request(cause.into()))
        })
        .transpose()
        .map(Option::flatten)
}

pub(super) fn capture(
    partition: &PartitionState,
    symbols: &dyn crate::symbols::data::SymbolTextLookup,
    request: Request<'_, '_, '_>,
) -> Result<(PartitionContentCommitment, u64), PartitionContentDigestError> {
    capture_using(partition, symbols, request, None)
}
pub(super) fn capture_with_frontiers(
    partition: &PartitionState,
    symbols: &dyn crate::symbols::data::SymbolTextLookup,
    frontiers: &PartitionHistoryFrontiers,
    request: Request<'_, '_, '_>,
) -> Result<(PartitionContentCommitment, u64), PartitionContentDigestError> {
    capture_using(partition, symbols, request, Some(frontiers))
}
fn capture_using(
    partition: &PartitionState,
    symbols: &dyn crate::symbols::data::SymbolTextLookup,
    request: Request<'_, '_, '_>,
    frontiers: Option<&PartitionHistoryFrontiers>,
) -> Result<(PartitionContentCommitment, u64), PartitionContentDigestError> {
    poll(request)?;
    let (entities, entity_count) = records::RecordContentIndex::capture_full(
        &partition.entity_arena,
        symbols,
        request,
        frontiers.map(|value| value.entities.as_slice()),
        |extra, symbols| {
            let fingerprint = extra
                .structural_fingerprint
                .map(|fingerprint| -> Result<_, PartitionContentDigestError> {
                    let family = symbols.resolve(fingerprint.family).ok_or(
                        PartitionContentDigestError::UnresolvedContentSymbol(fingerprint.family),
                    )?;
                    charge(request, family.len() as u64)?;
                    Ok((family, fingerprint.value))
                })
                .transpose()?;
            hash_value(
                b"entity-extra",
                &(
                    fingerprint,
                    &extra.lineage_id,
                    &extra.authoritative_aspect_state(),
                ),
            )
        },
    )?;
    let (relations, relation_count) = records::RecordContentIndex::capture_full(
        &partition.relation_arena,
        symbols,
        request,
        frontiers.map(|value| value.relations.as_slice()),
        |extra, _| hash_value(b"relation-extra", extra),
    )?;
    let (adjacency, outgoing_count) =
        adjacency::AdjacencyContentIndex::capture_full(&partition.adjacency, request)?;
    let (reverse_adjacency, incoming_count) =
        adjacency::AdjacencyContentIndex::capture_full(&partition.reverse_adjacency, request)?;
    let count = entity_count
        .checked_add(relation_count)
        .and_then(|sum| sum.checked_add(outgoing_count))
        .and_then(|sum| sum.checked_add(incoming_count))
        .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
    Ok((
        PartitionContentCommitment {
            entities,
            relations,
            adjacency,
            reverse_adjacency,
        },
        count,
    ))
}
