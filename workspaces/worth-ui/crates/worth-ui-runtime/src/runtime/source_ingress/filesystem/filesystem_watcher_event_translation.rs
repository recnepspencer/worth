use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};

use crate::runtime::source_ingress::WorthUiWatcherEvent;

/// Whether a backend notification can carry information about the source tree.
///
/// `Access` notifications report opens and closes, not content or structure. Under
/// inotify the backend registers `IN_OPEN`, so every file the watcher itself reads while
/// freezing a snapshot arrives back as one of these; treating them as unrest would keep
/// the tree unsettled for as long as the watcher keeps looking at it. They are discarded
/// at ingress, before they are counted or queued, and `translate_filesystem_event`
/// agrees by translating them to nothing.
pub(super) fn bears_on_source_tree(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_))
}

pub(super) fn translate_filesystem_event(
    event: Event,
    provider_id: &str,
) -> Vec<WorthUiWatcherEvent> {
    match event.kind {
        EventKind::Access(_) => Vec::new(),
        EventKind::Remove(_) => event
            .paths
            .into_iter()
            .map(WorthUiWatcherEvent::deleted)
            .collect(),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if event.paths.len() >= 2 => {
            vec![WorthUiWatcherEvent::atomic_rename(
                event.paths[0].clone(),
                event.paths[event.paths.len() - 1].clone(),
            )]
        }
        EventKind::Create(_) | EventKind::Modify(_) => event
            .paths
            .into_iter()
            .map(WorthUiWatcherEvent::modified)
            .collect(),
        EventKind::Any | EventKind::Other => {
            vec![WorthUiWatcherEvent::provider_revision(provider_id)]
        }
    }
}

#[cfg(test)]
mod tests {
    use notify::event::{AccessKind, AccessMode, ModifyKind};
    use notify::{Event, EventKind};

    use super::{bears_on_source_tree, translate_filesystem_event};

    #[test]
    fn access_notifications_bear_nothing_and_translate_to_nothing() {
        for kind in [
            EventKind::Access(AccessKind::Open(AccessMode::Any)),
            EventKind::Access(AccessKind::Close(AccessMode::Read)),
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
        ] {
            assert!(!bears_on_source_tree(&kind), "{kind:?}");
            let event = Event::new(kind).add_path("app/main.wui".into());
            assert!(translate_filesystem_event(event, "provider").is_empty());
        }
    }

    #[test]
    fn content_and_structure_notifications_bear_on_the_source_tree() {
        for kind in [
            EventKind::Create(notify::event::CreateKind::File),
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            EventKind::Remove(notify::event::RemoveKind::File),
            EventKind::Any,
            EventKind::Other,
        ] {
            assert!(bears_on_source_tree(&kind), "{kind:?}");
            let event = Event::new(kind).add_path("app/main.wui".into());
            assert!(!translate_filesystem_event(event, "provider").is_empty());
        }
    }
}
