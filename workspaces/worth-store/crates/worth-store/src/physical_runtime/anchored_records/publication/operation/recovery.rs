use super::*;

/// Strong custody held by the caller outside its disposable execution result.
#[must_use]
pub struct AnchoredRecordPublicationRecovery {
    pub(super) shared: Arc<PublicationShared>,
}
pub enum AnchoredRecordPublicationResume {
    Ready(AnchoredRecordPublicationOperation),
    Busy(AnchoredRecordPublicationRecovery),
    Stopped(AnchoredRecordStoppedPublication),
    Terminal(AnchoredRecordPublicationObservation),
}
#[must_use]
pub struct AnchoredRecordStoppedPublication {
    pub(super) shared: Arc<PublicationShared>,
    pub(super) custody: Option<PublicationResource>,
}
pub enum AnchoredRecordStoppedPoll {
    Pending(AnchoredRecordStoppedPublication),
    Settled(AnchoredRecordPublicationObservation),
    Failed(AnchoredRecordPublicationFailureCustody),
}
impl AnchoredRecordPublicationRecovery {
    pub fn observe(&self) -> AnchoredRecordPublicationObservation {
        self.shared.observe()
    }
    pub fn resume(self) -> AnchoredRecordPublicationResume {
        let mut state = self.shared.state();
        if state.terminal.is_some() {
            drop(state);
            return AnchoredRecordPublicationResume::Terminal(self.observe());
        }
        if state.stopped {
            let custody = state.returned_custody.take();
            drop(state);
            return AnchoredRecordPublicationResume::Stopped(AnchoredRecordStoppedPublication {
                shared: self.shared.clone(),
                custody,
            });
        }
        let Some(custody) = state.returned_custody.take() else {
            drop(state);
            return AnchoredRecordPublicationResume::Busy(self);
        };
        let stopped = state.stopped;
        drop(state);
        if stopped {
            AnchoredRecordPublicationResume::Stopped(AnchoredRecordStoppedPublication {
                shared: self.shared.clone(),
                custody: Some(custody),
            })
        } else {
            AnchoredRecordPublicationResume::Ready(AnchoredRecordPublicationOperation {
                shared: self.shared.clone(),
                custody: Some(custody),
            })
        }
    }
    pub fn stop(self) -> AnchoredRecordStoppedPublication {
        self.shared.stop();
        let custody = self.shared.state().returned_custody.take();
        AnchoredRecordStoppedPublication {
            shared: self.shared.clone(),
            custody,
        }
    }
}
impl AnchoredRecordStoppedPublication {
    pub fn observe(&self) -> AnchoredRecordPublicationObservation {
        self.shared.observe()
    }
    /// Explicit cancellation may synchronize with the current physical cutover.
    /// Observer loss or stop-advancement alone never withdraws effect authority.
    pub fn request_cancellation(&self) {
        let accepted = self.shared.state().accepted.clone();
        if let Some(step) = accepted {
            step.cancel();
        }
    }
    pub fn settle(mut self, records: &AnchoredRecords<'_>) -> AnchoredRecordStoppedPoll {
        if let Err(cause) = self.shared.require_receiver(records) {
            return AnchoredRecordStoppedPoll::Failed(AnchoredRecordPublicationFailureCustody {
                stopped: self,
                cause,
            });
        }
        match progression::settle_accepted(&self.shared, records) {
            Ok(false) => AnchoredRecordStoppedPoll::Pending(self),
            Ok(true) => {
                {
                    let mut state = self.shared.state();
                    state.plan = None;
                    state.live = false;
                    state.interruption = None;
                }
                if let Some(custody) = self.custody.take() {
                    let _receipt = custody.terminate(PublicationDisposition::Stopped);
                }
                AnchoredRecordStoppedPoll::Settled(self.observe())
            }
            Err(cause) => {
                AnchoredRecordStoppedPoll::Failed(AnchoredRecordPublicationFailureCustody {
                    stopped: self,
                    cause,
                })
            }
        }
    }
}
impl Drop for AnchoredRecordStoppedPublication {
    fn drop(&mut self) {
        if let Some(custody) = self.custody.take() {
            let _receipt = custody.terminate(PublicationDisposition::Stopped);
        }
    }
}

impl Drop for AnchoredRecordPublicationRecovery {
    fn drop(&mut self) {
        let abandoned = {
            let mut state = self.shared.state();
            state.recovery_alive = false;
            state.returned_custody.is_some()
        };
        if abandoned {
            self.shared.stop();
        }
    }
}
