//! Compiled body reads extend a native decision; they never replace its basis or custody.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::invariant_projection::{
    BorrowedOperationProjection, ObservedComputationInputs,
};
use worth_execution::ExecutionAllocationPolicy;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub(in crate::domain_computation::primary_graph) fn complete_compiled_body_projection(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        projection: BorrowedOperationProjection<Schema, Operation>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if projection.admission != self.admission.admission_identity()
            || projection.basis
                != WorthQueryProductBranchReadIdentity::from_observation(
                    self.lease.product().observation(),
                )
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                self.admission.operation(),
            ));
        }
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let (keys, (computations, retained)) = projection.facts.into_expected().map_err(|key| {
            denial(
                WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                format!("{key:?}"),
            )
        })?;
        // The installed scalar body has no partitioned computation completion.
        // Do not silently discard a future compute-producing extension here.
        if !matches!(computations, ObservedComputationInputs::None)
            || !self.computation_fact_ordinals.is_empty()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                self.admission.operation(),
            ));
        }
        let scope = super::super::read_scope::WorthQueryApplicationReadScope::projected(
            self.admission.scope_entity_id(),
            projection.scope,
        );
        let mut facts = Vec::new();
        for key in keys.iter() {
            let (_installed_position, fact) =
                super::projected_completion::observe_projection_dependency(
                    execution,
                    &self.admission,
                    &self.lease,
                    &scope,
                    key,
                )?;
            facts.push(fact);
        }
        drop(keys);
        facts.extend(projection.source_facts.into_values());
        // Both proven sets came from this one lease. Keep the complete native
        // prefix (and its existing indices), even when the body also reads a locus.
        // Publication must satisfy BOTH observations; a stale native obligation
        // cannot be replaced by a new body observation. Installed scope
        // positions describe only the completed projection prefix; every
        // appended fact was admitted against its installed scope just above.
        self.append_completed_facts(facts, allocation)?;
        self.consumed_outputs
            .extend(projection.consumed_outputs.into_values());
        let retained = retained.seal_at(None, self.lease.snapshot()).map_err(|()| {
            denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                self.admission.operation(),
            )
        })?;
        self.admission.record_completed_handler_facts(
            CompletedHandlerFactBoundary::from_completed_read(self.facts.len()),
            retained,
        );
        Ok(())
    }
}
