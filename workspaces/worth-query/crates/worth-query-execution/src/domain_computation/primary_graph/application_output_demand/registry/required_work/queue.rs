//! One queue of actual required members; selected work remains a hint.

use super::*;

#[derive(Default)]
struct QueueState {
    head: Option<Arc<RequiredWorkMembership>>,
    tail: Option<Arc<RequiredWorkMembership>>,
    len: usize,
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct RequiredWorkQueue
{
    state: Mutex<QueueState>,
    pub(super) capacity: Arc<RecordCapacity>,
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) enum RequiredWorkPop
{
    Empty,
    Stale,
    Claimed(RequiredWorkClaim),
}

impl RequiredWorkQueue {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn new(
        capacity: Arc<RecordCapacity>,
    ) -> Self {
        Self {
            state: Mutex::new(QueueState::default()),
            capacity,
        }
    }

    pub(super) fn queued_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len
    }

    pub(super) fn enqueue(&self, membership: &Arc<RequiredWorkMembership>) {
        // Queue then member is the only two-lock acquisition order. Readers
        // release this lock before joining the registry or native actor.
        let mut queue = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut member = membership
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !member.active || !member.marked || member.queued || member.claimed {
            return;
        }
        member.queued = true;
        drop(member);
        if let Some(tail) = queue.tail.as_ref() {
            tail.state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .next = Some(Arc::clone(membership));
        } else {
            queue.head = Some(Arc::clone(membership));
        }
        queue.tail = Some(Arc::clone(membership));
        queue.len = queue
            .len
            .checked_add(1)
            .expect("funded required queue is finite");
    }

    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn pop(
        &self,
    ) -> RequiredWorkPop {
        let mut queue = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(head) = queue.head.take() else {
            return RequiredWorkPop::Empty;
        };
        queue.len -= 1;
        let mut member = head
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        queue.head = member.next.take();
        if queue.head.is_none() {
            queue.tail = None;
        }
        member.queued = false;
        if !member.active || !member.marked {
            drop(member);
            drop(queue);
            drop(head);
            return RequiredWorkPop::Stale;
        }
        member.claimed = true;
        let claim = RequiredWorkClaim {
            membership: Arc::clone(&head),
            armed: true,
        };
        drop(member);
        drop(queue);
        RequiredWorkPop::Claimed(claim)
    }
}

impl Drop for RequiredWorkQueue {
    fn drop(&mut self) {
        let (mut head, tail) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (state.head.take(), state.tail.take())
        };
        drop(tail);
        while let Some(member) = head {
            head = member
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .next
                .take();
        }
    }
}

#[cfg(feature = "test-query-execution-observer")]
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Leave one checked attempt for the real producer scheduling, so its
    /// subsequent readiness evaluation refuses the exhausted runtime counter.
    #[doc(hidden)]
    pub fn leave_one_readiness_attempt_for_test(&self) {
        self.next_output_producer_attempt
            .store(u64::MAX - 1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Read the exact issued rows and their recorded intrinsic stop values.
    #[doc(hidden)]
    pub fn required_row_stops_for_root_for_test(
        &self,
        root: worth_relational::facade::identity::EntityId,
    ) -> Vec<(
        String,
        Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>,
    )> {
        let state = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .records
            .iter()
            .filter(|(key, _)| key.source_epoch().root_entity_for_test() == root)
            .map(|(key, record)| (format!("{key:?}"), record.required_stop.clone()))
            .collect()
    }

    #[doc(hidden)]
    pub fn queued_required_work_for_test(&self) -> usize {
        let queue = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .required_work_queue
            .clone();
        queue.map_or(0, |queue| queue.queued_count())
    }

    /// Scan oracle for the Published rows and their actual remaining owners.
    #[doc(hidden)]
    pub fn published_required_rows_for_test(&self) -> Vec<(String, usize, bool, String)> {
        let state = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .records
            .iter()
            .filter_map(|(key, record)| {
                let super::super::DemandState::Output(output) = &record.state else {
                    return None;
                };
                matches!(
                    output.checkpoint,
                    Some(super::super::WorthQueryOutputCheckpoint::Published { .. })
                )
                .then(|| {
                    (
                        format!("{key:?}"),
                        record.interests,
                        record.held_successor.is_some(),
                        format!(
                            "framework={}, prepared={}, cleanup={}, obligations={}, source={}",
                            record.framework_required_count,
                            record.prepared_prerequisite_claims,
                            record.pending_cleanup_queued,
                            record.performed_obligations.len(),
                            record.performed_source.is_some(),
                        ),
                    )
                })
            })
            .collect()
    }

    /// Exact issued row keys queued for this root, retained before an advance.
    #[doc(hidden)]
    pub fn queued_required_rows_for_root_for_test(
        &self,
        root: worth_relational::facade::identity::EntityId,
    ) -> Vec<String> {
        let queue = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .required_work_queue
            .clone();
        let Some(queue) = queue else {
            return Vec::new();
        };
        let queue = queue
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut cursor = queue.head.as_ref().map(Arc::clone);
        let mut keys = Vec::new();
        while let Some(member) = cursor {
            if member.key().source_epoch().root_entity_for_test() == root {
                keys.push(format!("{:?}", member.key()));
            }
            cursor = member
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .next
                .as_ref()
                .map(Arc::clone);
        }
        keys.sort();
        keys
    }
}
