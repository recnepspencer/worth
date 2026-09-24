use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

const MAXIMUM_REGISTERED_VIEWS: usize = 64;

pub(super) trait RegisteredDerivedView: Send + Sync {
    fn dispose(&self);
}

#[derive(Default)]
struct RegistryState {
    next_id: u64,
    views: BTreeMap<u64, Weak<dyn RegisteredDerivedView>>,
}

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ManagedDerivedViewRegistry {
    state: Mutex<RegistryState>,
}

impl ManagedDerivedViewRegistry {
    pub(super) fn register(self: &Arc<Self>, view: &Arc<dyn RegisteredDerivedView>) -> Option<u64> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.views.retain(|_, view| view.strong_count() > 0);
        if state.views.len() >= MAXIMUM_REGISTERED_VIEWS {
            return None;
        }
        let id = state.next_id.checked_add(1)?;
        state.next_id = id;
        state.views.insert(id, Arc::downgrade(view));
        Some(id)
    }

    pub(super) fn unregister(&self, id: u64) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .views
            .remove(&id);
    }

    pub(in crate::domain_computation::primary_graph) fn dispose(&self) {
        let views = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut state.views)
                .into_values()
                .filter_map(|view| view.upgrade())
                .collect::<Vec<_>>()
        };
        for view in views {
            view.dispose();
        }
    }
}

impl Drop for ManagedDerivedViewRegistry {
    fn drop(&mut self) {
        self.dispose();
    }
}
