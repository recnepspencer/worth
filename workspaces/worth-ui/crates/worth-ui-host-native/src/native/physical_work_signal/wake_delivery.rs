use super::routing::UiNativePhysicalSignalWork;

#[derive(Clone, Copy)]
struct UiNativePhysicalSignalReadyWork {
    work: UiNativePhysicalSignalWork,
}

pub(crate) struct UiNativePhysicalSignalWakeDelivery {
    ready: Vec<UiNativePhysicalSignalReadyWork>,
}

impl UiNativePhysicalSignalWakeDelivery {
    pub(crate) fn new() -> Self {
        Self { ready: Vec::new() }
    }

    pub(crate) fn request(&mut self, work: UiNativePhysicalSignalWork) {
        if !self.ready.iter().any(|ready| ready.work == work) {
            self.ready.push(UiNativePhysicalSignalReadyWork { work });
        }
    }

    pub(crate) fn take(&mut self, work: UiNativePhysicalSignalWork) -> bool {
        let Some(index) = self
            .ready
            .iter()
            .position(|candidate| candidate.work == work)
        else {
            return false;
        };
        self.ready.remove(index);
        true
    }

    pub(crate) fn remove(&mut self, work: UiNativePhysicalSignalWork) {
        if let Some(index) = self
            .ready
            .iter()
            .position(|candidate| candidate.work == work)
        {
            self.ready.remove(index);
        }
    }

    pub(crate) fn pending(&self) -> usize {
        self.ready.len()
    }

    pub(crate) fn next(&self) -> Option<UiNativePhysicalSignalWork> {
        self.ready.first().map(|ready| ready.work)
    }

    pub(crate) fn clear(&mut self) {
        self.ready.clear();
    }
}
