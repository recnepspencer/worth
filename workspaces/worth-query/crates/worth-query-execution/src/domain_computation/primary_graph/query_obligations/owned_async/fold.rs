//! Fold descriptive request custody; dropping a caller is not a release transition.
use super::super::{QueryObligationDenial as Denial, QueryObligationLedger};
use super::*;
use std::num::NonZeroU64;
impl QueryObligationLedger {
    pub fn owned_async(&self) -> impl Iterator<Item = &QueryOwnedAsyncObligation> {
        self.owned_async.values()
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn validate_owned_async(
        &self,
        record: &QueryOwnedAsyncRecord,
        position: u64,
    ) -> Result<(), Denial> {
        let identity = match record {
            QueryOwnedAsyncRecord::Retain(row) => row.identity,
            QueryOwnedAsyncRecord::Progress { identity, .. }
            | QueryOwnedAsyncRecord::ProcessLocalReleased { identity, .. }
            | QueryOwnedAsyncRecord::Discharge { identity }
            | QueryOwnedAsyncRecord::QueueDrained { identity } => *identity,
        };
        if identity.home() != self.home() {
            return Err(Denial::ForeignHome);
        }
        match record {
            QueryOwnedAsyncRecord::Retain(row) => {
                if identity.acquire_position().get() != position
                    || self.owned_async.contains_key(&identity)
                    || row.declaration.identity.is_empty()
                    || row.original != row.current
                    || !row.lineage.is_empty()
                    || row.posture != QueryOwnedAsyncPosture::InFlight
                    || row.completion.is_some()
                    || (row.declaration.retention == QueryOwnedAsyncRetention::ProcessLocal
                        && row.declaration.managed_queue_capacity.is_none())
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                validate_queue(row.declaration.managed_queue_capacity, row.queue_depth)?;
            }
            QueryOwnedAsyncRecord::Progress {
                current,
                lineage,
                posture,
                queue_depth,
                completion,
                ..
            } => {
                let row = self
                    .owned_async
                    .get(&identity)
                    .ok_or(Denial::UnknownObligation)?;
                if matches!(row.posture, QueryOwnedAsyncPosture::ProcessLocalReleased(_))
                    || matches!(row.posture, QueryOwnedAsyncPosture::Terminal(_))
                    || matches!(posture, QueryOwnedAsyncPosture::ProcessLocalReleased(_))
                    || if *current == row.current {
                        lineage != &row.lineage
                    } else {
                        lineage.len() != row.lineage.len() + 1
                            || !lineage.starts_with(&row.lineage)
                            || lineage.last() != Some(&row.current)
                    }
                    || matches!(
                        posture,
                        QueryOwnedAsyncPosture::Terminal(QueryOwnedAsyncTerminal::Fulfilled)
                    ) && completion.is_none()
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                validate_queue(row.declaration.managed_queue_capacity, *queue_depth)?;
            }
            QueryOwnedAsyncRecord::ProcessLocalReleased { terminal, .. } => {
                let row = self
                    .owned_async
                    .get(&identity)
                    .ok_or(Denial::UnknownObligation)?;
                if row.declaration.retention == QueryOwnedAsyncRetention::Mandatory {
                    return Err(Denial::OwnedAsyncMandatory { identity });
                }
                if row.queue_depth != 0 {
                    return Err(Denial::OwnedAsyncQueuePending { identity });
                }
                if matches!(row.posture, QueryOwnedAsyncPosture::Terminal(known) | QueryOwnedAsyncPosture::ProcessLocalReleased(known) if known != *terminal)
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryOwnedAsyncRecord::QueueDrained { .. } => {
                self.owned_async
                    .get(&identity)
                    .ok_or(Denial::UnknownObligation)?;
            }
            QueryOwnedAsyncRecord::Discharge { .. } => {
                let row = self
                    .owned_async
                    .get(&identity)
                    .ok_or(Denial::UnknownObligation)?;
                if !matches!(
                    row.continuation(),
                    QueryOwnedAsyncContinuation::Terminal(_)
                        | QueryOwnedAsyncContinuation::ReleasedProcessLocal(_)
                ) {
                    return Err(Denial::OwnedAsyncPending { identity });
                }
                if row.queue_depth != 0 {
                    return Err(Denial::OwnedAsyncQueuePending { identity });
                }
            }
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn apply_owned_async(
        &mut self,
        record: QueryOwnedAsyncRecord,
    ) {
        match record {
            QueryOwnedAsyncRecord::Retain(row) => {
                self.owned_async.insert(row.identity, row);
            }
            QueryOwnedAsyncRecord::Progress {
                identity,
                current,
                lineage,
                posture,
                queue_depth,
                completion,
            } => {
                let row = self
                    .owned_async
                    .get_mut(&identity)
                    .expect("validated owned-async custody");
                row.current = current;
                row.lineage = lineage;
                row.posture = posture;
                row.queue_depth = queue_depth;
                row.completion = completion;
            }
            QueryOwnedAsyncRecord::ProcessLocalReleased { identity, terminal } => {
                self.owned_async
                    .get_mut(&identity)
                    .expect("validated owned-async custody")
                    .posture = QueryOwnedAsyncPosture::ProcessLocalReleased(terminal);
            }
            QueryOwnedAsyncRecord::QueueDrained { identity } => {
                self.owned_async
                    .get_mut(&identity)
                    .expect("validated owned-async custody")
                    .queue_depth = 0;
            }
            QueryOwnedAsyncRecord::Discharge { identity } => {
                self.owned_async.remove(&identity);
            }
        }
    }
}
fn validate_queue(capacity: Option<NonZeroU64>, depth: u64) -> Result<(), Denial> {
    if depth > capacity.map_or(0, NonZeroU64::get) {
        Err(Denial::ObligationSubjectMismatch)
    } else {
        Ok(())
    }
}
