//! End a discarded frame without orphaning its undelivered publication.
use super::refreshed_rejoin::{occurrence_rows, refresh_order, superseded};
use super::*;

enum DeliveryOwner {
    Entry,
    RegistryFrame,
    None,
}

impl WorthQueryOutputDemandInterest {
    /// Close a discarded successor's custody. An entry owning this occurrence
    /// can deliver before collecting a wave; a mere prerequisite claim cannot.
    /// With no delivery owner, closing the frame closes its Published row.
    /// This release walks only the occurrence's metadata, as Interest::drop
    /// does. Obligation/prerequisite cleanup is noted for its paying drain.
    pub(in crate::domain_computation::primary_graph) fn close_discarded_successor(&self) {
        {
            let interest = self;
            let mut state = interest
                .owner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match delivery_owner(&state, &interest.key) {
                DeliveryOwner::Entry | DeliveryOwner::RegistryFrame => {}
                DeliveryOwner::None => {
                    let cleanup = state.records.get_mut(&interest.key).and_then(|record| {
                        let DemandState::Output(output) = &mut record.state else { return None; };
                        if !matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Published { .. })) {
                            return None;
                        }
                        output.stop(WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed, ""));
                        record.wake.notify();
                        record.work_membership.as_ref().map(|member| Arc::clone(member.key_arc()))
                    });
                    if let Some(key) = cleanup {
                        state.defer_terminal_cleanup(&key, 0);
                    }
                }
            }
        }
        // The successor's Drop releases its interest after this guard ends.
    }
}

fn delivery_owner(state: &DemandRegistryState, key: &WorthQueryOutputDemandKey) -> DeliveryOwner {
    let Some(record) = state.records.get(key) else {
        return DeliveryOwner::None;
    };
    if record.held_successor.is_some() {
        return DeliveryOwner::RegistryFrame;
    }
    if record.interests > 1 {
        return DeliveryOwner::Entry;
    }
    // A stale interested entry follows the newest row through rejoin_refreshed
    // and completes its checkpoint before selection. Required counts alone
    // retain truth, but cannot own that executable entry.
    if record.readmission_source.is_some()
        && super::admission::accepts_semantic_join(record)
        && occurrence_rows(&state.records, key).any(|(other, row)| {
            refresh_order(other, key) == Some(std::cmp::Ordering::Less)
                && row.interests != 0
                && superseded(row)
        })
    {
        DeliveryOwner::Entry
    } else {
        DeliveryOwner::None
    }
}
