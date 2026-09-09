use std::sync::Arc;
use worth_query_installation::facade::{
    WorthQueryHostConditionalPredicateProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentProjector,
};
use worth_runtime_bridge::facade::{
    BridgeManagedClockObservationParts, BridgeSealedRuntimeAssembly,
};

use super::clock_observation::{
    ErasedClockObservationOutcome, ErasedClockObservationReceipt,
    WorthQueryConditionalClockObservationFailureKind,
};
use super::installation::ConditionalClockLease;
use super::signal_decision_reentry::WorthQueryConditionalTruthBasis;

mod authoritative_clock_progression;
mod bridge_clock_outcome;
mod clock_source_observation;
mod commit_watch;
mod due_wake_retention;
mod evaluation_affinity;
mod evaluation_binding;
mod installed_operation;
mod operation_cell;
mod operation_totals;
mod registry;
mod resource_observation;
mod runtime_rebinding;
pub(in crate::domain_computation::primary_graph::conditional_operation) use clock_source_observation::isolate_clock_source;
pub(in crate::domain_computation::primary_graph) use installed_operation::{
    WorthQueryConditionalRetainedResourceCounts, WorthQueryInstalledConditionalOperation,
    WorthQueryInstalledTemporalOperation, WorthQueryPreparedConditionalRuntimeBinding,
};
pub(in crate::domain_computation::primary_graph) use registry::WorthQueryConditionalOperationRegistry;
pub(in crate::domain_computation::primary_graph) use operation_cell::WorthQueryConditionalOperationCell;

