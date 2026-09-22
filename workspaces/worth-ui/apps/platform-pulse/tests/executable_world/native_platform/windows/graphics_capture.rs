//! Process-bound desktop-duplication strip stream with OS presentation timestamps.
use super::{
    NativePlatformContract, NativePlatformFailure, WindowsCaptureExposure, WindowsNativePlatform,
    WindowsProcessBoundNativeClientArea,
};
use crate::external_observation::NativeTimedClientPixelCapture;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread::JoinHandle;
use std::time::Duration;
mod adapter_provenance;
mod clock;
mod crop;
#[cfg(test)]
mod tests;
mod worker;
pub(super) use clock::qpc_100ns;
use crop::CaptureCrop;
const BUFFERED_FRAMES: usize = 64;
const MAX_FRAME_BYTES: usize = 1_048_576;

pub(crate) struct WindowsCaptureStream<'bound> {
    bound: &'bound WindowsProcessBoundNativeClientArea,
    crop: CaptureCrop,
    receiver: mpsc::Receiver<NativeTimedClientPixelCapture>,
    failure: Arc<Mutex<Option<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), String>>>,
}

impl WindowsNativePlatform {
    pub(crate) fn start_capture_stream<'bound>(
        &self,
        exposure: &'bound WindowsCaptureExposure<'bound>,
        strip: [u32; 4],
    ) -> Result<WindowsCaptureStream<'bound>, NativePlatformFailure> {
        let bound = exposure.bound;
        self.observe_bound_client_area(bound)?;
        let crop = CaptureCrop::qualify(bound, strip)?;
        crop.require_current(bound)?;
        let (sender, receiver) = mpsc::sync_channel(BUFFERED_FRAMES);
        let failure = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_failure = failure.clone();
        let worker_stop = stop.clone();
        let process_id = bound.observation.process_id();
        let worker = std::thread::spawn(move || {
            let result = worker::capture(crop, process_id, sender, &worker_stop);
            if let Err(error) = &result {
                *worker_failure.lock().expect("capture failure lock") = Some(error.clone());
            }
            result
        });
        Ok(WindowsCaptureStream {
            bound,
            crop,
            receiver,
            failure,
            stop,
            worker: Some(worker),
        })
    }
}

impl WindowsCaptureStream<'_> {
    pub(crate) fn next(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<NativeTimedClientPixelCapture>, NativePlatformFailure> {
        self.check()?;
        let result = match self.receiver.recv_timeout(timeout) {
            Ok(frame) => Ok(Some(frame)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(capture_failure("capture stopped")),
        };
        self.check()?;
        result
    }

    fn check(&self) -> Result<(), NativePlatformFailure> {
        if let Some(error) = self.failure.lock().expect("capture failure lock").as_ref() {
            return Err(capture_failure(error));
        }
        WindowsNativePlatform::default().observe_bound_client_area(self.bound)?;
        self.crop.require_current(self.bound)?;
        if self.worker.as_ref().is_none_or(JoinHandle::is_finished) {
            return Err(capture_failure(
                "capture worker ended before trace completion",
            ));
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<(), NativePlatformFailure> {
        let posture = self.check();
        self.stop.store(true, Ordering::Release);
        let joined = self
            .worker
            .take()
            .expect("live capture")
            .join()
            .map_err(|_| capture_failure("capture worker panicked"))?
            .map_err(capture_failure);
        posture?;
        joined
    }
}

impl Drop for WindowsCaptureStream<'_> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            if !matches!(worker.join(), Ok(Ok(()))) {
                eprintln!("desktop duplication cleanup joined a failed capture worker");
            }
        }
    }
}

fn capture_failure(error: impl std::fmt::Display) -> NativePlatformFailure {
    NativePlatformFailure::ClientCapture(format!("timestamped desktop duplication: {error}"))
}
