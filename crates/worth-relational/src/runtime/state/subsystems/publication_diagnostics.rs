use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::diagnostics::data::RelationalDiagnosticArtifact;

#[derive(Debug, Default)]
struct DiagnosticArtifactState {
    artifacts: Vec<RelationalDiagnosticArtifact>,
    active_by_thread: HashMap<std::thread::ThreadId, Vec<u64>>,
    captured: HashMap<u64, Vec<RelationalDiagnosticArtifact>>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RelationalDiagnosticArtifactStore {
    state: Arc<Mutex<DiagnosticArtifactState>>,
}

#[derive(Debug)]
pub(crate) struct RelationalDiagnosticOperationCapture {
    store: RelationalDiagnosticArtifactStore,
    capture_id: u64,
    _identity: Arc<()>,
    thread_id: std::thread::ThreadId,
    active: bool,
}

impl RelationalDiagnosticArtifactStore {
    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        let state = DiagnosticArtifactState {
            artifacts: self.snapshot(),
            ..DiagnosticArtifactState::default()
        };
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub(crate) fn begin_operation_capture(&self) -> RelationalDiagnosticOperationCapture {
        let thread_id = std::thread::current().id();
        let mut state = self.lock();
        // Only live captures are looked up. Their guards retain distinct Arc
        // headers until finish removes every directory entry, so reuse after
        // release cannot collide with a live capture and no counter can exhaust.
        let identity = Arc::new(());
        let capture_id = Arc::as_ptr(&identity) as usize as u64;
        state
            .active_by_thread
            .entry(thread_id)
            .or_default()
            .push(capture_id);
        state.captured.insert(capture_id, Vec::new());
        RelationalDiagnosticOperationCapture {
            store: self.clone(),
            capture_id,
            _identity: identity,
            thread_id,
            active: true,
        }
    }

    pub(crate) fn push(&self, artifact: RelationalDiagnosticArtifact) {
        let thread_id = std::thread::current().id();
        let mut state = self.lock();
        let active = state
            .active_by_thread
            .get(&thread_id)
            .cloned()
            .unwrap_or_default();
        for capture_id in active {
            if let Some(captured) = state.captured.get_mut(&capture_id) {
                captured.push(artifact.clone());
            }
        }
        state.artifacts.push(artifact);
    }

    pub(crate) fn snapshot(&self) -> Vec<RelationalDiagnosticArtifact> {
        self.lock().artifacts.clone()
    }

    pub(crate) fn count(&self) -> usize {
        self.lock().artifacts.len()
    }

    pub(crate) fn since(&self, start: usize) -> Vec<RelationalDiagnosticArtifact> {
        let state = self.lock();
        state.artifacts[start.min(state.artifacts.len())..].to_vec()
    }

    fn finish_capture(
        &self,
        capture_id: u64,
        thread_id: std::thread::ThreadId,
    ) -> Vec<RelationalDiagnosticArtifact> {
        let mut state = self.lock();
        if let Some(active) = state.active_by_thread.get_mut(&thread_id) {
            active.retain(|active_id| *active_id != capture_id);
            if active.is_empty() {
                state.active_by_thread.remove(&thread_id);
            }
        }
        state.captured.remove(&capture_id).unwrap_or_default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, DiagnosticArtifactState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl RelationalDiagnosticOperationCapture {
    pub(crate) fn finish(mut self) -> Vec<RelationalDiagnosticArtifact> {
        let artifacts = self.store.finish_capture(self.capture_id, self.thread_id);
        self.active = false;
        artifacts
    }
}

impl Drop for RelationalDiagnosticOperationCapture {
    fn drop(&mut self) {
        if self.active {
            let _ = self.store.finish_capture(self.capture_id, self.thread_id);
        }
    }
}
