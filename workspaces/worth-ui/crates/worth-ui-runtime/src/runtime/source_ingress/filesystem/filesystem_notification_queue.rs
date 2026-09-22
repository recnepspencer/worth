use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::Arc;
use std::time::Duration;

use notify::{Event, EventHandler};

use super::filesystem_watcher_event_translation::bears_on_source_tree;

const NOTIFICATION_QUEUE_CAPACITY: usize = 256;

pub(super) struct WorthUiFilesystemNotificationQueue {
    receiver: Receiver<notify::Result<Event>>,
    resnapshot_pending: Arc<AtomicBool>,
    backend_failure_pending: Arc<AtomicBool>,
    observed_notification_count: Arc<AtomicU64>,
}

pub(super) struct WorthUiFilesystemNotificationHandler {
    sender: SyncSender<notify::Result<Event>>,
    resnapshot_pending: Arc<AtomicBool>,
    backend_failure_pending: Arc<AtomicBool>,
    observed_notification_count: Arc<AtomicU64>,
}

pub(super) fn filesystem_notification_queue() -> (
    WorthUiFilesystemNotificationQueue,
    WorthUiFilesystemNotificationHandler,
) {
    let (sender, receiver) = mpsc::sync_channel(NOTIFICATION_QUEUE_CAPACITY);
    let resnapshot_pending = Arc::new(AtomicBool::new(false));
    let backend_failure_pending = Arc::new(AtomicBool::new(false));
    let observed_notification_count = Arc::new(AtomicU64::new(0));
    (
        WorthUiFilesystemNotificationQueue {
            receiver,
            resnapshot_pending: Arc::clone(&resnapshot_pending),
            backend_failure_pending: Arc::clone(&backend_failure_pending),
            observed_notification_count: Arc::clone(&observed_notification_count),
        },
        WorthUiFilesystemNotificationHandler {
            sender,
            resnapshot_pending,
            backend_failure_pending,
            observed_notification_count,
        },
    )
}

impl WorthUiFilesystemNotificationQueue {
    pub(super) fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<notify::Result<Event>, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    pub(super) fn take_resnapshot_signal(&self) -> bool {
        self.resnapshot_pending.swap(false, Ordering::AcqRel)
    }

    pub(super) fn take_backend_failure(&self) -> bool {
        self.backend_failure_pending.swap(false, Ordering::AcqRel)
    }

    pub(super) fn observed_notification_count(&self) -> u64 {
        self.observed_notification_count.load(Ordering::Acquire)
    }
}

impl EventHandler for WorthUiFilesystemNotificationHandler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        if let Ok(event) = &event {
            if !bears_on_source_tree(&event.kind) {
                return;
            }
        }
        let _ = self.observed_notification_count.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |count| Some(count.saturating_add(1)),
        );
        match self.sender.try_send(event) {
            Ok(()) => {}
            Err(TrySendError::Full(Ok(_))) => {
                self.resnapshot_pending.store(true, Ordering::Release);
            }
            Err(TrySendError::Full(Err(_))) => {
                self.backend_failure_pending.store(true, Ordering::Release);
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use notify::event::{AccessKind, AccessMode};
    use notify::{Event, EventHandler, EventKind};

    use super::{filesystem_notification_queue, NOTIFICATION_QUEUE_CAPACITY};

    #[test]
    fn access_notifications_are_neither_counted_nor_queued() {
        let (queue, mut handler) = filesystem_notification_queue();
        for _ in 0..3 {
            handler.handle_event(Ok(Event::new(EventKind::Access(AccessKind::Open(
                AccessMode::Any,
            )))));
        }

        assert_eq!(queue.observed_notification_count(), 0);
        assert!(queue.recv_timeout(Duration::ZERO).is_err());

        handler.handle_event(Ok(Event::new(EventKind::Any)));
        assert_eq!(queue.observed_notification_count(), 1);
        assert!(queue.recv_timeout(Duration::ZERO).is_ok());
    }

    #[test]
    fn overflow_remains_bounded_and_preserves_a_resnapshot_trigger() {
        let (queue, mut handler) = filesystem_notification_queue();
        for _ in 0..=NOTIFICATION_QUEUE_CAPACITY {
            handler.handle_event(Ok(Event::new(EventKind::Any)));
        }

        assert_eq!(
            queue.observed_notification_count(),
            (NOTIFICATION_QUEUE_CAPACITY + 1) as u64
        );
        let mut queued = 0;
        while queue.recv_timeout(Duration::ZERO).is_ok() {
            queued += 1;
        }
        assert_eq!(queued, NOTIFICATION_QUEUE_CAPACITY);
        assert!(queue.take_resnapshot_signal());
        assert!(!queue.take_resnapshot_signal());
    }
}
