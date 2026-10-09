//! Calls derived from semantic model differences, independent of retained state.
use super::super::super::{tree_work, OwnerCalls, TOTALS_WORK};
use super::*;
use std::collections::BTreeMap;

impl Model {
    pub(in super::super::super) fn expected_observation_calls(&self) -> OwnerCalls {
        let incoming = self
            .entries
            .iter()
            .flatten()
            .filter(|entry| entry.incoming)
            .count();
        let partitions = self
            .entries
            .iter()
            .flatten()
            .map(|entry| {
                entry.region
                    + 2 * self.len() as u32
                    + if entry.incoming {
                        (OBSERVATION_SETS * incoming) as u32
                    } else {
                        0
                    }
            })
            .collect::<BTreeSet<_>>()
            .len();
        OwnerCalls {
            plans: 1,
            keys: self.len(),
            gathers: partitions,
            kernels: partitions,
        }
    }

    pub(in super::super::super) fn expected_observation_calls_at(
        &self,
        prior: Option<&Self>,
        ceiling: usize,
    ) -> OwnerCalls {
        let transformed = self.observation_keys();
        let prior_keys = prior.map(Self::observation_keys);
        let mut expected = transformed.expected_calls_at(prior_keys.as_ref(), ceiling);
        // Every key observes the two shared sets' adjacency, including member
        // identity, not just the cardinality projected from that adjacency.
        let members = |model: &Self| {
            model
                .entries
                .iter()
                .flatten()
                .map(|entry| entry.binding)
                .collect::<BTreeSet<_>>()
        };
        if prior.is_none_or(|old| members(old) != members(self)) {
            expected.keys = self.len();
        }
        expected
    }
    pub(in super::super::super) fn observation_keys(&self) -> Self {
        let mut now = self.clone();
        let members = self.len() as u32;
        let incoming = self
            .entries
            .iter()
            .flatten()
            .filter(|entry| entry.incoming)
            .count() as u32;
        for entry in now.entries.iter_mut().flatten() {
            entry.region += 2 * members
                + if entry.incoming {
                    OBSERVATION_SETS as u32 * incoming
                } else {
                    0
                };
        }
        now
    }
    pub(in super::super::super) fn with_eviction_entries(&self, count: usize) -> Self {
        let mut now = self.clone();
        if count == 0 {
            return now;
        }
        now.entries.resize(1000 + count, None);
        for item in 0..count {
            now.entries[1000 + item] = Some(ModelEntry {
                binding: u64::MAX - item as u64,
                region: 1000 + item as u32,
                incoming: false,
                value: 1.0,
                work: 1,
                fault: false,
            });
        }
        now
    }
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

    /// A successful run has ample preparation slack. With two heavy entries,
    /// their combined kernel work alone exceeds the ceiling; exactly the
    /// second heavy partition is the stop, irrespective of preparation slack.
    pub(in super::super::super) fn expected_calls(&self, prior: Option<&Self>) -> OwnerCalls {
        self.expected_calls_at(prior, TOTALS_WORK)
    }

    pub(in super::super::super) fn expected_calls_at(
        &self,
        prior: Option<&Self>,
        ceiling: usize,
    ) -> OwnerCalls {
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
        let mut kernel_work = 0;
        let mut kernels = 0;
        for (key, members) in ordered {
            kernels += usize::from(dirty.contains(key));
            kernel_work +=
                members.iter().map(|(_, e)| e.work).sum::<u64>() + u64::from(key.is_multiple_of(2));
            if (prior.is_some() && members.iter().any(|(_, e)| e.fault))
                || kernel_work > ceiling as u64
            {
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
