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
    Selected(SelectedRequiredWork),
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
        if !member.active || !member.marked || member.queued {
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
        let kind = if let Some(hint) = member.native_hints.as_ref() {
            SelectedRequiredWorkKind::Native {
                observer: hint.observer.clone(),
                settlement: Arc::clone(&hint.settlement),
                branch: Arc::clone(&hint.branch),
            }
        } else if let Some(settlement) = member.local_settlement.as_ref() {
            SelectedRequiredWorkKind::Local {
                settlement: Arc::clone(settlement),
            }
        } else if member.discontinuity_pending {
            SelectedRequiredWorkKind::Discontinuity
        } else {
            assert!(
                member.unresolved_initial,
                "queued required work has one cause"
            );
            SelectedRequiredWorkKind::UnresolvedInitial
        };
        let selected = SelectedRequiredWork {
            membership: Arc::clone(&head),
            version: member.version,
            kind,
            armed: true,
        };
        drop(member);
        drop(queue);
        RequiredWorkPop::Selected(selected)
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
