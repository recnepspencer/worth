//! Reaching the test medium a runtime publishes to, and binding a fresh
//! runtime to it, as a home builds its runtime with the medium it recovers.

use std::sync::Arc;

use super::TestDurableMedium;
use crate::durability::medium::RelationalDurableMedium;
use crate::runtime::RelationalRuntime;

/// The test medium a runtime is bound to, shared with that runtime.
#[derive(Debug, Clone)]
pub(crate) struct AttachedTestMedium(Arc<dyn RelationalDurableMedium>);

impl AttachedTestMedium {
    /// The medium as the runtime holds it, for binding another runtime.
    pub(crate) fn shared(&self) -> Arc<dyn RelationalDurableMedium> {
        Arc::clone(&self.0)
    }
}

impl std::ops::Deref for AttachedTestMedium {
    type Target = TestDurableMedium;

    fn deref(&self) -> &TestDurableMedium {
        self.0
            .in_crate_test_medium()
            .expect("durable test runtimes publish to the in-crate test medium")
    }
}

/// The durable medium `runtime` publishes to, if it has one.
pub(crate) fn attached_durable_medium(
    runtime: &RelationalRuntime,
) -> Option<Arc<dyn RelationalDurableMedium>> {
    runtime.durability.medium().map(|binding| binding.medium())
}

/// The in-crate test medium `runtime` publishes to.
pub(crate) fn attached_test_medium(runtime: &RelationalRuntime) -> AttachedTestMedium {
    let medium = attached_durable_medium(runtime).expect("runtime publishes to a durable medium");
    assert!(
        medium.in_crate_test_medium().is_some(),
        "durable test runtimes publish to the in-crate test medium"
    );
    AttachedTestMedium(medium)
}

/// Bind a freshly built runtime to `medium` before it recovers.
pub(crate) fn bind_durable_medium(
    recovering: &mut RelationalRuntime,
    medium: Arc<dyn RelationalDurableMedium>,
) {
    recovering
        .unshared_state_mut()
        .expect("a freshly built runtime is unshared")
        .durability
        .attach_medium(medium);
}
