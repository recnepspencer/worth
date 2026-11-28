//! Consume actual composite shares without revoking an independent basis.
use super::*;
use crate::basis::NativeBasisRetirement;
use worth_execution::ExecutionRequest;
use worth_relational::facade::branch::RelationalBranchBasisRetirementDenial;
use worth_relational::facade::runtime::RelationalRuntime;

#[derive(Debug)]
pub(crate) struct CompositeBasisRetirement {
    shared: Option<Arc<AdmittedCompositeRuntimeWorldBasisInner>>,
    original: Option<AdmittedCompositeRuntimeWorldBasisInner>,
    native: Option<NativeBasisRetirement>,
}
impl AdmittedCompositeRuntimeWorldBasis {
    pub(crate) fn into_home_retirement(self) -> CompositeBasisRetirement {
        CompositeBasisRetirement {
            shared: Some(self.inner),
            original: None,
            native: None,
        }
    }
}
impl CompositeBasisRetirement {
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RelationalBranchBasisRetirementDenial> {
        if let Some(shared) = self.shared.take() {
            self.original = Arc::into_inner(shared);
            if let Some(original) = &mut self.original {
                self.native = Some(NativeBasisRetirement::new(
                    original.basis.take_native_for_retirement(),
                ));
                return Ok(false);
            }
        }
        match &mut self.native {
            Some(task) => task.advance(request, native),
            None => Ok(true),
        }
    }
}
