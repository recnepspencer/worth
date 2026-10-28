use std::sync::Mutex;

use worth_server::WorthServerProductSessionClock;

#[derive(Debug)]
pub struct ManualProductSessionClock {
    now_epoch_millis: Mutex<u64>,
}

impl ManualProductSessionClock {
    pub fn new(now_epoch_millis: u64) -> Self {
        Self {
            now_epoch_millis: Mutex::new(now_epoch_millis),
        }
    }

    pub fn advance_millis(&self, delta_millis: u64) {
        let mut now_epoch_millis = self
            .now_epoch_millis
            .lock()
            .expect("manual product session clock lock");
        *now_epoch_millis = now_epoch_millis.saturating_add(delta_millis);
    }
}

impl WorthServerProductSessionClock for ManualProductSessionClock {
    fn current_time_millis(&self) -> u64 {
        *self
            .now_epoch_millis
            .lock()
            .expect("manual product session clock lock")
    }
}
