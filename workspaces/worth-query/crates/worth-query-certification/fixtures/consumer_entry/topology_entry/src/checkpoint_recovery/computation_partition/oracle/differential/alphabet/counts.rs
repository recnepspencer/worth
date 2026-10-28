//! Calls derived from semantic model differences, independent of retained state.
use super::super::super::{tree_work, OwnerCalls};
use super::*;
use std::collections::BTreeMap;

impl Model {
    pub(in super::super::super) fn partition_values(&self) -> BTreeMap<u32, f64> {
        self.groups()
            .into_iter()
            .map(|(key, members)| {
                let mut value = -0.0;
                if key.is_multiple_of(2) {
                    value += self.weights[usize::from(self.odd)];
                }
                for (_, entry) in members {
                    value += entry.value;
                }
                (key, value)
            })
            .collect()
    }

    fn groups(&self) -> BTreeMap<u32, Vec<(usize, ModelEntry)>> {
        let mut groups = BTreeMap::<_, Vec<_>>::new();
        for (number, entry) in self.entries.iter().enumerate() {
            if let Some(entry) = entry {
                groups
                    .entry(entry.region)
                    .or_default()
                    .push((number, *entry));
            }
        }
        groups
    }

    /// The calls a run makes after `prior`: an incremental run stops at the
    /// first refusing partition in canonical order.
    pub(in super::super::super) fn expected_calls(&self, prior: Option<&Self>) -> OwnerCalls {
        let prior = prior.filter(|p| p.odd == self.odd);
        let membership =
            prior.is_none_or(|p| {
                p.entries.iter().enumerate().any(|(n, old)| {
                    old.is_some() && self.entries.get(n).is_none_or(Option::is_none)
                }) || self.entries.iter().enumerate().any(|(n, e)| {
                    e.map(|e| e.binding) != p.entries.get(n).and_then(|e| e.map(|e| e.binding))
                })
            });
        let keys = self
            .entries
            .iter()
            .enumerate()
            .filter(|(n, e)| {
                e.is_some_and(|e| {
                    prior
                        .and_then(|p| p.entries.get(*n))
                        .and_then(|e| *e)
                        .is_none_or(|old| old.binding != e.binding || old.region != e.region)
                })
            })
            .count();
        let current = self.groups();
        let before = prior.map(Self::groups).unwrap_or_default();
        let dirty = current
            .iter()
            .filter(|(key, members)| {
                let same_members = before.get(key).is_some_and(|old| {
                    old.len() == members.len()
                        && old.iter().zip(*members).all(|((an, a), (bn, b))| {
                            an == bn
                                && a.binding == b.binding
                                && a.value.to_bits() == b.value.to_bits()
                                && a.work == b.work
                                && a.fault == b.fault
                        })
                });
                !same_members
                    || (key.is_multiple_of(2)
                        && prior.is_none_or(|p| {
                            p.weights[usize::from(p.odd)].to_bits()
                                != self.weights[usize::from(self.odd)].to_bits()
                        }))
            })
            .map(|(key, _)| *key)
            .collect::<BTreeSet<_>>();
        let mut ordered = current.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(key, _)| tree_work::identity(**key));
        // Courtroom full-run law: all kernels run before the canonical result prefix is judged.
        // Incremental calls stop at the affected canonical prefix.
        let mut kernels = 0;
        for (key, members) in ordered {
            kernels += usize::from(dirty.contains(key));
            if prior.is_some() && members.iter().any(|(_, e)| e.fault) {
                break;
            }
        }
        OwnerCalls {
            plans: usize::from(membership),
            keys,
            gathers: dirty.len(),
            kernels: if prior.is_none() {
                dirty.len()
            } else {
                kernels
            },
        }
    }
}
