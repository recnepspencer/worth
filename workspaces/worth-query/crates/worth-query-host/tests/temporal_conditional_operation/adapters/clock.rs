use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use worth_query_host::facade::domain;

pub struct CourtroomClock;

impl domain::WorthQueryNamedClock for CourtroomClock {
    const PORTABLE_IDENTITY: &'static str = "worth.query.host.courtroom.clock";
}

#[derive(Clone)]
pub struct ClockController {
    state: Arc<Mutex<ClockSourceState>>,
}

impl ClockController {
    pub fn push(&self, sequence: u64, now: u64) {
        self.state
            .lock()
            .unwrap()
            .scripted
            .push_back((sequence, now));
    }
}

pub struct ClockSource {
    controller: ClockController,
}

struct ClockSourceState {
    sequence: u64,
    now: u64,
    scripted: VecDeque<(u64, u64)>,
}

impl ClockSource {
    pub fn due() -> (Self, ClockController) {
        let controller = ClockController {
            state: Arc::new(Mutex::new(ClockSourceState {
                sequence: 0,
                now: 10,
                scripted: VecDeque::new(),
            })),
        };
        (
            Self {
                controller: controller.clone(),
            },
            controller,
        )
    }
}

impl domain::WorthQueryNamedClockSource<CourtroomClock> for ClockSource {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.courtroom.clock-source";

    fn source_identity(&self) -> domain::WorthQueryClockSourceIdentity {
        domain::WorthQueryClockSourceIdentity::declare("courtroom-source").unwrap()
    }

    fn timeline_identity(&self) -> domain::WorthQueryClockTimelineIdentity {
        domain::WorthQueryClockTimelineIdentity::declare("courtroom-timeline").unwrap()
    }

    fn observe(
        &self,
    ) -> Result<
        domain::WorthQueryNamedClockReading<CourtroomClock>,
        domain::WorthQueryNamedClockFailure,
    > {
        let mut state = self.controller.state.lock().unwrap();
        if let Some((sequence, now)) = state.scripted.pop_front() {
            state.sequence = sequence;
            state.now = now;
        } else {
            state.sequence += 1;
        }
        Ok(domain::WorthQueryNamedClockReading::new(
            state.sequence,
            domain::WorthQueryClockCoordinate::from_nanoseconds(state.now),
        ))
    }
}
