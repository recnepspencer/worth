//! Fixture-owned reader-free compute window after genuine native enrollment.
use std::sync::{mpsc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub(in super::super) struct EnrollmentWindow {
    armed: Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>,
}
pub(in super::super) struct Computing {
    reached: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
}
impl EnrollmentWindow {
    pub(in super::super) fn arm(&self) -> Computing {
        let (reached, receipt) = mpsc::channel();
        let (release, resume) = mpsc::channel();
        let mut armed = self.armed.lock().unwrap();
        assert!(
            armed.is_none(),
            "one reader-free compute window per fixture receiver"
        );
        *armed = Some((reached, resume));
        Computing {
            reached: receipt,
            release,
        }
    }
    pub(in super::super) fn before_compute(&self) {
        let armed = self.armed.lock().unwrap().take();
        if let Some((reached, resume)) = armed {
            reached
                .send(())
                .expect("original fixture window remains owned");
            resume
                .recv_timeout(Duration::from_secs(30))
                .expect("original reader-free compute must be released");
        }
    }
}
impl Computing {
    pub(in super::super) fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.reached.recv_timeout(timeout).is_ok()
    }
    pub(in super::super) fn release(&self) {
        let _ = self.release.send(());
    }
}
impl Drop for Computing {
    fn drop(&mut self) {
        self.release();
    }
}
