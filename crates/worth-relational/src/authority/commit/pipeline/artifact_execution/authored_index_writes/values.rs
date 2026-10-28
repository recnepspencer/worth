//! Borrowed exact-generation membership values at the real preparation roots.
use super::*;
use crate::identity::data::{EntityId, KindId};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::substrate::{EntityRecordKind, SlotView};
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView,
};

pub(super) fn live<'a>(
    access: &'a impl CurrentPartitionAccess,
    entity: EntityId,
) -> Option<(KindId, SlotView<'a, EntityRecordKind>)> {
    let partition = access.get_current_partition(entity.partition_id)?;
    let slot = partition.entity_arena().get_slot(entity.slot_index())?;
    if !slot.is_live() || slot.generation() != entity.generation_value() {
        return None;
    }
    Some((slot.kind_id()?, slot))
}

pub(super) fn field<'a>(
    state: Option<&'a AuthoritativeRecordAspectState>,
    locator: &AspectFieldLocator,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<&'a AspectValue>, Denial> {
    if let Some(state) = state {
        let aspect = locator.aspect().aspect_key();
        // The first probe discovers struct width; the sole value decoder below
        // repeats that probe. Both descents are admitted before either read.
        let probe = (state.aspects().len() as u64)
            .checked_add(1)
            .and_then(|n| n.checked_mul((aspect.as_str().len() as u64).checked_add(1)?))
            .and_then(|n| n.checked_mul(2))
            .ok_or_else(overflow)?;
        work(context, probe)?;
        if let Some(value) = state.aspects().get(aspect)
            && let ContractValidatedAspectValueView::Struct(value) = value.view()
        {
            let field = &locator.field_path().fields()[0];
            let probe = (value.len() as u64)
                .checked_add(1)
                .and_then(|n| n.checked_mul((field.as_str().len() as u64).checked_add(1)?))
                .ok_or_else(overflow)?;
            work(context, probe)?;
        }
    }
    Ok(
        crate::visibility::materialization::read_records::authoritative_state_query_locus_value(
            state, locator,
        ),
    )
}
