//! A fully verified V3 result may terminally consume an older WAL group.
//! The C9 operation fate remains Indeterminate; this is a separate, exact
//! post-verification disposition, never an ordinary page-LSN promotion.

use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::RecordFrameCoordinate;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalConsumedOperationSet {
    entries: Box<[HistoricalConsumedOperation]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HistoricalConsumedOperation {
    operation: [u8; 32],
    group: [u8; 32],
    descriptor_operation: [u8; 32],
}

impl HistoricalConsumedOperationSet {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.entries.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<HistoricalConsumedOperation>()).ok()?)
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, operation: [u8; 32]) -> bool {
        self.entries
            .binary_search_by_key(&operation, |entry| entry.operation)
            .is_ok()
    }

    pub fn descriptor_for(&self, operation: [u8; 32]) -> Option<[u8; 32]> {
        self.entries
            .binary_search_by_key(&operation, |entry| entry.operation)
            .ok()
            .map(|index| self.entries[index].descriptor_operation)
    }

    pub fn group_for(&self, operation: [u8; 32]) -> Option<[u8; 32]> {
        self.entries
            .binary_search_by_key(&operation, |entry| entry.operation)
            .ok()
            .map(|index| self.entries[index].group)
    }

    pub fn operations(&self) -> impl ExactSizeIterator<Item = [u8; 32]> + '_ {
        self.entries.iter().map(|entry| entry.operation)
    }

    pub fn memory_bytes(&self) -> u64 {
        (self.entries.len() * std::mem::size_of::<HistoricalConsumedOperation>()) as u64
    }
}

impl ImmutablePhysicalRedoPlan {
    /// Call only after the named V3 descriptors have passed full selected
    /// source, result, control, and WAL-fate verification. This checks every
    /// member and target of each affected older group before it can be
    /// treated as consumed by staging or handoff.
    pub fn admit_historical_consumed_operations(
        &self,
        verified_descriptors: &[[u8; 32]],
        selected_root_identity: [u8; 32],
    ) -> Option<HistoricalConsumedOperationSet> {
        let verified = verified_descriptors
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if verified.len() != verified_descriptors.len() {
            return None;
        }
        let mut groups = BTreeMap::<[u8; 32], Vec<&PhysicalRedoProjection>>::new();
        for projection in self.projections() {
            groups
                .entry(projection.group().group_identity())
                .or_default()
                .push(projection);
        }
        let mut entries = Vec::new();
        let mut historical_decisions = 0_u64;
        for (group_id, members) in groups {
            let affected = members.iter().any(|member| {
                self.decisions().iter().any(|decision| {
                    decision.operation() == member.operation()
                        && decision.kind()
                            == PhysicalRedoDecisionKind::SkipHistoricallyReleasedTarget
                })
            });
            if !affected {
                continue;
            }
            let binding = members.first()?.group();
            if members.len() != binding.member_count() as usize
                || members.iter().any(|member| {
                    member.group().group_identity() != group_id
                        || member.group().member_count() != binding.member_count()
                        || member.group().membership_digest() != binding.membership_digest()
                        || member.fate() != RecoveryOperationFate::Indeterminate
                })
            {
                return None;
            }
            let ordinals = members
                .iter()
                .map(|member| member.group().member_ordinal())
                .collect::<BTreeSet<_>>();
            if ordinals.len() != members.len() {
                return None;
            }
            let mut descriptor = None;
            for member in members {
                let decisions = self
                    .resolved_decisions()
                    .filter(|decision| decision.operation() == member.operation())
                    .collect::<Vec<_>>();
                if decisions.is_empty() {
                    return None;
                }
                for decision in decisions {
                    match (decision.kind(), decision.prior()) {
                        (
                            PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn,
                            PhysicalRedoDecisionPrior::Page(observation),
                        ) if !matches!(
                            observation.source(),
                            RecoveryPageSource::HistoricalReleasedDrop { .. }
                        ) => {}
                        (
                            PhysicalRedoDecisionKind::SkipHistoricallyReleasedTarget,
                            PhysicalRedoDecisionPrior::Page(observation),
                        ) => {
                            let RecoveryPageSource::HistoricalReleasedDrop {
                                coordinate,
                                selected_root_identity: observed_root,
                                descriptor_operation,
                                old_operation,
                                wal_target_digest,
                            } = observation.source()
                            else {
                                return None;
                            };
                            let target = decision.target();
                            if observed_root != selected_root_identity
                                || old_operation != member.operation()
                                || observation.target() != target.identity()
                                || wal_target_digest != target.resulting_digest()
                                || Some(coordinate)
                                    != RecordFrameCoordinate::new(
                                        target.artifact(),
                                        target.artifact_offset(),
                                        target.artifact_length(),
                                    )
                                || !verified.contains(&descriptor_operation)
                                || descriptor_operation == member.operation()
                                || descriptor.is_some_and(|prior| prior != descriptor_operation)
                            {
                                return None;
                            }
                            descriptor = Some(descriptor_operation);
                            historical_decisions = historical_decisions.checked_add(1)?;
                        }
                        _ => return None,
                    }
                }
            }
            let descriptor_operation = descriptor?;
            for member in self
                .projections()
                .iter()
                .filter(|member| member.group().group_identity() == group_id)
            {
                entries.push(HistoricalConsumedOperation {
                    operation: member.operation(),
                    group: group_id,
                    descriptor_operation,
                });
            }
        }
        if historical_decisions != self.counters().skip_historical_drop() {
            return None;
        }
        entries.sort_unstable_by_key(|entry| entry.operation);
        if entries
            .windows(2)
            .any(|pair| pair[0].operation == pair[1].operation)
        {
            return None;
        }
        Some(HistoricalConsumedOperationSet {
            entries: entries.into_boxed_slice(),
        })
    }
}
