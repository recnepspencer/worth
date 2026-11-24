use std::sync::{
    mpsc::{Receiver, SyncSender},
    Mutex,
};

#[derive(Debug)]
pub(super) struct CapturePause {
    pub(super) entered: SyncSender<()>,
    pub(super) resume: Mutex<Receiver<()>>,
}
impl CapturePause {
    pub(super) fn wait(&self) {
        let _ = self.entered.send(());
        let _ = self.resume.lock().unwrap_or_else(|p| p.into_inner()).recv();
    }
}

impl super::owner::RelationalBranchRetentionOwnerInner {
    pub(super) fn pause_component_capture(&self) {
        let Some(slot) = self.component_capture_pause.get() else {
            return;
        };
        let pause = slot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        if let Some(pause) = pause {
            pause.wait();
        }
    }
}
