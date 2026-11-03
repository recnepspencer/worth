//! Consume actual composite shares without revoking an independent basis.
use super::*;
use crate::basis::NativeBasisRetirement;
use worth_execution::ExecutionRequest;
use worth_relational::facade::branch::RelationalBranchBasisRetirementDenial;
use worth_relational::facade::runtime::RelationalRuntime;

pub(crate) struct CompositeBasisRetirement {
    shared: Option<Arc<AdmittedCompositeRuntimeWorldBasisInner>>,
    original: Option<AdmittedCompositeRuntimeWorldBasisInner>,
    native: Option<NativeBasisRetirement>,
    complete: bool,
}
impl AdmittedCompositeRuntimeWorldBasis {
    pub(crate) fn into_home_retirement(self) -> CompositeBasisRetirement {
        CompositeBasisRetirement {
            shared: Some(self.inner),
            original: None,
            native: None,
            complete: false,
        }
    }
}
impl CompositeBasisRetirement {
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
                self.native = Some(NativeBasisRetirement::new(
                    original.basis.take_native_for_retirement(),
                ));
            } else {
                self.complete = true;
            }
            return Ok(self.complete);
        }
        self.complete = self
            .native
            .as_mut()
            .expect("original composite has Native retirement")
            .advance(request, native)?;
        Ok(self.complete)
    }
}
