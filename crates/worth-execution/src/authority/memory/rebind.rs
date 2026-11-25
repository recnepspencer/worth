//! Atomic lock-ordered rebinding of memory charges between serial lineages.

use std::sync::Arc;

use super::{MemoryLimitDenial, MemoryLimitLevel, SerialLedger, SerialMemoryBudget};

impl SerialMemoryBudget {
    /// Atomically transfers `held` bytes from `old` to `bytes` bytes on `new`.
    /// Ledgers shared between `old` and `new` are locked once in canonical
    /// issue order and count `held` as available room before checking ceilings.
    /// Any refusal leaves every ledger untouched.
    pub(in crate::authority) fn rebind_between(
        old: Option<&Self>,
        new: Option<&Self>,
        held: u64,
        bytes: u64,
    ) -> Result<(), MemoryLimitDenial> {
        match (old, new) {
            (None, None) => Ok(()),
            (Some(old_b), Some(new_b)) if old_b.same_lineage(new_b) => {
                if held == 0 && bytes == 0 {
                    Ok(())
                } else {
                    new_b.resize(held, bytes)
                }
            }
            (None, Some(new_b)) => {
                if bytes == 0 {
                    Ok(())
                } else {
                    new_b.resize(0, bytes)
                }
            }
            (Some(old_b), None) => {
                if held != 0 {
                    let _ = old_b.resize(held, 0);
                }
                Ok(())
            }
            (Some(old_b), Some(new_b)) => {
                if held == 0 && bytes == 0 {
                    return Ok(());
                }
                rebind_distinct_lineages(old_b, new_b, held, bytes)
            }
        }
    }
}

struct RebindEntry<'a> {
    ledger: &'a Arc<SerialLedger>,
    from: u64,
    to: u64,
    new_ancestor: Option<u32>,
}

fn rebind_distinct_lineages(
    old: &SerialMemoryBudget,
    new: &SerialMemoryBudget,
    held: u64,
    bytes: u64,
) -> Result<(), MemoryLimitDenial> {
    let mut entries: Vec<RebindEntry<'_>> =
        Vec::with_capacity(new.ledgers.len() + old.ledgers.len());
    for (ancestor, ledger) in new.ledgers.iter().enumerate() {
        let from = if old.ledgers.iter().any(|item| Arc::ptr_eq(item, ledger)) {
            held
        } else {
            0
        };
        entries.push(RebindEntry {
            ledger,
            from,
            to: bytes,
            new_ancestor: Some(ancestor as u32),
        });
    }
    for ledger in old.ledgers.iter() {
        if !entries
            .iter()
            .any(|entry| Arc::ptr_eq(entry.ledger, ledger))
        {
            entries.push(RebindEntry {
                ledger,
                from: held,
                to: 0,
                new_ancestor: None,
            });
        }
    }
    entries.sort_unstable_by(|a, b| a.ledger.issue_order.cmp(&b.ledger.issue_order));
    let mut locked: Vec<_> = entries
        .iter()
        .map(|entry| {
            (
                entry,
                entry
                    .ledger
                    .charged
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
        })
        .collect();
    // Refusal order remains innermost new policy first.
    for ancestor in 0..new.ledgers.len() {
        let ancestor = ancestor as u32;
        let (entry, guard) = locked
            .iter()
            .find(|(entry, _)| entry.new_ancestor == Some(ancestor))
            .expect("new lineage ledger is locked");
        let charged = **guard - entry.from;
        let level = MemoryLimitLevel::Policy { ancestor };
        if let Some(ceiling) = entry.ledger.ceiling {
            let admitted = ceiling.saturating_sub(charged);
            if entry.to > admitted {
                return Err(MemoryLimitDenial::refused(entry.to, admitted, level));
            }
        }
        charged
            .checked_add(entry.to)
            .ok_or_else(|| MemoryLimitDenial::overflow(entry.to, charged, level))?;
    }
    for (entry, guard) in &mut locked {
        **guard = **guard - entry.from + entry.to;
    }
    Ok(())
}
