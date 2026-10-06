//! Versioned equality of exact output identities after a stable publication.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{
    admission::IndexAdmission,
    index_capacity,
    mark_state::{EqualOutputLink, FullVerificationReason, MarkState, SettlementMarks},
    SettlementRegistrationStop,
};
use crate::domain_computation::primary_graph::output_lineage::{
    input_cutoff::StableEqualityConsequence, RecordedSettlementIdentity,
};

/// Mutates only the unpublished successor mark root. All selected walks and
/// copy-on-write edits are admitted before their individual operations; a
/// denial discards this root without changing the visible source image.
pub(super) fn certify(
    state: &mut MarkState,
    equality: &StableEqualityConsequence<'_>,
    admission: &mut impl IndexAdmission,
) -> Result<(), SettlementRegistrationStop> {
    let predecessor = equality.predecessor();
    let successor = equality.successor();
    admission.work(2)?;
    if predecessor == successor || predecessor.source() != successor.source() {
        return Err(SettlementRegistrationStop::Foreign);
    }
    admission.ordered_read(state.settlements.len())?;
    if !state.settlements.contains_key(predecessor) {
        return Err(missing(FullVerificationReason::MissingSettlement));
    }
    admission.ordered_read(state.settlements.len())?;
    let Some(alias) = state.settlements.get(successor) else {
        return Err(missing(FullVerificationReason::MissingSettlement));
    };
    if alias.delivery_epoch != state.delivery_epoch
        || alias.verification_requirement.is_some()
        || !alias.dirty_ordinals.is_empty()
        || !alias.pending_upstream.is_empty()
    {
        return Err(missing(FullVerificationReason::RetainedDeliveryGap));
    }
    admission.ordered_read(state.equal_links.len())?;
    if state.equal_links.contains_key(successor) {
        return Err(missing(FullVerificationReason::RetainedDeliveryGap));
    }
    admission.ordered_read(state.equal_links.len())?;
    let old_link = state.equal_links.get(predecessor).cloned();
    if old_link.as_ref().is_some_and(|link| link.next.is_some()) {
        return Err(missing(FullVerificationReason::RetainedDeliveryGap));
    }

    let mut ancestor = Arc::clone(predecessor);
    let mut visited = OrdSet::new();
    loop {
        admission.work(1)?;
        admission.ordered_read(visited.len())?;
        if visited.contains(&ancestor) {
            return Err(missing(FullVerificationReason::RetainedDeliveryGap));
        }
        admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(visited.len())?;
        visited.insert(Arc::clone(&ancestor));
        admission.ordered_read(state.settlements.len())?;
        if !state.settlements.contains_key(&ancestor) {
            return Err(missing(FullVerificationReason::MissingSettlement));
        }
        discharge_selected_downstream(state, &ancestor, admission)?;
        admission.ordered_read(state.equal_links.len())?;
        let Some(link) = state.equal_links.get(&ancestor) else {
            break;
        };
        match &link.prior {
            Some(prior) => ancestor = Arc::clone(prior),
            None => break,
        }
    }

    admission.bytes(
        index_capacity::arc_bytes::<EqualOutputLink>()
            .and_then(|bytes| bytes.checked_mul(2))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    )?;
    admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<EqualOutputLink>>(
        state.equal_links.len(),
    )?;
    state.equal_links.insert(
        Arc::clone(predecessor),
        Arc::new(EqualOutputLink {
            prior: old_link.as_ref().and_then(|link| link.prior.clone()),
            next: Some(Arc::clone(successor)),
        }),
    );
    admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<EqualOutputLink>>(
        state.equal_links.len(),
    )?;
    state.equal_links.insert(
        Arc::clone(successor),
        Arc::new(EqualOutputLink {
            prior: Some(Arc::clone(predecessor)),
            next: None,
        }),
    );
    Ok(())
}

pub(super) fn discharge_selected_downstream(
    state: &mut MarkState,
    predecessor: &Arc<RecordedSettlementIdentity>,
    admission: &mut impl IndexAdmission,
) -> Result<(), SettlementRegistrationStop> {
    admission.ordered_read(state.downstream.len())?;
    let Some(targets) = state.downstream.get(predecessor).cloned() else {
        return Ok(());
    };
    for target in targets {
        admission.work(1)?;
        admission.ordered_read(state.settlements.len())?;
        let Some(existing) = state.settlements.get(&target) else {
            continue;
        };
        admission.ordered_read(existing.pending_upstream.len())?;
        if !existing.pending_upstream.contains(predecessor) {
            continue;
        }
        admission.bytes(
            index_capacity::arc_bytes::<SettlementMarks>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        admission.ordered_remove::<Arc<RecordedSettlementIdentity>, ()>(
            existing.pending_upstream.len(),
        )?;
        let mut row = (**existing).clone();
        row.pending_upstream.remove(predecessor);
        state.pending_edge_count -= 1;
        admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
            state.settlements.len(),
        )?;
        state.settlements.insert(target, Arc::new(row));
    }
    Ok(())
}

fn missing(reason: FullVerificationReason) -> SettlementRegistrationStop {
    SettlementRegistrationStop::Alignment(reason)
}
