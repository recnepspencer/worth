//! One projection owns each local posting key and all of its fact ordinals.
//! The global reverse index is edited once per unique key.

use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    sync::Arc,
};

use im::{OrdMap, OrdSet};
use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

use super::super::super::RecordedSettlementIdentity;
use super::super::{
    admission::{IndexAdmission, RetainedIndexAdmission},
    fact_key::FactPostingKey,
    fact_keys::{self, FactKeyProjectionStop},
    index_capacity,
    mark_state::{FactPosting, FullVerificationReason, MarkState},
    InvalidationEditAdmission,
};

mod ordering;

pub(super) struct PreparedPostingOrdinals {
    ordinals: OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
    posting_payload_bytes: u64,
}

struct ProjectedKey {
    hash: u64,
    key: Arc<FactPostingKey>,
    ordinal: usize,
}

struct PostingGroup {
    key: Arc<FactPostingKey>,
    ordinals: OrdSet<usize>,
}

/// This sizes only scratch backing. The semantic projection remains the sole
/// `fact_keys::visit` below, so an unsupported native address is still Full.
fn maximum_keys(fact: &WorthQueryApplicationObservedFact) -> Option<usize> {
    use WorthQueryApplicationObservedFact as Fact;
    match fact {
        Fact::SourceEntity { .. } | Fact::Entity { .. } => Some(1),
        Fact::SourceAspectRevision { .. }
        | Fact::SourceFieldRevision { .. }
        | Fact::Field { .. }
        | Fact::AbsentField { .. } => Some(3),
        Fact::SourceAdjacencyRevision { endpoints, .. } => endpoints.len().checked_add(2),
        Fact::Relation { .. } => Some(3),
        Fact::Adjacency { relations, .. } => relations.len().checked_mul(2)?.checked_add(2),
        Fact::IndexedEntitySelection { candidates, .. } => candidates.len().checked_add(2),
        Fact::RetiredOutputEntity { .. }
        | Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. }
        | Fact::WorkflowInstanceCapacity { .. }
        | Fact::WorkflowHistoryBasis { .. } => Some(0),
    }
}

