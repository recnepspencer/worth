//! One bounded borrowed index of original output expectations.
//!
//! Reconstruction orders F facts once, then resolves P role/aspect probes in
//! O((F + P) log F) comparisons. Conflicting duplicate keys remain unavailable;
//! neither an arbitrary duplicate nor the current graph supplies expectations.

use super::*;
use std::cmp::Ordering;

#[derive(Clone, Copy)]
enum Key<'a> {
    Entity(EntityId),
    Aspect(EntityId, &'a AspectKey),
}

#[derive(Clone, Copy)]
enum Value {
    Entity(KindId),
    Aspect(Option<u64>),
}

#[derive(Clone, Copy)]
struct Entry<'a> {
    key: Key<'a>,
    value: Value,
    conflicting: bool,
}

pub(super) struct CheckpointFactIndex<'a> {
    entries: Vec<Entry<'a>>,
}

impl<'a> CheckpointFactIndex<'a> {
    pub(super) fn prepare(
        facts: &'a [Fact],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let mut count = 0usize;
        for fact in facts {
            admission.charge_external_work(1)?;
            if entry(fact).is_some() {
                count = count.checked_add(1).ok_or_else(overflow)?;
            }
        }
        let bytes = count
            .checked_mul(size_of::<Entry<'_>>())
            .ok_or_else(overflow)?;
        admission.admit_read_scratch(u64::try_from(bytes).map_err(|_| overflow())?)?;
        let mut entries = Vec::new();
        if entries.try_reserve_exact(count).is_err() {
            // No comparison proof was constructed. Preserve the existing
            // unavailable-witness posture instead of panicking or inventing
            // a numerical admission limit for an allocator refusal.
            return Ok(None);
        }
        for fact in facts {
            admission.charge_external_work(1)?;
            if let Some(entry) = entry(fact) {
                admission.charge_external_work(1)?;
                entries.push(entry);
            }
        }
        sort(&mut entries, admission)?;
        let mut unique = 0usize;
        for read in 0..entries.len() {
            admission.charge_external_work(1)?;
            if unique > 0
                && compare(entries[unique - 1].key, entries[read].key, admission)?
                    == Ordering::Equal
            {
                admission.charge_external_work((2 * size_of::<Value>() + 1) as u64)?;
                let same = match (entries[unique - 1].value, entries[read].value) {
                    (Value::Entity(a), Value::Entity(b)) => a == b,
                    (Value::Aspect(a), Value::Aspect(b)) => a == b,
                    _ => unreachable!("equal keys have the same fact kind"),
                };
                entries[unique - 1].conflicting |= !same;
            } else {
                admission.charge_external_work(1)?;
                entries[unique] = entries[read];
                unique += 1;
            }
        }
        entries.truncate(unique);
        Ok(Some(Self { entries }))
    }

    pub(super) fn entity_matches(
        &self,
        entity: EntityId,
        expected: KindId,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        let found = self.find(Key::Entity(entity), admission)?;
        admission.charge_external_work((2 * size_of::<KindId>() + 1) as u64)?;
        Ok(matches!(found, Some(Value::Entity(kind)) if kind == expected))
    }

    pub(super) fn aspect_revision(
        &self,
        entity: EntityId,
        aspect: &AspectKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Option<u64>>, CompanionPreflightStop> {
        Ok(match self.find(Key::Aspect(entity, aspect), admission)? {
            Some(Value::Aspect(revision)) => Some(revision),
            _ => None,
        })
    }

    fn find(
        &self,
        key: Key<'_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Value>, CompanionPreflightStop> {
        let mut lower = 0usize;
        let mut upper = self.entries.len();
        while lower < upper {
            admission.charge_external_work(1)?;
            let middle = lower + (upper - lower) / 2;
            let entry = self.entries[middle];
            match compare(entry.key, key, admission)? {
                Ordering::Less => lower = middle + 1,
                Ordering::Greater => upper = middle,
                Ordering::Equal => return Ok((!entry.conflicting).then_some(entry.value)),
            }
        }
        Ok(None)
    }
}

fn entry(fact: &Fact) -> Option<Entry<'_>> {
    let (key, value) = match fact {
        Fact::Entity { entity_id, kind } => (Key::Entity(*entity_id), Value::Entity(*kind)),
        Fact::SourceAspectRevision {
            entity_id,
            aspect,
            native_revision,
        } => (
            Key::Aspect(*entity_id, aspect),
            Value::Aspect(*native_revision),
        ),
        _ => return None,
    };
    Some(Entry {
        key,
        value,
        conflicting: false,
    })
}

fn compare(
    left: Key<'_>,
    right: Key<'_>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Ordering, CompanionPreflightStop> {
    admission.charge_external_work((2 * size_of::<EntityId>() + 1) as u64)?;
    let entity = |key| match key {
        Key::Entity(id) | Key::Aspect(id, _) => id,
    };
    let ordering = entity(left).cmp(&entity(right));
    if ordering != Ordering::Equal {
        return Ok(ordering);
    }
    admission.charge_external_work(1)?;
    Ok(match (left, right) {
        (Key::Entity(_), Key::Entity(_)) => Ordering::Equal,
        (Key::Entity(_), Key::Aspect(..)) => Ordering::Less,
        (Key::Aspect(..), Key::Entity(_)) => Ordering::Greater,
        (Key::Aspect(_, a), Key::Aspect(_, b)) => {
            let bytes = a
                .as_str()
                .len()
                .checked_add(b.as_str().len())
                .and_then(|n| n.checked_add(1))
                .ok_or_else(overflow)?;
            admission.charge_external_work(u64::try_from(bytes).map_err(|_| overflow())?)?;
            a.as_str().cmp(b.as_str())
        }
    })
}

// Fallible in-place heapsort makes both comparison and movement admission
// explicit. Library sorting cannot stop before an unaffordable comparison.
fn sort(
    entries: &mut [Entry<'_>],
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    for root in (0..entries.len() / 2).rev() {
        sift(entries, root, entries.len(), admission)?;
    }
    for end in (1..entries.len()).rev() {
        admission.charge_external_work(2)?;
        entries.swap(0, end);
        sift(entries, 0, end, admission)?;
    }
    Ok(())
}

fn sift(
    entries: &mut [Entry<'_>],
    mut root: usize,
    end: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    loop {
        admission.charge_external_work(1)?;
        let child = root
            .checked_mul(2)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(overflow)?;
        if child >= end {
            return Ok(());
        }
        let mut largest = child;
        if child + 1 < end
            && compare(entries[child].key, entries[child + 1].key, admission)? == Ordering::Less
        {
            largest = child + 1;
        }
        if compare(entries[root].key, entries[largest].key, admission)? != Ordering::Less {
            return Ok(());
        }
        admission.charge_external_work(2)?;
        entries.swap(root, largest);
        root = largest;
    }
}

pub(super) fn preparation_work_bound(
    facts: u64,
    longest_aspect: u64,
) -> Result<u64, CompanionPreflightStop> {
    let levels = u64::from(u64::BITS - facts.leading_zeros()) + 1;
    let comparison = comparison_work_bound(longest_aspect)?;
    let sift = comparison.checked_add(4).ok_or_else(overflow)?;
    let coalesce = comparison
        .checked_add((2 * size_of::<Value>()) as u64 + 8)
        .ok_or_else(overflow)?;
    // At most F heap-build and F removal sifts, two comparisons per level;
    // counting/copying, duplicate coalescing and all movements are included.
    facts
        .checked_mul(levels)
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_mul(sift))
        .and_then(|n| n.checked_add(facts.checked_mul(coalesce)?))
        .ok_or_else(overflow)
}

pub(super) fn lookup_work_bound(
    facts: u64,
    longest_aspect: u64,
) -> Result<u64, CompanionPreflightStop> {
    let levels = u64::from(u64::BITS - facts.leading_zeros()) + 1;
    levels
        .checked_mul(
            comparison_work_bound(longest_aspect)?
                .checked_add(1)
                .ok_or_else(overflow)?,
        )
        .and_then(|n| n.checked_add((2 * size_of::<KindId>() + 1) as u64))
        .ok_or_else(overflow)
}

fn comparison_work_bound(longest_aspect: u64) -> Result<u64, CompanionPreflightStop> {
    longest_aspect
        .checked_mul(2)
        .and_then(|n| n.checked_add((2 * size_of::<EntityId>() + 3) as u64))
        .ok_or_else(overflow)
}
