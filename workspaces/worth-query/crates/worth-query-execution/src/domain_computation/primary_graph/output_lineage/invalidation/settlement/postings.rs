//! Exact fact-ordinal projection into the one physical reverse index.
use super::super::fact_key::FactPostingKey;
use super::super::fact_keys;
use super::super::fact_keys::FactKeyProjectionStop;
use super::super::mark_state::FullVerificationReason;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use im::OrdMap;
use im::OrdSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop;
mod ordering;
pub(in super::super) struct PreparedPostingOrdinals {
    ordinals: OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
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
impl PreparedPostingOrdinals {
    pub(in super::super) fn prepare(
        facts: &(impl FactSequence + ?Sized),
        output_facts: Option<&[WorthQueryApplicationObservedFact]>,
        verification_requirement: &mut Option<FullVerificationReason>,
    ) -> Result<Self, CompanionPreflightStop> {
        let mut projected = Vec::new();
        for (ordinal, fact) in facts
            .iter()
            .chain(output_facts.into_iter().flatten())
            .enumerate()
        {
            crate::domain_computation::primary_graph::fact_verification_observation::posting_fact();
            match fact_keys::visit(fact, |key| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                key.hash(&mut hasher);
                projected.push(ProjectedKey {
                    hash: hasher.finish(),
                    key: Arc::new(key),
                    ordinal,
                });
                Ok::<_, CompanionPreflightStop>(())
            }) {
                Ok(()) => {}
                Err(FactKeyProjectionStop::Admission(stop)) => return Err(stop),
                Err(FactKeyProjectionStop::FullVerificationRequired(reason)) => {
                    *verification_requirement = Some(reason)
                }
            }
        }
        let mut ordinals = OrdMap::new();
        for group in group_projected(projected)? {
            ordinals.insert(group.key, group.ordinals);
        }
        Ok(Self { ordinals })
    }
    pub(in super::super) fn into_query_parts(self) -> OrdMap<Arc<FactPostingKey>, OrdSet<usize>> {
        self.ordinals
    }
    pub(super) fn into_projection(
        self,
        id: u64,
        source_len: usize,
        output_len: usize,
        requirement: Option<FullVerificationReason>,
    ) -> Arc<super::PostingProjection> {
        Arc::new(super::PostingProjection {
            id,
            ordinals: self.ordinals,
            source_len,
            output_len,
            requirement,
        })
    }
}
/// Fingerprints order the sort; exact key equality alone groups ordinals.
fn group_projected(
    mut projected: Vec<ProjectedKey>,
) -> Result<Vec<PostingGroup>, CompanionPreflightStop> {
    ordering::sort(&mut projected);
    let mut groups: Vec<PostingGroup> = Vec::new();
    let mut preceding_hash = None;
    for item in projected {
        if preceding_hash == Some(item.hash)
            && groups.last().is_some_and(|group| *group.key == *item.key)
        {
            groups
                .last_mut()
                .expect("matched preceding group")
                .ordinals
                .insert(item.ordinal);
        } else {
            preceding_hash = Some(item.hash);
            groups.push(PostingGroup {
                key: item.key,
                ordinals: OrdSet::unit(item.ordinal),
            });
        }
    }
    Ok(groups)
}
#[cfg(test)]
mod tests;