impl PreparedPostingOrdinals {
    pub(super) fn prepare(
        facts: &[WorthQueryApplicationObservedFact],
        output_facts: Option<&[WorthQueryApplicationObservedFact]>,
        verification_requirement: &mut Option<FullVerificationReason>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, CompanionPreflightStop> {
        let mut maximum = 0usize;
        let fact_count = facts
            .len()
            .checked_add(output_facts.map_or(0, |items| items.len()))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.work(
            u64::try_from(fact_count).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        for fact in facts.iter().chain(output_facts.into_iter().flatten()) {
            maximum = maximum
                .checked_add(
                    maximum_keys(fact)
                        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
                )
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        }
        let scratch = maximum
            .checked_mul(std::mem::size_of::<ProjectedKey>())
            .and_then(|bytes| {
                bytes.checked_add(maximum.checked_mul(std::mem::size_of::<PostingGroup>())?)
            })
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.bytes(
            u64::try_from(scratch)
                .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut projected = Vec::with_capacity(maximum);
        let meter = RefCell::new(admission);
        let output = output_facts.into_iter().flatten();
        for (ordinal, fact) in facts.iter().chain(output).enumerate() {
            meter.borrow_mut().work(1)?;
            let result = fact_keys::visit(
                fact,
                |work, bytes| {
                    let mut admission = meter.borrow_mut();
                    admission.work(work as u64)?;
                    admission.bytes(bytes as u64)
                },
                |key| {
                    let mut admission = meter.borrow_mut();
                    admission.work(
                        key.comparison_work_bound()
                            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
                    )?;
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    key.hash(&mut hasher);
                    admission.bytes(
                        index_capacity::arc_bytes::<FactPostingKey>()
                            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
                    )?;
                    if projected.len() >= maximum {
                        return Err(CompanionPreflightStop::PreparationMemoryCounterOverflow);
                    }
                    admission.work(1)?;
                    projected.push(ProjectedKey {
                        hash: hasher.finish(),
                        key: Arc::new(key),
                        ordinal,
                    });
                    Ok(())
                },
            );
            match result {
                Ok(()) => {}
                Err(FactKeyProjectionStop::Admission(stop)) => return Err(stop),
                Err(FactKeyProjectionStop::FullVerificationRequired(reason)) => {
                    *verification_requirement = Some(reason);
                }
            }
        }
        let admission = meter.into_inner();
        let groups = group_projected(projected, maximum, admission)?;
        let mut ordinals = OrdMap::new();
        let mut posting_payload_bytes = 0u64;
        let mut ordinal_count = 0usize;
        for group in groups {
            admission.work(1)?;
            ordinal_count = ordinal_count
                .checked_add(group.ordinals.len())
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            posting_payload_bytes = posting_payload_bytes
                .checked_add(key_payload_bytes(&group.key)?)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            admission.fresh_key_insert::<Arc<FactPostingKey>, OrdSet<usize>>(
                &group.key,
                ordinals.len(),
            )?;
            ordinals.insert(group.key, group.ordinals);
        }
        // Only the final local map, its independently rooted ordinal sets and
        // surviving key allocations become row storage. Sorting/group scratch,
        // duplicate projected keys and nodes a fresh build split away do not.
        // These local trees are created empty here and are never shared while
        // they grow, so each insertion is admitted as fresh-build growth.
        let retained = index_capacity::retained_map_bytes::<Arc<FactPostingKey>, OrdSet<usize>>(
            ordinals.len(),
        )
        .and_then(|bytes| {
            bytes.checked_add(index_capacity::retained_forest_bytes::<usize, ()>(
                ordinal_count,
                ordinals.len(),
            )?)
        })
        .and_then(|bytes| bytes.checked_add(posting_payload_bytes))
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.record_index_bytes(retained)?;
        Ok(Self {
            ordinals,
            posting_payload_bytes,
        })
    }

    pub(super) fn install(
        self,
        state: &mut MarkState,
        identity: &Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(OrdMap<Arc<FactPostingKey>, OrdSet<usize>>, u64), CompanionPreflightStop> {
        use super::super::admission::PreparedTreeEdits;
        let Self {
            ordinals,
            posting_payload_bytes,
        } = self;
        let mut map_edits = PreparedTreeEdits::<Arc<FactPostingKey>, OrdSet<FactPosting>>::new();
        for (key, own) in &ordinals {
            admission.work(1)?;
            admission.key_read(key, state.postings.len())?;
            let (stored, mut postings, new_key) = match state.postings.get_key_value(key) {
                Some((stored, prior)) => (Arc::clone(stored), prior.clone(), false),
                None => (Arc::clone(key), OrdSet::new(), true),
            };
            admission.work(1)?;
            let mut changed = false;
            let mut set_edits = PreparedTreeEdits::<FactPosting, ()>::new();
            for ordinal in own {
                admission.work(1)?;
                let posting = FactPosting {
                    settlement: Arc::clone(identity),
                    ordinal: *ordinal,
                };
                admission.ordered_read(postings.len())?;
                if !postings.contains(&posting) {
                    set_edits.insert(postings.len(), admission)?;
                    let next_count = state
                        .posting_count
                        .checked_add(1)
                        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                    postings.insert(posting);
                    state.posting_count = next_count;
                    changed = true;
                }
            }
            if changed {
                set_edits.retain(postings.len(), admission)?;
                let next_payload = if new_key {
                    Some(
                        state
                            .key_payload_bytes
                            .checked_add(key_payload_bytes(&stored)?)
                            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
                    )
                } else {
                    None
                };
                map_edits.key_insert(&stored, state.postings.len(), admission)?;
                state.postings.insert(stored, postings);
                if let Some(payload) = next_payload {
                    state.key_payload_bytes = payload;
                }
            }
        }
        map_edits.retain(state.postings.len(), admission)?;
        Ok((ordinals, posting_payload_bytes))
    }
}

/// Order by fingerprint and exact key, then group adjacent equal keys. Even
/// colliding fingerprints take O(n log n) comparisons, not a bucket scan per
/// occurrence. Hashes never authorize deduplication; every ordinal survives.
fn group_projected(
    mut projected: Vec<ProjectedKey>,
    maximum: usize,
    admission: &mut impl IndexAdmission,
) -> Result<Vec<PostingGroup>, CompanionPreflightStop> {
    ordering::sort(&mut projected, admission)?;
    let mut groups: Vec<PostingGroup> = Vec::with_capacity(maximum);
    let mut preceding_hash = None;
    admission.work(
        u64::try_from(projected.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
    )?;
    for projection in projected {
        let mut equal_key = false;
        if preceding_hash == Some(projection.hash) {
            admission.work(super::super::admission::key_traversal_work(&projection.key))?;
            admission.work(
                projection
                    .key
                    .comparison_work_bound()
                    .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
            )?;
            equal_key = groups
                .last()
                .is_some_and(|group| group.key == projection.key);
        }
        preceding_hash = Some(projection.hash);
        if equal_key {
            let ordinals = &mut groups
                .last_mut()
                .expect("an equal key has a group")
                .ordinals;
            admission.fresh_insert::<usize, ()>(ordinals.len())?;
            ordinals.insert(projection.ordinal);
        } else {
            admission.work(1)?;
            let mut ordinals = OrdSet::new();
            admission.fresh_insert::<usize, ()>(0)?;
            ordinals.insert(projection.ordinal);
            groups.push(PostingGroup {
                key: projection.key,
                ordinals,
            });
        }
    }
    Ok(groups)
}

fn key_payload_bytes(key: &FactPostingKey) -> Result<u64, CompanionPreflightStop> {
    key.owned_payload_capacity_bytes()
        .and_then(|bytes| bytes.checked_add(index_capacity::arc_bytes::<FactPostingKey>()?))
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)
}

#[cfg(test)]
mod tests;
