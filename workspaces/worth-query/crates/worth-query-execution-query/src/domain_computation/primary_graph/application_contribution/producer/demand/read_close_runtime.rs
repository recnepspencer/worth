//! Runtime read-request close on `WorthQueryAdmittedOutputDemand`.

use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputReadCloseDenial, WorthQueryProducerOutputFamily,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::PendingProducerPublication;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<S: ApplicationSchema + 'static, F: WorthQueryProducerOutputFamily<S>>
    WorthQueryAdmittedOutputDemand<S, F>
{
    pub(crate) fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        phase
            .checked_execution_request_for(&runtime.runtime.product_runtime)
            .map_err(WorthQueryOutputReadCloseDenial::Execution)?;
        if self.runtime_authority != runtime.runtime.authority_identity().as_u64()
            || self.schema_binding != runtime.installed_schema.binding_identity()
            || self
                .observed_source
                .selected_product_occurrence()
                .is_some_and(|original| original != branch.occurrence())
        {
            return Err(WorthQueryOutputReadCloseDenial::ForeignDemand);
        }
        if let Some(denial) = self.terminal_caller_denial.as_mut() {
            denial.close_read_cleanup(phase, runtime)?;
        }
        if let Some(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint::PublicationPending { owner, .. }) =
            self.unpublished_selected_checkpoint.as_mut()
        {
            owner
                .as_any_mut()
                .downcast_mut::<PendingProducerPublication>()
                .expect("unpublished checkpoint holds PendingProducerPublication")
                .close(phase, runtime)?;
            self.unpublished_selected_checkpoint = None;
        }
        if self.unpublished_selected_checkpoint.is_some() {
            return Err(WorthQueryOutputReadCloseDenial::NativePublicationRetained);
        }
        if let Some(interest) = self.interest.as_ref() {
            runtime.close_unpublished_checkpoint(
                phase,
                interest,
                &mut self.unpublished_selected_checkpoint,
            )?;
            interest.require_read_closeable()?;
        }
        self.required_continuations
            .close_read_request(phase, runtime, branch)?;
        // The visitor has discharged every owned successor; close only these interests.
        self.close();
        Ok(())
    }
}
