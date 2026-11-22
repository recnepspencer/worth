use std::sync::Arc;

#[derive(Debug)]
pub struct WorthQueryCandidateOccurrenceBinding(Arc<str>);

impl WorthQueryCandidateOccurrenceBinding {
    pub fn owner_derived(identity: Arc<str>) -> Self {
        Self(identity)
    }

    pub fn identity(&self) -> &str {
        &self.0
    }
}
