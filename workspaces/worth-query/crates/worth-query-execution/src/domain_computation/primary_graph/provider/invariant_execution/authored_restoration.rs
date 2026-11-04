//! Extend one candidate with the exact Native inverse before complete-state validation.
use super::*;
use worth_relational::facade::mvcc::RequiredOutputPublication;

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn prepare_authored_restoration(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut material: ApplicationInvariantCandidateMaterial,
    ) -> Result<ApplicationInvariantCandidateMaterial, WorthQueryInvariantExecutionFailure> {
        let Some((retirement, original, target, co_anchors)) = &material.authored_restoration
        else {
            return Ok(material);
        };
        let publisher = self
            .graph
            .application_publication
            .as_ref()
            .ok_or_else(owner_failure)?;
        let plan = self
            .graph
            .with_runtime(execution, |runtime| {
                runtime.prepare_authored_source_restoration_with_co_anchors(
                    execution,
                    &publisher.native,
                    material.product.observation().basis().relational_basis(),
                    *retirement,
                    (*original, target.clone()),
                    co_anchors,
                )
            })
            .map_err(map_validation_failure)?;
        let Some(plan) = plan else {
            return Ok(material);
        };
        let original = std::mem::replace(
            &mut material.batch,
            worth_relational::facade::transactions::WorkerIntentBatch::new(String::new()),
        );
        material.batch = plan
            .extend_batch(execution, original)
            .map_err(map_validation_failure)?;
        material.restoration_plan = Some(plan);
        Ok(material)
    }

    pub(super) fn bind_independent_restoration(
        &self,
        material: &mut ApplicationInvariantCandidateMaterial,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        let Some(plan) = material.restoration_plan.take() else {
            return Ok(());
        };
        let publisher = self
            .graph
            .application_publication
            .as_ref()
            .ok_or_else(owner_failure)?;
        let admission = publisher
            .owner
            .admit_authored_output_retirements(plan.authorities())
            .ok_or_else(owner_failure)?;
        material.restoration_publication =
            Some(RequiredOutputPublication::AuthoredCascadeRestoration {
                admission,
                restoration: plan,
            });
        Ok(())
    }
}
