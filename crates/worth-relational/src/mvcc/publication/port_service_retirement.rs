//! The publication receiver lends only its own alias slot to Native retirement.
use super::RelationalPublicationPort;
use crate::durability::medium::{RelationalDurableMediumBinding, RelationalMediumRetirementDenial};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
impl RelationalPublicationPort {
    pub(crate) fn retire_home_service_medium_aliases(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &crate::runtime::RelationalRuntime,
        fork: &mut Option<Arc<RelationalDurableMediumBinding>>,
    ) -> Result<(), RelationalMediumRetirementDenial> {
        owner
            .durability
            .retire_local_service_medium_aliases(request, fork, &mut self.medium)
    }
}
