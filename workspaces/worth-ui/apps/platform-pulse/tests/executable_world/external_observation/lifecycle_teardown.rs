use std::fmt;
use std::sync::mpsc::RecvTimeoutError;
use std::thread::JoinHandle;
use std::time::Instant;

use super::lifecycle_stream::{
    LifecycleStreamItem, PlatformPulseLifecycleStream, PlatformPulseLifecycleStreamFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseLifecycleTeardownEvidence {
    discarded_envelopes: usize,
    reader_joined: bool,
}

#[derive(Debug)]
pub(crate) enum PlatformPulseLifecycleTeardownFailure {
    Deadline {
        discarded_envelopes: usize,
        reader_finished: bool,
    },
    Stream {
        failure: PlatformPulseLifecycleStreamFailure,
        discarded_envelopes: usize,
        reader_joined: bool,
    },
    ReaderPanicked {
        discarded_envelopes: usize,
    },
}

struct LifecycleFailureDrain {
    discarded_envelopes: usize,
    stream_failure: Option<PlatformPulseLifecycleStreamFailure>,
}

impl PlatformPulseLifecycleStream {
    pub(crate) fn teardown_after_failure(
        mut self,
        deadline: Instant,
    ) -> Result<PlatformPulseLifecycleTeardownEvidence, PlatformPulseLifecycleTeardownFailure> {
        if self.reader.is_none() {
            return Ok(PlatformPulseLifecycleTeardownEvidence {
                discarded_envelopes: 0,
                reader_joined: true,
            });
        }
        let drain = self.drain_after_failure(deadline)?;
        self.settle_reader_after_failure(drain)
    }

    fn drain_after_failure(
        &self,
        deadline: Instant,
    ) -> Result<LifecycleFailureDrain, PlatformPulseLifecycleTeardownFailure> {
        let mut discarded_envelopes = 0_usize;
        let mut stream_failure = None;
        loop {
            let remaining = match deadline.checked_duration_since(Instant::now()) {
                Some(remaining) => remaining,
                None => {
                    return Err(PlatformPulseLifecycleTeardownFailure::Deadline {
                        discarded_envelopes,
                        reader_finished: self.reader_is_finished(),
                    })
                }
            };
            match self.receiver.recv_timeout(remaining) {
                Ok(LifecycleStreamItem::Envelope { .. }) => {
                    discarded_envelopes = discarded_envelopes.saturating_add(1);
                }
                Ok(LifecycleStreamItem::Failure(failure)) => {
                    if stream_failure.is_none() {
                        stream_failure = Some(failure);
                    }
                }
                Ok(LifecycleStreamItem::End) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(PlatformPulseLifecycleTeardownFailure::Deadline {
                        discarded_envelopes,
                        reader_finished: self.reader_is_finished(),
                    })
                }
            }
        }
        Ok(LifecycleFailureDrain {
            discarded_envelopes,
            stream_failure,
        })
    }

    fn settle_reader_after_failure(
        &mut self,
        drain: LifecycleFailureDrain,
    ) -> Result<PlatformPulseLifecycleTeardownEvidence, PlatformPulseLifecycleTeardownFailure> {
        let LifecycleFailureDrain {
            discarded_envelopes,
            stream_failure,
        } = drain;
        let reader_joined = self
            .reader
            .take()
            .ok_or(PlatformPulseLifecycleTeardownFailure::Stream {
                failure: PlatformPulseLifecycleStreamFailure::ReaderDisconnected,
                discarded_envelopes,
                reader_joined: false,
            })?
            .join()
            .is_ok();
        if !reader_joined {
            return Err(PlatformPulseLifecycleTeardownFailure::ReaderPanicked {
                discarded_envelopes,
            });
        }
        if let Some(failure) = stream_failure {
            return Err(PlatformPulseLifecycleTeardownFailure::Stream {
                failure,
                discarded_envelopes,
                reader_joined,
            });
        }
        Ok(PlatformPulseLifecycleTeardownEvidence {
            discarded_envelopes,
            reader_joined,
        })
    }

    fn reader_is_finished(&self) -> bool {
        self.reader.as_ref().is_some_and(JoinHandle::is_finished)
    }
}

impl PlatformPulseLifecycleTeardownEvidence {
    pub(crate) fn discarded_envelopes(self) -> usize {
        self.discarded_envelopes
    }

    pub(crate) fn reader_joined(self) -> bool {
        self.reader_joined
    }
}

impl fmt::Display for PlatformPulseLifecycleTeardownFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deadline {
                discarded_envelopes,
                reader_finished,
            } => write!(
                formatter,
                "reader teardown deadline elapsed after discarding {discarded_envelopes} envelope(s); reader_finished={reader_finished}"
            ),
            Self::Stream {
                failure,
                discarded_envelopes,
                reader_joined,
            } => write!(
                formatter,
                "reader reported `{failure}` while teardown discarded {discarded_envelopes} envelope(s); reader_joined={reader_joined}"
            ),
            Self::ReaderPanicked {
                discarded_envelopes,
            } => write!(
                formatter,
                "reader panicked after teardown discarded {discarded_envelopes} envelope(s)"
            ),
        }
    }
}
