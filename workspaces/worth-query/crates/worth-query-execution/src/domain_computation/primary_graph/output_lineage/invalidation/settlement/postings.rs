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
    admission::IndexAdmission,
    fact_key::FactPostingKey,
    fact_keys::{self, FactKeyProjectionStop},
    index_capacity,
    mark_state::{FactPosting, FullVerificationReason, MarkState},
    InvalidationEditAdmission,
};

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
        | Fact::AbsentField { .. }
        | Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. } => Some(3),
        Fact::SourceAdjacencyRevision { endpoints, .. } => endpoints.len().checked_add(2),
        Fact::Relation { .. } => Some(3),
        Fact::Adjacency { relations, .. }
        | Fact::WorkflowInstanceCapacity {
            instances: relations,
            ..
        } => relations.len().checked_mul(2)?.checked_add(2),
        Fact::IndexedEntitySelection { candidates, .. } => candidates.len().checked_add(2),
        Fact::WorkflowHistoryBasis { .. } => Some(0),
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
        let mut admission = meter.into_inner();
        let groups = group_projected(projected, maximum, &mut admission)?;
        let mut ordinals = OrdMap::new();
        let mut posting_payload_bytes = 0u64;
        for group in groups {
            admission.work(1)?;
            posting_payload_bytes = posting_payload_bytes
                .checked_add(key_payload_bytes(&group.key)?)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            admission.key_edit::<Arc<FactPostingKey>, OrdSet<usize>>(&group.key, ordinals.len())?;
            ordinals.insert(group.key, group.ordinals);
        }
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
        let Self {
            ordinals,
            posting_payload_bytes,
        } = self;
        for (key, own) in &ordinals {
            admission.work(1)?;
            admission.key_read(key, state.postings.len())?;
            let (stored, mut postings, new_key) = match state.postings.get_key_value(key) {
                Some((stored, prior)) => (Arc::clone(stored), prior.clone(), false),
                None => (Arc::clone(key), OrdSet::new(), true),
            };
            admission.work(1)?;
            let mut changed = false;
            for ordinal in own {
                admission.work(1)?;
                let posting = FactPosting {
                    settlement: Arc::clone(identity),
                    ordinal: *ordinal,
                };
                admission.ordered_read(postings.len())?;
                if !postings.contains(&posting) {
                    admission.ordered_edit::<FactPosting, ()>(postings.len())?;
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
                admission.key_edit::<Arc<FactPostingKey>, OrdSet<FactPosting>>(
                    &stored,
                    state.postings.len(),
                )?;
                state.postings.insert(stored, postings);
                if let Some(payload) = next_payload {
                    state.key_payload_bytes = payload;
                }
            }
        }
        Ok((ordinals, posting_payload_bytes))
    }
}

/// Sort only fixed-width fingerprints. Full equality inside each fingerprint
/// bucket is still mandatory: a hash never authorizes fact-key deduplication.
fn group_projected(
    mut projected: Vec<ProjectedKey>,
    maximum: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<Vec<PostingGroup>, CompanionPreflightStop> {
    for index in 1..projected.len() {
        let mut cursor = index;
        while cursor != 0 {
            admission.work(1)?;
            if projected[cursor - 1].hash <= projected[cursor].hash {
                break;
            }
            admission.work(1)?;
            projected.swap(cursor - 1, cursor);
            cursor -= 1;
        }
    }
    let mut groups: Vec<PostingGroup> = Vec::with_capacity(maximum);
    let mut bucket_start = 0usize;
    let mut preceding_hash = None;
    admission.work(
        u64::try_from(projected.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
    )?;
    for projection in projected {
        if preceding_hash != Some(projection.hash) {
            preceding_hash = Some(projection.hash);
            bucket_start = groups.len();
        }
        let mut matching = None;
        for index in bucket_start..groups.len() {
            admission.work(
                projection
                    .key
                    .comparison_work_bound()
                    .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
            )?;
            if projection.key.as_ref() == groups[index].key.as_ref() {
                matching = Some(index);
                break;
            }
        }
        if let Some(index) = matching {
            let ordinals = &mut groups[index].ordinals;
            admission.ordered_edit::<usize, ()>(ordinals.len())?;
            ordinals.insert(projection.ordinal);
        } else {
            admission.work(1)?;
            let mut ordinals = OrdSet::new();
            admission.ordered_edit::<usize, ()>(0)?;
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
