//! The original submitted subject and its actual acknowledgement travel with
//! existing attempt resources through abandonment and retained materialization.
use super::{ActualWorldPublication, WorldDurableArtifact, WorldDurableMediumBinding};
use std::sync::{Arc, Mutex};
use worth_proof::DurablyCommitted;

pub(crate) struct WorldDurableOpenCustody {
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) subject: WorldDurableArtifact,
    committed: Mutex<Option<DurablyCommitted<ActualWorldPublication>>>,
}
impl std::fmt::Debug for WorldDurableOpenCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldDurableOpenCustody")
            .field("subject", &self.subject)
            .finish_non_exhaustive()
    }
}
impl WorldDurableOpenCustody {
    pub(crate) fn submitted(
        binding: Arc<WorldDurableMediumBinding>,
        subject: WorldDurableArtifact,
    ) -> Arc<Self> {
        Arc::new(Self {
            binding,
            subject,
            committed: Mutex::new(None),
        })
    }
    pub(crate) fn acknowledge(&self, committed: DurablyCommitted<ActualWorldPublication>) {
        // The binding submitted this subject and retains it until this exact
        // actual committed result is returned. Preserve that move-only proof.
        *self
            .committed
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(committed);
    }
}
