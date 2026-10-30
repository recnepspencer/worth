//! Shared pending-WAL child roles, mounted by each consuming court.
use super::*;

pub fn dispatch_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER_ENV).expect("child marker"));
    if let Some(first) = role.to_str().and_then(first::World::of_role) {
        return first::child(&marker, first);
    }
    match role.to_str() {
        Some("second") => crate::production_entry::pending_wal_world::second::child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("second source root")),
            &marker,
        ),
        Some("distinct") => crate::production_entry::pending_wal_world::second::distinct_child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("distinct source root")),
            &marker,
        ),
        Some("successor-first") => crate::production_entry::pending_wal_world::successor::child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            crate::production_entry::pending_wal_world::successor::FIRST_OBJECT,
            1,
        ),
        Some("successor-first-full") => {
            crate::production_entry::pending_wal_world::successor::child(
                std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
                &marker,
                crate::production_entry::pending_wal_world::successor::FIRST_OBJECT,
                crate::production_entry::pending_wal_world::successor::FULL_BATCH,
            )
        }
        Some("successor-distinct") => crate::production_entry::pending_wal_world::successor::child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            crate::production_entry::pending_wal_world::successor::DISTINCT_OBJECT,
            1,
        ),
        other => panic!("unknown pending V3 child role: {other:?}"),
    }
}
