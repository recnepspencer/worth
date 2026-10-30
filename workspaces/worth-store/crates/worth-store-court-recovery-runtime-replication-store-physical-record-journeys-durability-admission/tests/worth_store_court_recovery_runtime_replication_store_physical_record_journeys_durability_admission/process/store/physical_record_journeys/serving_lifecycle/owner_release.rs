use std::path::Path;

use super::serving_from_initialization;

pub(super) fn assert_abort_drop_and_unwind_release_owner(parent: &Path) {
    let abort_root = parent.join("abort-store");
    let aborting = serving_from_initialization(&abort_root);
    let abort_observer = aborting.observer();
    aborting.abort();
    assert_eq!(abort_observer.record_counters().owner_live(), 0);

    let drop_root = parent.join("drop-store");
    let dropped = serving_from_initialization(&drop_root);
    let drop_observer = dropped.observer();
    drop(dropped);
    assert_eq!(drop_observer.record_counters().owner_live(), 0);

    let panic_root = parent.join("panic-store");
    let panicking = serving_from_initialization(&panic_root);
    let panic_observer = panicking.observer();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _owned = panicking;
        panic!("controlled lifecycle panic");
    }));
    assert!(result.is_err());
    assert_eq!(panic_observer.record_counters().owner_live(), 0);
}
