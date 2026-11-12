//! Existing outputs verified at this exact proposed edit, never Current or settlement.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        retained_decision_facts::StorageControl,
        WorthQueryApplicationAttemptDenial as AttemptDenial,
    },
    application_checkpoint::facts::{FactPayload, FactSequence},
    application_contribution::producer::execution::denial::candidate_support_denied,
    application_query::AppliedQuerySource,
    WorthQueryApplicationObservedFact as Fact,
};
use worth_relational::facade::{
    identity::EntityId,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};
mod custody;
mod expectations;
mod qualification;
type Pin = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
pub(in crate::domain_computation::primary_graph) struct VerifiedAppliedParent {
    source: EntityId,
    family: &'static str,
    pin: Pin,
    facts: FactPayload,
}
impl VerifiedAppliedParent {
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> EntityId {
        self.source
    }
    pub(in crate::domain_computation::primary_graph) fn family(&self) -> &'static str {
        self.family
    }
    pub(in crate::domain_computation::primary_graph) fn pin(&self) -> &Pin {
        &self.pin
    }
    pub(in crate::domain_computation::primary_graph) fn changed_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        complete: &Pin,
    ) -> Result<Option<&Fact>, AttemptDenial> {
        for fact in self.facts.iter() {
            if !fact
                .support_equal_after_own_effect(execution, runtime, before, &self.pin, complete)?
            {
                return Ok(Some(fact));
            }
        }
        Ok(None)
    }
}
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn verify_applied_output_parent<
        F: WorthQueryProducerOutputFamily<S>,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        before: &crate::basis::WorthQueryProductBranchLease,
        pin: &Pin,
        source: &AppliedQuerySource<FamilySourceQuery<S, F>, FamilySourceValue<S, F>>,
        prior: Option<&worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    ) -> Result<Option<VerifiedAppliedParent>, WorthQueryOutputDemandDenial> {
        if source.candidate.qualification() != pin.candidate_identity().qualification()
            || pin.candidate_identity().proposal().branch_observation()
                != before.relational_basis().reference()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                F::IDENTITY,
            ));
        }
        let graph = &self.runtime.primary_provider.graph;
        let bindings = self.installed_producers.family_output_bindings::<F>();
        let observation = before.observation();
        let read_identity =
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(observation);
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source.root());
        let mut lineage = graph
            .output_lineage
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        self.record_recovered_output_candidates(
            Some(execution),
            source.partition(),
            &source.checkpoint_identity(),
            &read_identity,
            scope,
            &bindings,
            &mut lineage,
        )
        .map_err(|stop| selected_basis::selection_lookup_denial(stop, F::IDENTITY))?;
        let candidates = lineage
            .retained_output_candidates_metered(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                &bindings,
                source.partition(),
                execution,
            )
            .map_err(|stop| selected_basis::selection_lookup_denial(stop, F::IDENTITY))?;
        drop(lineage);
        for candidate in candidates {
            let (_, installed) = self
                .installed_producers
                .select_exact::<F>(candidate.binding)?;
            let origin_matches = match prior {
                Some(prior) => {
                    custody::matches(&graph.layout, execution, &candidate, prior, F::IDENTITY)?
                }
                // Native's current route reader found no publication at C.
                // Independent ordinary rows retain Query's original execution
                // or authenticated checkpoint, not fabricated Native custody.
                None => {
                    installed.declaration.ownership
                        == worth_foundational::facade::ProducerPublicationOwnership::Independent
                        && candidate.required_publication.is_none()
                }
            };
            if candidate.historical_selection.is_some()
                || !origin_matches
                || !source_identity_matches(
                    &candidate,
                    source.runtime_source_identity(),
                    crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(source.checkpoint_identity()),
                )
                || !implementation_matches::<S, F>(self, &candidate)?
            {
                continue;
            }
            let (_, installed) = self
                .installed_producers
                .select_exact::<F>(candidate.binding)?;
            let original_ready = candidate.original_ready_key.as_ref().is_some_and(|key| {
                key.family_identity() == F::IDENTITY
                    && key.producer_identity() == installed.declaration.identity
                    && self.runtime.output_demands.has_executed_ready(
                        key,
                        scope,
                        observation.lifecycle_incarnation(),
                        candidate.correspondence.as_ref(),
                        &candidate.idempotency_key_identity,
                        &candidate.settlement_identity,
                    )
            });
            if !original_ready
                && !exact_source_contract(&candidate, installed)
                && candidate
                    .completed_coupled_receipt(
                        source.runtime_source_identity(),
                        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(source.checkpoint_identity()),
                    )
                    .is_none()
            {
                continue;
            }
            let Some(facts) = expectations::retain(&candidate, execution, request, F::IDENTITY)?
            else {
                continue;
            };
            if qualification::holds(
                self,
                execution,
                before,
                pin,
                &candidate,
                &facts,
                F::IDENTITY,
            )? {
                return Ok(Some(VerifiedAppliedParent {
                    source: source.root(),
                    family: F::IDENTITY,
                    pin: pin.clone(),
                    facts,
                }));
            }
        }
        Ok(None)
    }
}
