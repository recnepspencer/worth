use std::collections::BTreeMap;
use std::sync::Arc;

use crate::runtime::state::subsystems::RuntimeOwnedState;
use crate::simulation::data::CompiledExecutionArtifact;

/// The compiled artifacts a runtime has published, keyed by issue order.
#[derive(Debug, Default)]
struct CompiledArtifacts {
    by_id: BTreeMap<u64, Arc<CompiledExecutionArtifact>>,
    next_compiled_artifact_id: u64,
}

/// Compiled execution artifacts, owned behind their own lock so a simulation
/// can publish one without exclusive access to the runtime.
///
/// Artifacts are handed out by shared ownership, so a reader carries one away
/// from the store rather than borrowing into it.
#[derive(Debug, Default)]
pub(crate) struct CompiledArtifactStore {
    artifacts: RuntimeOwnedState<CompiledArtifacts>,
}

impl Clone for CompiledArtifactStore {
    /// Cloning binds a second handle to the same artifact authority, so a
    /// preparation binding publishes into the runtime's own store. Use
    /// [`CompiledArtifactStore::detached`] when an independent copy is wanted.
    fn clone(&self) -> Self {
        Self {
            artifacts: self.artifacts.share(),
        }
    }
}

impl CompiledArtifactStore {
    pub(crate) fn new() -> Self {
        Self {
            artifacts: RuntimeOwnedState::new(CompiledArtifacts {
                by_id: BTreeMap::new(),
                next_compiled_artifact_id: 1,
            }),
        }
    }

    pub(crate) fn detached(&self) -> Self {
        let source = self.artifacts.read();
        Self {
            artifacts: RuntimeOwnedState::new(CompiledArtifacts {
                by_id: source.by_id.clone(),
                next_compiled_artifact_id: source.next_compiled_artifact_id,
            }),
        }
    }

    pub(crate) fn compiled_artifact(
        &self,
        compiled_artifact_id: u64,
    ) -> Option<Arc<CompiledExecutionArtifact>> {
        self.artifacts
            .read()
            .by_id
            .get(&compiled_artifact_id)
            .cloned()
    }

    pub(crate) fn next_compiled_artifact_id(&self) -> u64 {
        self.artifacts.read().next_compiled_artifact_id
    }

    pub(crate) fn store_compiled_artifact(&self, artifact: CompiledExecutionArtifact) -> u64 {
        let mut artifacts = self.artifacts.write();
        let artifact_id = artifacts.next_compiled_artifact_id;
        artifacts.next_compiled_artifact_id += 1;
        artifacts.by_id.insert(artifact_id, Arc::new(artifact));
        artifact_id
    }
}
