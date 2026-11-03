use std::time::Instant;

trait WorthQueryAuthenticationTimeSource: Send + Sync {
    fn current_time(&self) -> Instant;
}

struct WorthQuerySystemAuthenticationTimeSource;

impl WorthQueryAuthenticationTimeSource for WorthQuerySystemAuthenticationTimeSource {
    fn current_time(&self) -> Instant {
        Instant::now()
    }
}

pub(super) struct WorthQueryAuthenticationClock {
    source: Box<dyn WorthQueryAuthenticationTimeSource>,
}

impl WorthQueryAuthenticationClock {
    pub(super) fn system() -> Self {
        Self {
            source: Box::new(WorthQuerySystemAuthenticationTimeSource),
        }
    }

    pub(super) fn is_expired(&self, valid_until: Instant) -> bool {
        self.source.current_time() >= valid_until
    }

    pub(super) fn fixed(now: Instant) -> Self {
        Self {
            source: Box::new(WorthQueryFixedAuthenticationTimeSource { now }),
        }
    }
}

struct WorthQueryFixedAuthenticationTimeSource {
    now: Instant,
}

impl WorthQueryAuthenticationTimeSource for WorthQueryFixedAuthenticationTimeSource {
    fn current_time(&self) -> Instant {
        self.now
    }
}
