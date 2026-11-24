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
