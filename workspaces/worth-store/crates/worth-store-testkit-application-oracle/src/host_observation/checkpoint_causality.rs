//! Descriptive count-trigger comparison, not checkpoint or coverage authority.
use worth_store_application::{
    ApplicationHomeCheckpointInspection as Inspection, ApplicationHomeCheckpointLimits as Limits,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointCausalityDivergence {
    PriorAttemptPending,
    PriorAlreadyDue,
    JournalRosterChanged,
    CoverageAheadOfHead { journal: usize },
    HeadRegressed { journal: usize },
    CoverageRegressed { journal: usize },
    PriorCountAlreadyDue { journal: usize },
    CrossingNotAcknowledged { journal: usize },
    NoCountCrossing,
    ResultAttemptPending,
    ResultStillDue,
}

/// A comparison result only. It cannot select or acknowledge a checkpoint.
#[derive(Debug, Eq, PartialEq)]
pub struct CountCheckpointCrossing {
    pub journal: &'static str,
    pub prior_covered: u64,
    pub prior_head: u64,
    pub crossing_head: u64,
    pub acknowledged: u64,
}

/// The caller must bracket exactly one ordinary mutation, with no explicit
/// checkpoint, close or other publisher between these actual observations.
/// A selected prefix alone cannot establish that this operation triggered it.
pub fn compare_count_triggered_mutation(
    before: &Inspection,
    after: &Inspection,
    limits: Limits,
) -> Result<Vec<CountCheckpointCrossing>, CheckpointCausalityDivergence> {
    use CheckpointCausalityDivergence as D;
    if before.pending.is_some() {
        return Err(D::PriorAttemptPending);
    }
    if before.due {
        return Err(D::PriorAlreadyDue);
    }
    if after.pending.is_some() {
        return Err(D::ResultAttemptPending);
    }
    if after.due {
        return Err(D::ResultStillDue);
    }
    let mut crossings = Vec::new();
    for (journal, (b, a)) in before.journals.iter().zip(&after.journals).enumerate() {
        if b.name != a.name {
            return Err(D::JournalRosterChanged);
        }
        if b.covered > b.head || a.covered > a.head {
            return Err(D::CoverageAheadOfHead { journal });
        }
        if a.head < b.head {
            return Err(D::HeadRegressed { journal });
        }
        if a.covered < b.covered {
            return Err(D::CoverageRegressed { journal });
        }
        if b.head - b.covered >= limits.entries().get() {
            return Err(D::PriorCountAlreadyDue { journal });
        }
        if a.head - b.covered >= limits.entries().get() {
            if a.covered != a.head {
                return Err(D::CrossingNotAcknowledged { journal });
            }
            crossings.push(CountCheckpointCrossing {
                journal: b.name,
                prior_covered: b.covered,
                prior_head: b.head,
                crossing_head: a.head,
                acknowledged: a.covered,
            });
        }
    }
    if crossings.is_empty() {
        return Err(D::NoCountCrossing);
    }
    Ok(crossings)
}
