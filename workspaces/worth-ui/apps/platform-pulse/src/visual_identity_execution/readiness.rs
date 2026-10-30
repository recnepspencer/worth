use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Instant;

enum PlatformPulseVisualReadinessCommand {
    Schedule(Instant),
    Stop,
}

pub(super) struct PlatformPulseVisualReadiness {
    commands: Sender<PlatformPulseVisualReadinessCommand>,
    worker: Option<JoinHandle<()>>,
}

impl PlatformPulseVisualReadiness {
    #[expect(
        clippy::disallowed_methods,
        reason = "event-loop-watcher thread-builder=1: UI readiness, source, theme or event-loop observation runs independently of computation."
    )]
    pub(super) fn install(
        signal: worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal,
    ) -> Self {
        let (commands, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("worth-ui-platform-pulse-visual-readiness".to_owned())
            .spawn(move || {
                let mut deadline: Option<Instant> = None;
                loop {
                    let command = match deadline {
                        Some(at) => match at.checked_duration_since(Instant::now()) {
                            Some(wait) => match receiver.recv_timeout(wait) {
                                Ok(command) => command,
                                Err(RecvTimeoutError::Timeout) => {
                                    signal.signal();
                                    deadline = None;
                                    continue;
                                }
                                Err(RecvTimeoutError::Disconnected) => return,
                            },
                            None => {
                                signal.signal();
                                deadline = None;
                                continue;
                            }
                        },
                        None => match receiver.recv() {
                            Ok(command) => command,
                            Err(_) => return,
                        },
                    };
                    match command {
                        PlatformPulseVisualReadinessCommand::Schedule(at) => deadline = Some(at),
                        PlatformPulseVisualReadinessCommand::Stop => return,
                    }
                }
            })
            .expect("visual readiness worker should start");
        Self {
            commands,
            worker: Some(worker),
        }
    }

    pub(super) fn schedule(&self, deadline: Instant) {
        let _ = self
            .commands
            .send(PlatformPulseVisualReadinessCommand::Schedule(deadline));
    }
}

impl Drop for PlatformPulseVisualReadiness {
    fn drop(&mut self) {
        let _ = self
            .commands
            .send(PlatformPulseVisualReadinessCommand::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
