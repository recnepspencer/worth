//! Original backing and work admitted before retained normalization changes.
use super::*;

impl ComputationFactAttribution {
    pub(super) fn normalization_bytes(
        &self,
        handler: &BTreeSet<Key>,
        fallback: bool,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<u64, Cause> {
        let mut keys = 0usize;
        let mut payload = 0u64;
        let mut readers = 0usize;
        let mut expansion_work = 0u64;
        let mut expansion_keys = 0usize;
        for key in handler.iter().filter(|key| allowance(key).is_some()) {
            AggregateObservation::checkpoint(context)?;
            keys = keys.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
            payload = payload
                .checked_add(key.additional_charged_bytes())
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
        for (key, reads) in self
            .readers
            .iter()
            .filter(|(key, _)| allowance(key).is_some())
        {
            AggregateObservation::checkpoint(context)?;
            keys = keys.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
            payload = payload
                .checked_add(key.additional_charged_bytes())
                .ok_or(Cause::ChargedBytesOverflow)?;
            for _ in reads.reads() {
                AggregateObservation::checkpoint(context)?;
                readers = readers.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
            }
        }
        if fallback && let Some(native) = &self.native {
            let clone_count = if self.range_progress.is_some() {
                self.fresh_calls.len()
            } else {
                0
            };
            readers = readers
                .checked_add(clone_count)
                .ok_or(Cause::ChargedBytesOverflow)?;
            let mut calls = self.carried.len();
            if let Some(progress) = &self.range_progress {
                for _ in progress.carried(&self.fresh_calls) {
                    AggregateObservation::checkpoint(context)?;
                    calls = calls.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
                }
            }
            readers = readers
                .checked_add(calls)
                .ok_or(Cause::ChargedBytesOverflow)?;
            // Real call-index navigation, call-set insertion and the one
            // fresh-call clone performed by the existing fallback owner.
            let call_units = u64::try_from(
                calls
                    .checked_add(clone_count)
                    .ok_or(Cause::WorkCounterOverflow)?,
            )
            .ok()
            .and_then(|n| {
                n.checked_mul(
                    levels(native.facts().index.calls.len())
                        .checked_mul(2)?
                        .checked_add(
                            levels(self.calls.len().checked_add(calls)?).checked_mul(11)?,
                        )?,
                )
            })
            .ok_or(Cause::WorkCounterOverflow)?;
            checkpoint(context, call_units)?;
            for (key, fact) in native.facts().metadata.iter() {
                let mut count = 0usize;
                for read in fact.readers.reads() {
                    AggregateObservation::checkpoint(context)?;
                    if self.is_carried(read) {
                        count = count.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
                    }
                }
                if count != 0 {
                    // The genuine fact's sole locator emitter covers shared
                    // Native keys/addresses. Add owned key payload for names
                    // (such as entity aliases) absent from the fact locator.
                    let semantic = native
                        .facts()
                        .original
                        .get(fact.ordinal)
                        .expect("sealed original fact")
                        .dependency_locator_byte_bound()
                        .ok_or(Cause::WorkCounterOverflow)?
                        .checked_add(key.additional_charged_bytes())
                        .ok_or(Cause::WorkCounterOverflow)?;
                    expansion_work = expansion_work
                        .checked_add(
                            semantic
                                .checked_add(1)
                                .and_then(|n| n.checked_mul(u64::try_from(count).ok()?))
                                .ok_or(Cause::WorkCounterOverflow)?,
                        )
                        .ok_or(Cause::WorkCounterOverflow)?;
                    // Existing materialize_carried clones a key per actual
                    // reader; quote even duplicate entry temporaries first.
                    keys = keys.checked_add(count).ok_or(Cause::ChargedBytesOverflow)?;
                    expansion_keys = expansion_keys
                        .checked_add(count)
                        .ok_or(Cause::ChargedBytesOverflow)?;
                    payload = payload
                        .checked_add(
                            key.additional_charged_bytes()
                                .checked_mul(
                                    u64::try_from(count)
                                        .map_err(|_| Cause::ChargedBytesOverflow)?,
                                )
                                .ok_or(Cause::ChargedBytesOverflow)?,
                        )
                        .ok_or(Cause::ChargedBytesOverflow)?;
                    readers = readers
                        .checked_add(count)
                        .ok_or(Cause::ChargedBytesOverflow)?;
                }
            }
        }
        // Untouched keys add no payload quote, but they still determine the
        // actual target tree's possible search/split height.
        let target_count = handler
            .len()
            .checked_add(self.readers.len())
            .and_then(|n| n.checked_add(expansion_keys))
            .ok_or(Cause::ChargedBytesOverflow)?;
        // Each expansion clones one genuine key and descends an insertion
        // path. Admit all of that work BEFORE existing materialize_carried.
        checkpoint(
            context,
            expansion_work
                .checked_mul(
                    levels(target_count)
                        .checked_mul(11)
                        .and_then(|n| n.checked_add(1))
                        .ok_or(Cause::WorkCounterOverflow)?,
                )
                .ok_or(Cause::WorkCounterOverflow)?,
        )?;
        // Four simultaneous tree/key backings cover maxima, old/new handler,
        // old/new attribution and carried expansion. Each quoted insertion can
        // split every search level; reader sets and fresh-call clone are inline.
        let nodes = u64::try_from(keys)
            .ok()
            .and_then(|n| n.checked_mul(levels(target_count)))
            .and_then(|n| n.checked_mul(4))
            .ok_or(Cause::ChargedBytesOverflow)?;
        let node = 11usize
            .checked_mul(size_of::<Key>() + size_of::<ComputationFactReads>() + size_of::<usize>())
            .and_then(|n| n.checked_add(14 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let reader_nodes = u64::try_from(readers)
            .ok()
            .and_then(|n| n.checked_mul(levels(readers)))
            .and_then(|n| {
                n.checked_mul((22 * size_of::<ComputationRead>() + 28 * size_of::<usize>()) as u64)
            })
            .ok_or(Cause::ChargedBytesOverflow)?;
        nodes
            .checked_mul(node)
            .and_then(|n| n.checked_add(payload.checked_mul(4)?))
            .and_then(|n| n.checked_add(reader_nodes))
            .ok_or(Cause::ChargedBytesOverflow)
    }
}
