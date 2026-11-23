//! Check the live memory authority before copying retained evidence.
use super::*;
use crate::history::commit::{RelationalCommitArtifact, RelationalCommitIndexEntry};
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn export_captured_memory_component(
        &self,
        request: &ExecutionRequest<'_, '_>,
        component: &ObservedRelationalMemoryComponent,
        captured: Option<(&RelationalCommitIndexEntry, &RelationalCommitArtifact)>,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::request_custody::check_request(request).map_err(Refusal::Request)?;
        let meaning = match component.original.as_ref() {
            MemoryComponentOrigin::Live(value) => value,
            MemoryComponentOrigin::Interim(_) => {
                return self.export_memory_component(request, component)
            }
        };
        if !meaning.owner.same_origin(&self.services.origin_identity()) {
            return Err(Refusal::Basis(
                crate::branch::basis_denial::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        super::capture::copy_bound_original(request, meaning, captured)
    }
}
