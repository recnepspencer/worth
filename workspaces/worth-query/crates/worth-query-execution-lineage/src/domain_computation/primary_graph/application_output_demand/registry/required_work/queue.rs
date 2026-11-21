//! One queue of actual required members; selected work remains a hint.

use super::*;

#[derive(Default)]
pub struct QueueState {
    pub head: Option<Arc<RequiredWorkMembership>>,
    pub tail: Option<Arc<RequiredWorkMembership>>,
    pub len: usize,
}

pub struct RequiredWorkQueue {
    pub state: Mutex<QueueState>,
}

pub enum RequiredWorkPop {
    Empty,
    Stale,
    Claimed(RequiredWorkClaim),
}

impl RequiredWorkQueue {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(QueueState::default()),
        }
    }

    /// Inactive native observers may outlive the last queued demand.
    pub fn is_quiescent(
        &self,
    ) -> bool {
        self.state
            .lock()
            .is_ok_and(|state| state.head.is_none() && state.tail.is_none() && state.len == 0)
    }

    pub fn queued_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len
    }

    pub fn enqueue(&self, membership: &Arc<RequiredWorkMembership>) {
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

    pub fn pop(&self) -> RequiredWorkPop {
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

impl Default for RequiredWorkQueue {
    fn default() -> Self {
        Self::new()
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

