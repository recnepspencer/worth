use super::PhysicalIdempotencyReopenFailure;

pub(super) fn require_contiguous(
    spans: &[(u64, u64)],
    next_tail_lsn: Option<u64>,
    start: u64,
) -> Result<(), PhysicalIdempotencyReopenFailure> {
    let Some(mut cursor) = next_tail_lsn else {
        return Ok(());
    };
    if cursor == start {
        return Ok(());
    }
    while cursor < start {
        let Some((_, end)) = spans
            .iter()
            .find(|(span_start, end)| *span_start == cursor && *end <= start)
        else {
            return Err(PhysicalIdempotencyReopenFailure::WalTailDiscontinuity);
        };
        cursor = *end;
    }
    if cursor == start {
        Ok(())
    } else {
        Err(PhysicalIdempotencyReopenFailure::WalTailDiscontinuity)
    }
}
