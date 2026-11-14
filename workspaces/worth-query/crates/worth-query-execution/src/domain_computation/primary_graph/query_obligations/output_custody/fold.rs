//! Fold required custody without treating optional interests as obligations.
use super::*;
use crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial as Denial;
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use worth_foundational::facade::HomeIdentity;

#[derive(Clone, Debug, Default)]
pub(in crate::domain_computation::primary_graph::query_obligations) struct OutputCustodyFold {
    pub accepted: BTreeMap<(String, String, (u32, u64, u32), [u8; 32]), QueryAcceptedOutputRecord>,
    pub sources: BTreeMap<DurableCompositeCommitIdentity, QueryOutputSourceObligation>,
    pub demands: BTreeMap<NonZeroU64, QueryOutputDemandObligation>,
    pub failures: BTreeMap<NonZeroU64, Vec<u8>>,
    pub completed: std::collections::BTreeSet<NonZeroU64>,
    pub subjects: BTreeMap<OutputDemandSubject, NonZeroU64>,
}
impl OutputCustodyFold {
    pub fn count(&self, revision: &CanonicalDigestId) -> usize {
        self.sources
            .values()
            .filter(|row| row.revision == *revision)
            .count()
            + self
                .demands
                .values()
                .filter(|row| {
                    row.revision == *revision && !self.completed.contains(&row.acquire_position)
                })
                .count()
    }
    pub fn validate_source(
        &self,
        record: &QueryOutputSourceRecord,
        home: HomeIdentity,
    ) -> Result<(), Denial> {
        match record {
            QueryOutputSourceRecord::Bind { commit, sources } => {
                let row = self.sources.get(commit).ok_or(Denial::UnknownObligation)?;
                if sources.windows(2).any(|pair| pair[0] >= pair[1])
                    || row.bindings.as_ref().is_some_and(|known| known != sources)
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryOutputSourceRecord::Retain(row) => {
                if row.commit.owner().home() != home {
                    return Err(Denial::ForeignHome);
                }
                if row.root.is_empty() || row.branch.is_empty() || row.intent.is_empty() {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                if self.sources.contains_key(&row.commit) {
                    return Err(Denial::DuplicateObligation);
                }
                if row.discovery_recipe.is_none()
                    && self.sources.values().any(|prior| {
                        prior.discovery_recipe.is_none()
                            && prior.bindings.is_none()
                            && prior.branch == row.branch
                            && prior.root == row.root
                            && prior.scope == row.scope
                            && prior.commit.ordinal() >= row.commit.ordinal()
                    })
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryOutputSourceRecord::Discharge { commit } => {
                if commit.owner().home() != home {
                    return Err(Denial::ForeignHome);
                }
                if !self.sources.contains_key(commit) {
                    return Err(Denial::UnknownObligation);
                }
            }
        }
        Ok(())
    }
    pub fn validate_demand(
        &self,
        record: &QueryOutputDemandRecord,
        home: HomeIdentity,
        position: u64,
    ) -> Result<(), Denial> {
        match record {
            QueryOutputDemandRecord::DomainDenied {
                acquire_position,
                failure,
            } => {
                if !self.demands.contains_key(acquire_position)
                    || self.completed.contains(acquire_position)
                {
                    return Err(Denial::UnknownObligation);
                }
                if failure.is_empty()
                    || self
                        .failures
                        .get(acquire_position)
                        .is_some_and(|prior| prior != failure)
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryOutputDemandRecord::Retain(row) => {
                if row.acquire_position.get() != position {
                    return Err(Denial::ObligationIdentityMismatch);
                }
                if row.basis.owner().home() != home {
                    return Err(Denial::ForeignHome);
                }
                if self.demands.contains_key(&row.acquire_position)
                    || self.subjects.contains_key(&row.subject())
                {
                    return Err(Denial::DuplicateObligation);
                }
                if row.family.is_empty() || row.producer.is_empty() || row.branch.is_empty() {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                if row
                    .checkpoint_prerequisites
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                self.validate_links(row, &row.performed_sources, &row.prerequisites)?;
            }
            QueryOutputDemandRecord::Links {
                acquire_position,
                performed_sources,
                prerequisites,
                checkpoint_prerequisites,
            } => {
                let row = self
                    .demands
                    .get(acquire_position)
                    .filter(|_| !self.completed.contains(acquire_position))
                    .ok_or(Denial::UnknownObligation)?;
                if checkpoint_prerequisites
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                self.validate_links(row, performed_sources, prerequisites)?;
            }
            QueryOutputDemandRecord::Discharge { acquire_position } => {
                if !self.demands.contains_key(acquire_position)
                    || self.completed.contains(acquire_position)
                {
                    return Err(Denial::UnknownObligation);
                }
            }
        }
        Ok(())
    }
    fn validate_links(
        &self,
        row: &QueryOutputDemandObligation,
        sources: &[DurableCompositeCommitIdentity],
        prerequisites: &[NonZeroU64],
    ) -> Result<(), Denial> {
        if prerequisites.windows(2).any(|pair| pair[0] >= pair[1])
            || sources.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(Denial::ObligationSubjectMismatch);
        }
        for source in sources {
            if let Some(source) = self.sources.get(source) {
                if source.branch != row.branch {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            } else if !row.performed_sources.contains(source) {
                return Err(Denial::UnknownObligation);
            }
        }
        for prerequisite in prerequisites {
            if *prerequisite == row.acquire_position {
                return Err(Denial::ObligationSubjectMismatch);
            }
            let prior = self
                .demands
                .get(prerequisite)
                .ok_or(Denial::UnknownObligation)?;
            if prior.branch != row.branch {
                return Err(Denial::ObligationSubjectMismatch);
            }
        }
        Ok(())
    }
    pub fn apply_source(&mut self, record: QueryOutputSourceRecord) {
        match record {
            QueryOutputSourceRecord::Bind { commit, sources } => {
                self.sources
                    .get_mut(&commit)
                    .expect("validated source")
                    .bindings = Some(sources);
            }
            QueryOutputSourceRecord::Retain(row) => {
                // A newer fixed source supersedes the same unbound scope, as
                // retain_performed_source does in the live registry. Bound
                // required work continues to own its original source.
                if row.discovery_recipe.is_none() {
                    self.sources.retain(|_, prior| {
                        prior.discovery_recipe.is_some()
                            || prior.bindings.is_some()
                            || prior.branch != row.branch
                            || prior.root != row.root
                            || prior.scope != row.scope
                            || prior.commit.ordinal() >= row.commit.ordinal()
                    });
                }
                self.sources.insert(row.commit, row);
            }
            QueryOutputSourceRecord::Discharge { commit } => {
                self.sources.remove(&commit);
            }
        }
    }
    pub fn apply_demand(&mut self, record: QueryOutputDemandRecord) {
        match record {
            QueryOutputDemandRecord::DomainDenied {
                acquire_position,
                failure,
            } => {
                self.failures.insert(acquire_position, failure);
            }
            QueryOutputDemandRecord::Retain(row) => {
                self.subjects.insert(row.subject(), row.acquire_position);
                self.demands.insert(row.acquire_position, row);
            }
            QueryOutputDemandRecord::Links {
                acquire_position,
                performed_sources,
                prerequisites,
                checkpoint_prerequisites,
            } => {
                let row = self
                    .demands
                    .get_mut(&acquire_position)
                    .expect("validated demand");
                row.performed_sources = performed_sources;
                row.prerequisites = prerequisites;
                row.checkpoint_prerequisites = checkpoint_prerequisites;
            }
            QueryOutputDemandRecord::Discharge { acquire_position } => {
                self.completed.insert(acquire_position);
                self.subjects
                    .remove(&self.demands[&acquire_position].subject());
            }
        }
    }
}
