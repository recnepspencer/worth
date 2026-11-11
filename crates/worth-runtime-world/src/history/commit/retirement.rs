//! Retire one original commit share and preserve its immutable metadata.
use super::*;
use crate::basis::CompositeBasisRetirement;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchBasisRetirementDenial, runtime::RelationalRuntime,
};
#[derive(Debug)]
pub(crate) struct CompositeCommitRetirement {
    shared: Option<Arc<CompositeRuntimeWorldCommit>>,
    original: Option<CompositeRuntimeWorldCommit>,
    basis: Option<CompositeBasisRetirement>,
    complete: bool,
}
impl CompositeCommitRetirement {
    pub(crate) fn new(shared: Arc<CompositeRuntimeWorldCommit>) -> Self {
        Self {
            shared: Some(shared),
            original: None,
            basis: None,
            complete: false,
        }
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RelationalBranchBasisRetirementDenial> {
        if self.complete {
            return Ok(true);
        }
        if let Some(shared) = self.shared.take() {
            self.original = Arc::into_inner(shared);
            if let Some(original) = &mut self.original {
                self.basis = Some(
                    original
                        .basis
                        .take()
                        .expect("original commit basis")
                        .into_home_retirement(),
                );
            } else {
                self.complete = true;
            }
            return Ok(self.complete);
        }
        self.complete = self
            .basis
            .as_mut()
            .expect("original commit basis retirement")
            .advance(request, native)?;
        Ok(self.complete)
    }
}
