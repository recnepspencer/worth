//! The medium a runtime owns over the test medium's durable contents, and
//! reaching those contents from a runtime, as a home builds a fresh journal
//! medium over its Store each time it opens.

use std::sync::Arc;

use super::TestDurableMedium;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::{RelationalDurableMedium, RelationalDurablePublication};
use crate::runtime::RelationalRuntime;

/// One runtime's medium over shared test durable contents. It is not
/// cloneable: like any medium, it is moved into exactly one runtime.
#[derive(Debug)]
pub(crate) struct TestDurableMediumAttachment {
    contents: Arc<TestDurableMedium>,
}

impl TestDurableMediumAttachment {
    pub(super) fn new(contents: Arc<TestDurableMedium>) -> Self {
        Self { contents }
    }
}

impl RelationalDurableMedium for TestDurableMediumAttachment {
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        self.contents.publish(artifact)
    }

    fn in_crate_test_medium(&self) -> Option<Arc<TestDurableMedium>> {
        Some(Arc::clone(&self.contents))
    }
}

/// The test durable contents a runtime publishes to.
#[derive(Debug, Clone)]
pub(crate) struct AttachedTestMedium(Arc<TestDurableMedium>);

impl AttachedTestMedium {
    /// A fresh medium over the same contents, for another runtime to own, as
    /// a reopened home builds its own journal medium over the same Store.
    pub(crate) fn attachment(&self) -> Box<dyn RelationalDurableMedium> {
        self.0.attachment()
    }
}

impl std::ops::Deref for AttachedTestMedium {
    type Target = TestDurableMedium;

    fn deref(&self) -> &TestDurableMedium {
        &self.0
    }
}

/// The test durable contents `runtime` publishes to, if it owns a medium.
pub(crate) fn attached_durable_medium(
    runtime: &RelationalRuntime,
) -> Option<Arc<TestDurableMedium>> {
    runtime.durability.medium().map(|binding| {
        binding
            .in_crate_test_medium()
            .expect("durable test runtimes publish to the in-crate test medium")
    })
}

/// The in-crate test medium contents `runtime` publishes to.
pub(crate) fn attached_test_medium(runtime: &RelationalRuntime) -> AttachedTestMedium {
    AttachedTestMedium(
        attached_durable_medium(runtime).expect("runtime publishes to a durable medium"),
    )
}

/// Move `medium` into a freshly built runtime before it recovers.
pub(crate) fn bind_durable_medium(
    recovering: &mut RelationalRuntime,
    medium: Box<dyn RelationalDurableMedium>,
) {
    recovering
        .unshared_state_mut()
        .expect("a freshly built runtime is unshared")
        .durability
        .attach_medium(medium);
}
