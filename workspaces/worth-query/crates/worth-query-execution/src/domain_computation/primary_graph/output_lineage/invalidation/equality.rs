//! Versioned equality of exact output identities after a stable publication.

use std::sync::Arc;
mod projection;

use im::OrdSet;

use super::mark_state::EqualOutputLink;
use super::mark_state::FullVerificationReason;
use super::mark_state::MarkState;
use super::SettlementRegistrationStop;
use crate::domain_computation::primary_graph::output_lineage::input_cutoff::StableEqualityConsequence;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

/// Mutates only the unpublished successor mark root. All selected walks and
/// copy-on-write edits are admitted before their individual operations; a
/// denial discards this root without changing the visible source image.
pub(super) fn certify(
    state: &mut MarkState,
    equality: &StableEqualityConsequence<'_>,
) -> Result<(), SettlementRegistrationStop> {
    let predecessor = equality.predecessor();
    let successor = equality.successor();

    if predecessor == successor || predecessor.source() != successor.source() {
        return Err(SettlementRegistrationStop::Foreign);
    }

    if !state.settlements.contains_key(predecessor) {
        return Err(missing(FullVerificationReason::MissingSettlement));
    }

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

    if state.equal_links.contains_key(successor) {
        return Err(missing(FullVerificationReason::RetainedDeliveryGap));
    }

    let old_link = state.equal_links.get(predecessor).cloned();
    if old_link.as_ref().is_some_and(|link| link.next.is_some()) {
        return Err(missing(FullVerificationReason::RetainedDeliveryGap));
    }

    projection::carry(state, predecessor, successor)?;
    let mut ancestor = Arc::clone(predecessor);
    let mut visited = OrdSet::new();
    loop {
        if visited.contains(&ancestor) {
            return Err(missing(FullVerificationReason::RetainedDeliveryGap));
        }
        // The walk creates `visited` empty and never shares it while it grows.

        visited.insert(Arc::clone(&ancestor));

        if !state.settlements.contains_key(&ancestor) {
            return Err(missing(FullVerificationReason::MissingSettlement));
        }
        discharge_selected_downstream(state, &ancestor)?;

        let Some(link) = state.equal_links.get(&ancestor) else {
            break;
        };
        match &link.prior {
            Some(prior) => ancestor = Arc::clone(prior),
            None => break,
        }
    }

    // These links can keep identities after their lineage records retire.
    // Their creation was already prepared by the lineage owner.

    state.equal_links.insert(
        Arc::clone(predecessor),
        Arc::new(EqualOutputLink {
            prior: old_link.as_ref().and_then(|link| link.prior.clone()),
            next: Some(Arc::clone(successor)),
        }),
    );

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
) -> Result<(), SettlementRegistrationStop> {
    let Some(targets) = state.downstream.get(predecessor).cloned() else {
        return Ok(());
    };
    for target in targets {
        let Some(existing) = state.settlements.get(&target) else {
            continue;
        };

        if !existing.pending_upstream.contains(predecessor) {
            continue;
        }

        let mut row = (**existing).clone();
        row.pending_upstream.remove(predecessor);
        state.pending_edge_count -= 1;

        state.settlements.insert(target, Arc::new(row));
    }
    Ok(())
}

fn missing(reason: FullVerificationReason) -> SettlementRegistrationStop {
    SettlementRegistrationStop::Alignment(reason)
}
