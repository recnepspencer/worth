//! Apply node drafts by identity, then mint their semantic evidence in plan order.
use super::{
    EpochPublishedProducer, RetainedNodePayload, SignalError, SignalPreparationBudget, Work,
};

pub(super) type PlanSemanticSlot = Option<(usize, usize)>;

pub(super) fn record_slot(
    slots: &mut [PlanSemanticSlot],
    plan_index: usize,
    payload_index: usize,
    published_index: usize,
    work: &mut Work,
) -> Result<(), SignalError> {
    work.visit()
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
    let slot = slots
        .get_mut(plan_index)
        .ok_or_else(|| SignalError::internal("epoch producer index outside plan"))?;
    if slot.replace((payload_index, published_index)).is_some() {
        return Err(SignalError::internal("duplicate epoch semantic slot"));
    }
    Ok(())
}

pub(super) fn prepare_in_plan_order<P>(
    payloads: &mut [P],
    published: &mut [EpochPublishedProducer],
    slots: Vec<PlanSemanticSlot>,
    work: &mut Work,
    mut preparation: Option<&mut SignalPreparationBudget>,
    mut payload_for: impl FnMut(&mut P) -> &mut RetainedNodePayload,
    mut prepare_semantic: impl FnMut(
        &mut RetainedNodePayload,
        &mut EpochPublishedProducer,
        &mut Work,
        Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError>,
) -> Result<(), SignalError> {
    for slot in slots {
        work.visit()
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let (payload_index, published_index) =
            slot.ok_or_else(|| SignalError::internal("missing epoch semantic slot"))?;
        let payload = payload_for(
            payloads
                .get_mut(payload_index)
                .ok_or_else(|| SignalError::internal("epoch semantic payload unavailable"))?,
        );
        let producer = published
            .get_mut(published_index)
            .ok_or_else(|| SignalError::internal("epoch semantic producer unavailable"))?;
        prepare_semantic(payload, producer, work, preparation.as_deref_mut())?;
    }
    Ok(())
}
