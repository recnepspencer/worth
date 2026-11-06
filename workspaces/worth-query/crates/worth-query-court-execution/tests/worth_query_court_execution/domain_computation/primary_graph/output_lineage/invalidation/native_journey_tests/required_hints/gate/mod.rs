//! Rendezvous points for the two-branch preparation journey.

use std::sync::Condvar;
use std::sync::Mutex;
use std::time::Duration;

use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Default)]
pub struct Latch(Mutex<bool>, Condvar);

impl Latch {
    pub fn open(&self) {
        *self.0.lock().unwrap() = true;
        self.1.notify_all();
    }

    pub fn wait(&self) -> Result<(), CompanionPreflightStop> {
        let open = self.0.lock().unwrap();
        let (open, _) = self
            .1
            .wait_timeout_while(open, Duration::from_secs(30), |open| !*open)
            .unwrap();
        if *open {
            Ok(())
        } else {
            Err(CompanionPreflightStop::TopologyPending)
        }
    }
}

#[derive(Default)]
pub struct TwoPrepareGate {
    pub count: Mutex<usize>,
    pub ready: Condvar,
    pub main_linked: Latch,
    // The patch-position reservation is one runtime-wide nonblocking slot, so
    // the fork cutover waits for main's publication to return rather than
    // racing it into a legal contention deferral.
    pub main_published: Latch,
}

impl TwoPrepareGate {
    pub fn meet(&self) -> Result<(), CompanionPreflightStop> {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        if *count == 2 {
            self.ready.notify_all();
            return Ok(());
        }
        let (count, timed) = self
            .ready
            .wait_timeout_while(count, Duration::from_secs(30), |count| *count < 2)
            .unwrap();
        if *count < 2 && timed.timed_out() {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(())
    }
}
