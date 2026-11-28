//! Fixed-delay retry policy for a request-response resource.
use core::{num::NonZeroU32, time::Duration};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationResourceFixedDelayRetry {
    pub maximum_attempts: NonZeroU32,
    pub delay: Duration,
}
