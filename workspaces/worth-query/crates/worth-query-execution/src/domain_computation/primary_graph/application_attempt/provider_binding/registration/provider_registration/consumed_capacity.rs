use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

/// Admit the new Arc-slice backing before any application effect. Every row
/// retains the shared ticket, so a selected upstream edge keeps the backing
/// funded after the original lineage and attempt retire.
pub(super) fn admit_backing(
    provider: &WorthQueryPrimaryGraphProvider,
    consumed_outputs: &mut [ConsumedOutputEvidence],
) -> Result<(), &'static str> {
    if consumed_outputs.is_empty() {
        return Ok(());
    }
    let alignment = std::mem::align_of::<ConsumedOutputEvidence>();
    let backing_bytes = consumed_outputs
        .len()
        .checked_mul(std::mem::size_of::<ConsumedOutputEvidence>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<usize>() * 2 + alignment * 2))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or("consumed output backing capacity overflow")?;
    let owner = &provider.graph.source_owner.invalidation_owner;
    let mut admission = owner.edit_admission();
    let backing = owner
        .retain_consumed_output_backing(backing_bytes, &mut admission)
        .map_err(|_| "consumed output backing capacity unavailable")?;
    for consumed in consumed_outputs {
        consumed.attach_backing_capacity(std::sync::Arc::clone(&backing));
    }
    Ok(())
}
