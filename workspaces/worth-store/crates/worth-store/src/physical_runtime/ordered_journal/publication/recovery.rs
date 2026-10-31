use super::super::OrderedJournal;
use super::{operation::*, outcome::*};
use std::sync::Arc;
#[must_use]
pub struct OrderedJournalPublicationRecovery {
    pub(super) shared: Arc<JournalPublicationShared>,
}
pub enum OrderedJournalPublicationResume {
    Committed(OrderedJournalCommitted),
    Ready {
        operation: OrderedJournalPublicationOperation,
        recovery: OrderedJournalPublicationRecovery,
    },
    Busy(OrderedJournalPublicationRecovery),
    Stopped {
        operation: OrderedJournalStoppedPublication,
        recovery: OrderedJournalPublicationRecovery,
    },
    Terminal(OrderedJournalPublicationObservation),
}
#[must_use]
pub struct OrderedJournalStoppedPublication {
    pub(super) shared: Arc<JournalPublicationShared>,
    pub(super) custody: Option<Resource>,
}
pub enum OrderedJournalStoppedPoll {
    Pending(OrderedJournalStoppedPublication),
    Settled(OrderedJournalPublicationObservation),
    Failed(OrderedJournalPublicationFailureCustody),
}
impl OrderedJournalPublicationRecovery {
    pub fn observe(&self) -> OrderedJournalPublicationObservation {
        self.shared.observe()
    }
    pub fn resume(self) -> OrderedJournalPublicationResume {
        let observed = self.observe();
        if observed.terminal().is_some() {
            if let Some(committed) = self.shared.readmit_committed() {
                return OrderedJournalPublicationResume::Committed(committed);
            }
            return OrderedJournalPublicationResume::Terminal(observed);
        }
        let mut s = self.shared.state();
        if s.endpoint_live {
            drop(s);
            return OrderedJournalPublicationResume::Busy(self);
        }
        if s.stopped {
            let custody = s.returned.take();
            s.endpoint_live = true;
            drop(s);
            let operation = OrderedJournalStoppedPublication {
                shared: self.shared.clone(),
                custody,
            };
            return OrderedJournalPublicationResume::Stopped {
                operation,
                recovery: self,
            };
        }
        let Some(custody) = s.returned.take() else {
            drop(s);
            return OrderedJournalPublicationResume::Busy(self);
        };
        s.endpoint_live = true;
        drop(s);
        let operation = OrderedJournalPublicationOperation {
            shared: self.shared.clone(),
            custody: Some(custody),
        };
        OrderedJournalPublicationResume::Ready {
            operation,
            recovery: self,
        }
    }
    /// Irreversibly fences advancement without lending a second endpoint.
    pub fn stop(self) -> Self {
        self.shared.stop();
        self
    }
}
impl OrderedJournalStoppedPublication {
    pub fn observe(&self) -> OrderedJournalPublicationObservation {
        self.shared.observe()
    }
    pub fn request_cancellation(&self) {
        let attempt = self.shared.state().attempt.upgrade();
        if let Some(a) = attempt {
            crate::physical_runtime::PhysicalMutationHandle::new(&a).request_cancellation();
        }
    }
    pub fn settle(mut self, journal: &OrderedJournal<'_>) -> OrderedJournalStoppedPoll {
        if let Err(cause) = self.shared.require_receiver(journal) {
            return OrderedJournalStoppedPoll::Failed(OrderedJournalPublicationFailureCustody {
                stopped: self,
                cause,
            });
        }
        match self.shared.poll_terminal() {
            Ok(false) => OrderedJournalStoppedPoll::Pending(self),
            Ok(true) => {
                if let Some(r) = self.custody.take() {
                    let _ = r.terminate(Disposition::Stopped);
                }
                OrderedJournalStoppedPoll::Settled(self.observe())
            }
            Err(cause) => {
                OrderedJournalStoppedPoll::Failed(OrderedJournalPublicationFailureCustody {
                    stopped: self,
                    cause,
                })
            }
        }
    }
}
impl Drop for OrderedJournalPublicationRecovery {
    fn drop(&mut self) {
        let abandoned = {
            let mut s = self.shared.state();
            s.recovery_alive = false;
            s.returned.is_some()
        };
        if abandoned {
            self.shared.stop();
        }
    }
}
impl Drop for OrderedJournalStoppedPublication {
    fn drop(&mut self) {
        self.shared.state().endpoint_live = false;
        if let Some(r) = self.custody.take() {
            let _ = r.terminate(Disposition::Stopped);
        }
    }
}
impl std::fmt::Debug for OrderedJournalPublicationRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OrderedJournalPublicationRecovery")
            .field(&self.observe())
            .finish()
    }
}
impl std::fmt::Debug for OrderedJournalStoppedPublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OrderedJournalStoppedPublication")
            .field(&self.observe())
            .finish()
    }
}
