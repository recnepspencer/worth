//! Ingress relevance for every Platform Pulse file watcher.
//!
//! A watcher must never observe its own reads as change. notify's inotify backend
//! registers `IN_OPEN`, so each `std::fs::read` of a watched file emits
//! `EventKind::Access(Open)`; a watcher that queues raw notifications re-reads on
//! its own read and loops until its channel overflows or the worker spins.
//! Windows' `ReadDirectoryChangesW` emits no access events, which is why the loop
//! was invisible on the first qualified platform. Access events are therefore
//! dropped at ingress on every platform, and only notifications naming the watched
//! file pass. Error notifications always pass so the owning worker can surface
//! them as its typed denial.

use notify::EventKind;

/// Whether a notification may bear on the content of `file_name` inside the watched
/// root. `Access(_)` never does: opens and reads are observation, and `Close(Write)`
/// is redundant with the `Modify` the same write already emitted.
pub fn bears_on_watched_file(event: &notify::Result<notify::Event>, file_name: &str) -> bool {
    match event {
        Ok(event) => {
            !matches!(event.kind, EventKind::Access(_))
                && event
                    .paths
                    .iter()
                    .any(|path| path.file_name().is_some_and(|name| name == file_name))
        }
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use notify::event::{AccessKind, AccessMode, CreateKind, DataChange, ModifyKind, RenameMode};
    use notify::{Event, EventKind};

    use super::bears_on_watched_file;

    const WATCHED: &str = "platform-pulse-intent.json";

    fn event(kind: EventKind, names: &[&str]) -> notify::Result<Event> {
        let mut event = Event::new(kind);
        for name in names {
            event = event.add_path(PathBuf::from("/watched/root").join(name));
        }
        Ok(event)
    }

    #[test]
    fn own_reads_of_the_watched_file_are_not_change() {
        let open = event(
            EventKind::Access(AccessKind::Open(AccessMode::Read)),
            &[WATCHED],
        );
        let read = event(EventKind::Access(AccessKind::Read), &[WATCHED]);
        let close_write = event(
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
            &[WATCHED],
        );
        assert!(!bears_on_watched_file(&open, WATCHED));
        assert!(!bears_on_watched_file(&read, WATCHED));
        assert!(!bears_on_watched_file(&close_write, WATCHED));
    }

    #[test]
    fn writes_creates_and_renames_into_the_watched_file_are_change() {
        let modify = event(
            EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            &[WATCHED],
        );
        let create = event(EventKind::Create(CreateKind::File), &[WATCHED]);
        let rename_into_place = event(
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            &["platform-pulse-intent.json.tmp", WATCHED],
        );
        assert!(bears_on_watched_file(&modify, WATCHED));
        assert!(bears_on_watched_file(&create, WATCHED));
        assert!(bears_on_watched_file(&rename_into_place, WATCHED));
    }

    #[test]
    fn traffic_on_sibling_files_in_the_root_is_not_change() {
        let sibling = event(
            EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            &["platform-pulse-value.json"],
        );
        let temp_only = event(
            EventKind::Create(CreateKind::File),
            &["platform-pulse-intent.json.tmp"],
        );
        let pathless = event(EventKind::Modify(ModifyKind::Data(DataChange::Any)), &[]);
        assert!(!bears_on_watched_file(&sibling, WATCHED));
        assert!(!bears_on_watched_file(&temp_only, WATCHED));
        assert!(!bears_on_watched_file(&pathless, WATCHED));
    }

    #[test]
    fn watcher_errors_always_reach_the_owner() {
        let error: notify::Result<Event> = Err(notify::Error::generic("queue overflow"));
        assert!(bears_on_watched_file(&error, WATCHED));
    }
}
