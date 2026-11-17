//! Debug-only names for actual physical shares; observations cannot retain Serving.
use std::sync::{Arc, Mutex, Weak};
use worth_store::physical_runtime::ServingPhysicalRuntime;

struct Registration {
    serving: Weak<ServingPhysicalRuntime>,
    owner: &'static str,
    alive: Weak<()>,
}
static OWNERS: Mutex<Vec<Registration>> = Mutex::new(Vec::new());

pub(crate) fn register<T>(serving: &Arc<ServingPhysicalRuntime>) -> Arc<()> {
    let alive = Arc::new(());
    let mut owners = OWNERS.lock().unwrap_or_else(|error| error.into_inner());
    owners.retain(|entry| entry.alive.strong_count() != 0);
    owners.push(Registration {
        serving: Arc::downgrade(serving),
        owner: std::any::type_name::<T>(),
        alive: Arc::downgrade(&alive),
    });
    alive
}

/// Reports debug registrations only, never authority to close or release a share.
pub fn relational_physical_owner_labels(
    serving: &Arc<ServingPhysicalRuntime>,
) -> Vec<&'static str> {
    let owners = OWNERS.lock().unwrap_or_else(|error| error.into_inner());
    owners
        .iter()
        .filter(|entry| {
            entry.serving.as_ptr() == Arc::as_ptr(serving) && entry.alive.strong_count() != 0
        })
        .map(|entry| entry.owner)
        .collect()
}
