//! Replay the pin journal into a fresh kernel-backed table.
use std::collections::BTreeMap;

use worth_store_physical_format::DurablePhysicalRootManifest;

use super::event::{RootPinEvent, MAXIMUM_PIN_ENTRY_BYTES};
use crate::physical_runtime::stability::PinnedPhysicalRoot;
use crate::physical_runtime::OrderedJournal;
use crate::pin_table::{RootPinCore, RootPinId, RootPinState};

/// The journal is damaged or unreadable; the caller keeps every root protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct RootPinJournalDamaged;

/// Replay entries `1..=head` in order through the same kernel transitions
/// that recorded them. Reader loans are process-local and start at zero.
///
/// Cost: O(journal entries); one bounded read each.
pub(in crate::physical_runtime) fn replay(
    journal: &OrderedJournal<'_>,
    prior_roots: Vec<DurablePhysicalRootManifest>,
) -> Result<RootPinCore<PinnedPhysicalRoot>, RootPinJournalDamaged> {
    let roots: BTreeMap<u64, DurablePhysicalRootManifest> = prior_roots
        .into_iter()
        .map(|root| (root.generation(), root))
        .collect();
    let head = journal
        .head()
        .map_err(|_| RootPinJournalDamaged)?
        .map_or(0, |entry| entry.position());
    let mut core = RootPinCore::default();
    for position in 1..=head {
        let entry = journal
            .read(position, MAXIMUM_PIN_ENTRY_BYTES)
            .map_err(|_| RootPinJournalDamaged)?;
        match RootPinEvent::decode(entry.bytes()).ok_or(RootPinJournalDamaged)? {
            RootPinEvent::Acquire {
                pin,
                root,
                policy,
                parent,
                inline_tail,
                payload,
            } => {
                let manifest = roots.get(&root).cloned();
                core.acquire_exact(
                    pin,
                    root,
                    parent,
                    policy,
                    PinnedPhysicalRoot::of(manifest, inline_tail, payload),
                )
                .map_err(|_| RootPinJournalDamaged)?;
            }
            RootPinEvent::Expiring { pin } => match core.state(pin).ok_or(RootPinJournalDamaged)? {
                RootPinState::Active => {
                    core.expire(pin).map_err(|_| RootPinJournalDamaged)?;
                }
                RootPinState::Expiring => {}
                RootPinState::Expired => return Err(RootPinJournalDamaged),
            },
            RootPinEvent::Expired { pin } => complete(&mut core, pin)?,
        }
    }
    Ok(core)
}

/// Drive a recorded `Expired` pin to `Expired`; it was drained when recorded.
fn complete(
    core: &mut RootPinCore<PinnedPhysicalRoot>,
    pin: RootPinId,
) -> Result<(), RootPinJournalDamaged> {
    for _ in 0..2 {
        match core.state(pin).ok_or(RootPinJournalDamaged)? {
            RootPinState::Expired => return Ok(()),
            RootPinState::Active | RootPinState::Expiring => {
                core.expire(pin).map_err(|_| RootPinJournalDamaged)?;
            }
        }
    }
    match core.state(pin) {
        Some(RootPinState::Expired) => Ok(()),
        _ => Err(RootPinJournalDamaged),
    }
}
