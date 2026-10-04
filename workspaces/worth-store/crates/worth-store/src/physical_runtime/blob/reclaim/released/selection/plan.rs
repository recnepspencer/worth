use sha2::{Digest, Sha256};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use crate::physical_runtime::PhysicalRecordReader;

use super::super::super::{scan, BlobReclaimDeferral, BlobReclaimFailure, BlobReclaimLimits};
use super::chain::ValidatedReleaseChain;
use super::graph_types::{
    is_exclusive_target, own_frontier, released_with_session, target_session,
};
use super::inventory::{SelectedBlobFact, SelectedReleaseFact, SelectedReleaseInventory};
use super::transcript;

pub(super) struct PlannedReleaseDrop {
    pub(super) dropped: Vec<PersistedRecordIdentity>,
    pub(super) remaining: u64,
    pub(super) cumulative_dropped: u64,
    pub(super) terminal: bool,
    pub(super) postorder_drop_sha256: [u8; 32],
}

pub(super) fn select_post_order(
    reader: &PhysicalRecordReader,
    inventory: &SelectedReleaseInventory,
    chain: &ValidatedReleaseChain,
    publication_referenced: bool,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<PlannedReleaseDrop, BlobReclaimFailure> {
    let capacity = usize::from(limits.maximum_dropped_records());
    let mut dropped = Vec::new();
    let mut ordered = Sha256::new();
    ordered.update(transcript::POSTORDER_DOMAIN);
    transcript::record_id(&mut ordered, inventory.basis.publication_record());
    dropped
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if dropped.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let session = inventory.basis.session();
    let owned = inventory
        .facts
        .iter()
        .filter(|fact| released_with_session(fact, session))
        .count();
    let unconnected = inventory
        .facts
        .iter()
        .filter(|fact| target_session(fact, session) && !fact.reachable && !fact.protected)
        .count();
    // A release can only drop records authenticated by its publication tree.
    // Same-session records outside that closure need a separate residue fate;
    // dropping the publication now would strand their custody permanently.
    if unconnected != 0 && !chain.terminal {
        return Err(BlobReclaimFailure::Deferred(
            BlobReclaimDeferral::UnconnectedSessionResidue,
        ));
    }
    if chain.terminal {
        if inventory.publication_selected || owned != 0 || unconnected != 0 {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        return Ok(PlannedReleaseDrop {
            dropped,
            remaining: 0,
            cumulative_dropped: chain.cumulative_dropped,
            terminal: true,
            postorder_drop_sha256: {
                ordered.update(0_u64.to_le_bytes());
                ordered.finalize().into()
            },
        });
    }
    if inventory.publication_selected {
        if publication_referenced {
            return Err(BlobReclaimFailure::Deferred(
                BlobReclaimDeferral::SharedReferences,
            ));
        }
        push_drop(
            &mut dropped,
            &mut ordered,
            0,
            inventory.basis.publication_record(),
        );
    }
    // A selected frontier names a selected chunk prefix, so the session's
    // frontiers leave before any chunk: a batch too small for every frontier
    // takes no chunk. The stage byte is a transcript tag, not a position.
    for fact in &inventory.facts {
        if dropped.len() == capacity {
            break;
        }
        if own_frontier(fact, session) {
            push_drop(&mut dropped, &mut ordered, 3, fact.record);
        }
    }
    // Leaves precede their parents even in a publication's first batch. A
    // shared source chunk keeps every ancestor selected for historical reads.
    for fact in &inventory.facts {
        if dropped.len() == capacity {
            break;
        }
        if is_exclusive_target(fact, session)
            && matches!(
                fact.blob,
                SelectedBlobFact::Chunk { .. } | SelectedBlobFact::ReuseClaim { .. }
            )
        {
            push_drop(&mut dropped, &mut ordered, 1, fact.record);
        }
    }
    for level in 0..=u8::MAX {
        for fact in &inventory.facts {
            if dropped.len() == capacity {
                break;
            }
            if !is_exclusive_target(fact, session)
                || !matches!(fact.blob, SelectedBlobFact::Tree { level: value, .. } if value == level)
            {
                continue;
            }
            if tree_children_settled(
                reader, inventory, chain, fact, &dropped, limits, scratch, work,
            )? {
                push_drop(&mut dropped, &mut ordered, 2, fact.record);
            }
        }
        if dropped.len() == capacity {
            break;
        }
    }
    if dropped.is_empty() {
        return Err(BlobReclaimFailure::Deferred(
            BlobReclaimDeferral::SharedReferences,
        ));
    }
    let selected_target = dropped
        .iter()
        .filter(|record| {
            inventory
                .fact(**record)
                .is_some_and(|fact| released_with_session(fact, session))
        })
        .count();
    let remaining = owned
        .checked_sub(selected_target)
        .and_then(|value| value.checked_add(unconnected))
        .ok_or(BlobReclaimFailure::ScanBoundExhausted)? as u64;
    let terminal = remaining == 0;
    ordered.update((dropped.len() as u64).to_le_bytes());
    let postorder_drop_sha256 = ordered.finalize().into();
    dropped.sort_unstable();
    let cumulative_dropped = chain
        .cumulative_dropped
        .checked_add(dropped.len() as u64)
        .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
    Ok(PlannedReleaseDrop {
        dropped,
        remaining,
        cumulative_dropped,
        terminal,
        postorder_drop_sha256,
    })
}

fn push_drop(
    dropped: &mut Vec<PersistedRecordIdentity>,
    ordered: &mut Sha256,
    stage: u8,
    record: PersistedRecordIdentity,
) {
    ordered.update([stage]);
    transcript::record_id(ordered, record);
    dropped.push(record);
}

fn tree_children_settled(
    reader: &PhysicalRecordReader,
    inventory: &SelectedReleaseInventory,
    chain: &ValidatedReleaseChain,
    tree: &SelectedReleaseFact,
    selected: &[PersistedRecordIdentity],
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<bool, BlobReclaimFailure> {
    work.records = work
        .records
        .checked_add(1)
        .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
    let used = scan::read_selected(
        reader,
        tree.record,
        tree.payload_bytes,
        scratch,
        limits,
        work,
    )?;
    let bytes = &scratch[..used];
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    if digest != tree.frame_sha256 {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let BlobRecordV1::TreeNode(node) =
        decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?
    else {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    };
    for entry in node.entries() {
        match inventory.fact(entry.record()) {
            Some(child) if !target_session(child, inventory.basis.session()) => {}
            Some(child) if child.protected => return Ok(false),
            Some(child) if selected.contains(&child.record) => {}
            Some(_) => return Ok(false),
            None if chain.settled_absence_authority => {}
            None => return Err(BlobReclaimFailure::ConflictingSelectedFate),
        }
    }
    Ok(true)
}