#[rustfmt::skip]
impl<
        Schema, ApplicationOperation, Input, D, O, F, Node, Provider, Clock, Source, Query, Parameters, QueryResult, Scope, Projector,
        PrincipalBinding, PrincipalMapping, Principal, PrincipalIdentity, ScopeAspect, ScopeField, ScopeValue, ScopeWrite, ScopeUnit, PrincipalSource,
        QueryAuthorization, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField,
        RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization,
    > WorthQueryInstalledConditionalOperation<Schema>
    for WorthQueryInstalledTemporalOperation<
        WorthQueryInstalledTemporalConditionalOperation<
            Schema, ApplicationOperation, Input, D, O, F, Node, Provider,
            Clock, Source, Query, Parameters, QueryResult, Scope, Projector,
        >,
        super::reconstruction_authority::WorthQueryTemporalReconstructionAccess<
            Schema, PrincipalBinding, PrincipalMapping, Principal, PrincipalIdentity, Scope,
            ScopeAspect, ScopeField, ScopeValue, ScopeWrite, ScopeUnit, PrincipalSource, QueryAuthorization,
        >,
        super::operation_invocation::WorthQueryTemporalOperationExecution<
            Schema, ApplicationOperation, Input, Scope, Invoker, IntentEntity,
            IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit,
            RevisionAspect, RevisionField, RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit,
            LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization,
        >,
        Clock,
        Input,
    >
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
    ApplicationOperation: 'static,
    Input: Clone + Send + Sync + 'static,
    D: 'static,
    O: 'static,
    F: 'static,
    Node: 'static,
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Query: 'static,
    Parameters: 'static,
    QueryResult: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<Schema, Query> + 'static,
    Scope: 'static,
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
    PrincipalBinding: 'static,
    PrincipalMapping: 'static,
    Principal: 'static,
    PrincipalIdentity: worth_query_installation::facade::TypedApplicationIdentityValue + 'static,
    ScopeAspect: 'static,
    ScopeField: 'static,
    ScopeValue: worth_query_installation::facade::TypedApplicationValue + Clone + Send + Sync + 'static,
    ScopeWrite: worth_query_installation::facade::WritePosture + 'static,
    ScopeUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    PrincipalSource: super::reconstruction_authority::WorthQueryTemporalPrincipalSource<Schema>,
    QueryAuthorization: super::WorthQueryTemporalQueryAuthorization<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>,
    Invoker: super::operation_invocation::WorthQueryTemporalOperationInvoker<Schema, ApplicationOperation, Input, Scope>,
    IntentEntity: 'static,
    IdentityAspect: 'static,
    IdentityField: worth_query_installation::facade::OperationReads<ApplicationOperation> + 'static,
    IdentityValue: worth_query_installation::facade::TypedApplicationReadableValue + Clone + Send + 'static,
    IdentityWrite: worth_query_installation::facade::WritePosture + 'static,
    IdentityUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    RevisionAspect: 'static,
    RevisionField: worth_query_installation::facade::OperationReads<ApplicationOperation> + worth_query_installation::facade::OperationWrites<ApplicationOperation> + 'static,
    RevisionValue: worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue + worth_query_installation::facade::TypedApplicationReadableValue + Clone + Send + 'static,
    RevisionWrite: worth_query_installation::facade::WritableCapability + 'static,
    RevisionEquality: 'static,
    RevisionUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    LifecycleAspect: 'static,
    LifecycleField: worth_query_installation::facade::OperationReads<ApplicationOperation> + worth_query_installation::facade::OperationWrites<ApplicationOperation> + 'static,
    LifecycleValue: worth_query_installation::facade::TypedApplicationReadableValue + Clone + Send + Sync + 'static,
    LifecycleWrite: worth_query_installation::facade::WritableCapability + 'static,
    LifecycleEquality: 'static,
    LifecycleUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    Authorization: super::WorthQueryTemporalOperationAuthorization<Schema, ApplicationOperation, Input, Scope> + 'static,
{
    fn binding_identity(&self) -> &str {
        self.definition.binding_identity.support_identity()
    }

    fn installation_canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence {
        self.definition.installation_canonical_work
    }

    fn clock_lease(&self) -> Arc<ConditionalClockLease> {
        Arc::clone(&self.definition.clock_lease)
    }

    fn lowering_anchor(&self) -> Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering> {
        Arc::clone(&self.lowering_anchor)
    }

    fn select_product_binding(
        &mut self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> Result<(), super::installation::WorthQueryConditionalRuntimeInstallationDenial> {
        self.select_product_evaluation_binding(bridge, runtime, truth)
    }

    fn reconstruct(
        &mut self,
        _runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
    ) -> Result<(), super::installation::WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn authoritative_commit_routes(
        &self,
    ) -> (
        Vec<worth_relational::facade::transactions::RecordRef>,
        bool,
    ) {
        let mut records = std::collections::BTreeSet::new();
        let mut whole_graph = false;
        if self.active_affinity.is_some() {
            records.extend(self.commit_watch.records().cloned());
            whole_graph |= self.commit_watch.includes_whole_graph();
        }
        for binding in self.inactive_bindings.values() {
            records.extend(binding.commit_watch.records().cloned());
            whole_graph |= binding.commit_watch.includes_whole_graph();
        }
        (records.into_iter().collect(), whole_graph)
    }

    fn bootstrap_commit_route_pending(&self) -> bool {
        self.bootstrap_commit_catch_up_pending
    }

    #[rustfmt::skip]
    fn reconcile_reconstruction(&mut self, bridge: &mut BridgeSealedRuntimeAssembly) -> Result<(), super::installation::WorthQueryConditionalRuntimeInstallationDenial> {
        let Some(managed_clock) = self.managed_clock.as_ref() else { return Ok(()); };
        super::temporal_reconstruction::reconcile_temporal_intents(bridge, managed_clock, &mut self.reconstructed_intents)
    }

    fn prepare_derived_runtime_reinstallation(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
        bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryPreparedConditionalRuntimeBinding,
        super::installation::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let mut prepared = runtime_rebinding::prepare_temporal_runtime_binding(
            &self.definition.binding,
            bridge,
            self.active_lowering(),
            Arc::clone(&self.runtime_binding_identity),
            product,
        )?;
        prepared.authoritative_commit_cursor = runtime.primary_provider.conditional_commit_sequence();
        let reconstruction = super::temporal_reconstruction::reconstruct_temporal_intents(
            runtime, &self.definition.binding, &self.definition.reconstruction, self.definition.execution.identity_field,
            product,
        )?;
        prepared.reconstructed_intent_count = reconstruction.intents.len();
        prepared.commit_watch = commit_watch::WorthQueryConditionalCommitWatchSet::successor(
            None,
            &reconstruction.intents,
            prepared.lowering.as_ref(),
            reconstruction.intents.len(),
        )
        .map_err(|detail| {
            super::installation::WorthQueryConditionalRuntimeInstallationDenial::new(
                super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::RetentionCapacityExhausted,
                detail,
            )
        })?;
        prepared.authoritative_reconstruction = Box::new(reconstruction);
        Ok(prepared)
    }

    fn apply_derived_runtime_reinstallation(
        &mut self,
        prepared: WorthQueryPreparedConditionalRuntimeBinding,
    ) {
        self.bootstrap_lowering = prepared.lowering;
        self.lowering_anchor = Arc::clone(&self.bootstrap_lowering);
        self.active_affinity = Some(prepared.affinity);
        self.managed_clock = Some(prepared.managed_clock);
        self.authoritative_commit_cursor = prepared.authoritative_commit_cursor;
        self.bootstrap_commit_catch_up_pending = false;
        self.commit_watch = prepared.commit_watch;
        let reconstruction = *prepared
            .authoritative_reconstruction
            .downcast::<super::temporal_reconstruction::WorthQueryTemporalReconstruction<Clock, Input>>()
            .expect("prepared temporal reconstruction retains its installed binding type");
        self.reconstructed_intents = reconstruction.intents;
        self.reconstruction_work = reconstruction.work;
        self.retained_wakes.clear();
        self.inactive_bindings.clear();
        self.next_evaluation_binding_ordinal = 1;
    }

    #[rustfmt::skip]
    fn reconcile_prepared_runtime_reinstallation(&self, bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution, prepared: &mut WorthQueryPreparedConditionalRuntimeBinding) -> Result<(), super::installation::WorthQueryConditionalRuntimeInstallationDenial> {
        let reconstruction = prepared.authoritative_reconstruction.downcast_mut::<super::temporal_reconstruction::WorthQueryTemporalReconstruction<Clock, Input>>().expect("prepared temporal reconstruction retains its installed binding type");
        super::temporal_reconstruction::reconcile_prepared_temporal_intents(bridge, &prepared.managed_clock, &mut reconstruction.intents)
    }

    fn observe_clock(
        &mut self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> ErasedClockObservationOutcome {
        let observation = match isolate_clock_source(|| self.definition.binding.observe_clock_for_runtime()) {
            Ok(observation) => observation,
            Err(outcome) => return outcome,
        };
        let sequence = observation.sequence();
        let coordinate = observation.observed_time().nanoseconds();
        let signal_basis = match self.active_affinity.as_ref() {
            Some(affinity) => affinity.signal_basis(),
            None => {
                return authoritative_clock_progression::runtime_rejection(
                    "conditional clock observation requires a selected product binding"
                        .to_string(),
                )
            }
        };
        let relational_commit_ceiling = self
            .active_affinity
            .as_ref()
            .and_then(|affinity| affinity.relational_commit_ceiling());
        let managed_clock = self
            .managed_clock
            .as_ref()
            .expect("selected conditional operation retains a managed clock");
        let (watched_records, include_whole_graph) = self.authoritative_commit_routes();
        let mut authoritative = match authoritative_clock_progression::reconsider_authoritative_clock_work(
            authoritative_clock_progression::AuthoritativeClockWork {
                runtime,
                bridge,
                signal_basis,
                relational_branch: truth.product().relational_basis_descriptor().branch_id(),
                relational_commit_ceiling,
                cursor: &mut self.authoritative_commit_cursor,
                maximum_commits: self.definition.binding.bounds().maximum_due_wakes_per_observation(),
                watched_records,
                include_whole_graph,
                bootstrap_identity: self
                    .bootstrap_commit_catch_up_pending
                    .then_some(self.definition.binding_identity.support_identity()),
                retained_wakes: &mut self.retained_wakes,
                runtime_binding_identity: &self.runtime_binding_identity,
                runtime_capability_identity: self.runtime_capability_identity,
                truth,
            },
        ) {
            Ok(progress) => progress,
            Err(outcome) => return outcome,
        };
        if self.bootstrap_commit_catch_up_pending
            && authoritative.caught_up_to_latest
            && runtime
                .primary_provider
                .narrow_conditional_bootstrap_route_if_current(
                    self.definition.binding_identity.support_identity(),
                    self.authoritative_commit_cursor,
                )
        {
            self.bootstrap_commit_catch_up_pending = false;
        }
        let outcome = bridge.observe_managed_clock(BridgeManagedClockObservationParts {
            binding: managed_clock,
            source_identity: observation.source().as_str(),
            timeline_identity: observation.timeline().as_str(),
            sequence,
            observed_coordinate: coordinate,
        });
        let retain_and_reenter = |accepted| {
            let receipt = due_wake_retention::retain_due(
                accepted,
                bridge,
                truth,
                &authoritative.granular_invalidations,
                signal_basis,
                &self.runtime_binding_identity,
                self.runtime_capability_identity,
                &mut self.retained_wakes,
            );
            let operation = self
                .definition
                .binding
                .clocked_node()
                .provider()
                .node()
                .operation()
                .application_operation();
            let counts = super::application_operation_reentry::reenter_retained_wakes(
                runtime,
                truth.product(),
                bridge,
                managed_clock,
                operation,
                &self.definition.reconstruction,
                &self.definition.execution,
                &mut self.reconstructed_intents,
                &mut self.retained_wakes,
                &self.runtime_canonical_identity,
            );
            authoritative.granular_invalidations =
                super::authoritative_reconsideration::promote_performed_signal_deliveries(
                    std::mem::take(&mut authoritative.granular_invalidations),
                    &mut self.retained_wakes,
                );
            due_wake_retention::complete_clock_receipt(
                receipt,
                counts,
                authoritative,
                &mut self.retained_wakes,
                &mut self.operation_totals,
            )
        };
        bridge_clock_outcome::map_bridge_clock_outcome(outcome, retain_and_reenter)
    }

    fn retained_resource_counts(&self) -> WorthQueryConditionalRetainedResourceCounts {
        resource_observation::retained_resource_counts(self)
    }

    fn reconstruction_work(
        &self,
    ) -> super::temporal_reconstruction::WorthQueryTemporalReconstructionWork {
        self.reconstruction_work
    }

    fn lifecycle_resources(
        &self,
    ) -> super::lifecycle_inventory::WorthQueryConditionalOperationLiveness {
        resource_observation::lifecycle_resources(self)
    }
}

#[cfg(test)]
mod tests;
