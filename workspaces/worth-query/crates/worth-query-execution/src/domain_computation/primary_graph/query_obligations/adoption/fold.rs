//! Branch adoption custody retains both canonical program meanings exactly once.
use super::*;
#[derive(Clone, Debug, Default)]
pub(in crate::domain_computation::primary_graph::query_obligations) struct AdoptionFold {
    pub stopped:
        BTreeMap<DurableCompositePublicationAttemptIdentity, QueryStoppedBranchSetObligation>,
    pub completed: std::collections::BTreeSet<DurableCompositePublicationAttemptIdentity>,
    pub branches:
        BTreeMap<DurableCompositePublicationAttemptIdentity, QueryBranchAdoptionObligation>,
}
impl AdoptionFold {
    pub fn count(&self, revision: &CanonicalDigestId) -> usize {
        self.branches
            .values()
            .filter(|row| {
                !self.completed.contains(&row.attempt)
                    && (row.source == *revision || row.target == *revision)
            })
            .count()
            + self
                .stopped
                .values()
                .map(|row| {
                    row.remaining
                        .iter()
                        .filter(|(_, _, source)| *source == *revision || row.target == *revision)
                        .count()
                })
                .sum::<usize>()
    }
    pub fn mentions(&self, record: &QueryAdoptionRecord, revision: &CanonicalDigestId) -> bool {
        match record {
            QueryAdoptionRecord::RetainStopped(row) => {
                return row.target == *revision
                    || row
                        .remaining
                        .iter()
                        .any(|(_, _, source)| source == revision)
            }
            QueryAdoptionRecord::ReleaseStopped { attempt } => {
                return self.stopped.get(attempt).is_some_and(|row| {
                    row.target == *revision
                        || row
                            .remaining
                            .iter()
                            .any(|(_, _, source)| source == revision)
                })
            }
            _ => {}
        }
        let row = match record {
            QueryAdoptionRecord::RetainBranch(row) => Some(row),
            QueryAdoptionRecord::RetainStopped(_) | QueryAdoptionRecord::ReleaseStopped { .. } => {
                unreachable!()
            }
            QueryAdoptionRecord::ReleaseBranch { attempt } => self.branches.get(attempt),
        };
        row.is_some_and(|row| row.source == *revision || row.target == *revision)
    }
    pub fn validate(
        &self,
        record: &QueryAdoptionRecord,
        home: HomeIdentity,
    ) -> Result<(), QueryObligationDenial> {
        use QueryObligationDenial as Denial;
        match record {
            QueryAdoptionRecord::RetainStopped(row) => {
                if row.attempt.owner().home() != home {
                    return Err(Denial::ForeignHome);
                }
                if self.stopped.contains_key(&row.attempt) {
                    return Err(Denial::DuplicateObligation);
                }
                if row.remaining.is_empty()
                    || row.remaining.iter().any(|(name, _, _)| name.is_empty())
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                let mut seen = std::collections::BTreeSet::new();
                if row
                    .remaining
                    .iter()
                    .any(|(name, lifecycle, _)| !seen.insert((name, *lifecycle)))
                    || row.performed.iter().any(|p| {
                        p.commit.owner() != row.attempt.owner()
                            || p.branch.is_empty()
                            || seen.contains(&(&p.branch, p.lifecycle))
                    })
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryAdoptionRecord::ReleaseStopped { attempt } => {
                if attempt.owner().home() != home {
                    return Err(Denial::ForeignHome);
                }
                if !self.stopped.contains_key(attempt) {
                    return Err(Denial::UnknownObligation);
                }
            }
            QueryAdoptionRecord::RetainBranch(row) => {
                if row.attempt.owner().home() != home || row.expected.owner() != row.attempt.owner()
                {
                    return Err(Denial::ForeignHome);
                }
                if row.branch.is_empty() || row.generation == 0 {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                if self.branches.contains_key(&row.attempt) {
                    return Err(Denial::DuplicateObligation);
                }
            }
            QueryAdoptionRecord::ReleaseBranch { attempt } => {
                if attempt.owner().home() != home {
                    return Err(Denial::ForeignHome);
                }
                if !self.branches.contains_key(attempt) || self.completed.contains(attempt) {
                    return Err(Denial::UnknownObligation);
                }
            }
        }
        Ok(())
    }
    pub fn apply(&mut self, record: QueryAdoptionRecord) {
        match record {
            QueryAdoptionRecord::RetainStopped(row) => {
                self.stopped.insert(row.attempt, row);
            }
            QueryAdoptionRecord::ReleaseStopped { attempt } => {
                self.stopped.remove(&attempt);
            }
            QueryAdoptionRecord::RetainBranch(row) => {
                self.branches.insert(row.attempt, row);
            }
            QueryAdoptionRecord::ReleaseBranch { attempt } => {
                self.completed.insert(attempt);
            }
        }
    }
}
