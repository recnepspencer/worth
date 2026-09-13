//! A real hostile metadata writer, not an observer hook or edited report.
use std::{
    fs::{FileTimes, OpenOptions},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

pub(super) struct SourceChanges {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<std::io::Result<u64>>>,
}

impl SourceChanges {
    pub(super) fn start(path: &Path) -> Self {
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // WRITE_ATTRIBUTES does not request WRITE_DATA. Windows can deny
            // data writers while metadata changes still invalidate the snapshot.
            options.access_mode(0x100).share_mode(0x1 | 0x2 | 0x4);
        }
        let file = options.open(path).unwrap();
        let original = std::fs::metadata(path).unwrap().modified().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let (ready, arrived) = mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(90);
            let mut writes = 0_u64;
            let result = (|| {
                while !worker_stop.load(Ordering::Acquire) && Instant::now() < deadline {
                    writes += 1;
                    file.set_times(
                        FileTimes::new()
                            .set_modified(original + Duration::from_nanos(writes * 100)),
                    )?;
                    if writes == 1 {
                        let _ = ready.send(());
                    }
                }
                Ok(writes)
            })();
            file.set_times(FileTimes::new().set_modified(original))?;
            result
        });
        let changes = Self {
            stop,
            worker: Some(worker),
        };
        arrived
            .recv_timeout(Duration::from_secs(5))
            .expect("real metadata writer starts before observer");
        changes
    }

    pub(super) fn finish(mut self) -> u64 {
        self.stop.store(true, Ordering::Release);
        self.worker.take().unwrap().join().unwrap().unwrap()
    }
}

impl Drop for SourceChanges {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
