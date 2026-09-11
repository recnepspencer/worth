use std::{collections::BTreeMap, sync::Arc};

use worth_query_installation::facade::{
    WorthQueryHostConditionalPredicateProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentProjector,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalSignalBasisBinding, BridgeInstalledConditionalLowering,
    BridgeManagedClockBinding, BridgeSealedRuntimeAssembly,
};

use super::super::{
    signal_decision_reentry::WorthQueryRetainedConditionalWake,
    temporal_reconstruction::{
        WorthQueryReconstructedTemporalIntent, WorthQueryTemporalReconstructionWork,
    },
};

pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryInactiveTemporalEvaluationBinding<
    Clock,
    Input,
> {
    pub(super) affinity: super::evaluation_affinity::WorthQueryConditionalEvaluationAffinity,
    pub(super) managed_clock: BridgeManagedClockBinding,
    pub(super) retained_wakes: Vec<WorthQueryRetainedConditionalWake>,
    pub(super) pending_direct_delivery: super::direct_delivery::WorthQueryPendingDirectDelivery,
    pub(super) reconstructed_intents:
        BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    pub(super) reconstruction_work: WorthQueryTemporalReconstructionWork,
    pub(super) authoritative_commit_cursor: u64,
    pub(super) commit_watch: super::commit_watch::WorthQueryConditionalCommitWatchSet,
}

#[rustfmt::skip]
impl<
        Schema, ApplicationOperation, Input, D, O, F, Node, Provider, Clock, Source, Query, Parameters, QueryResult, Scope, Projector,
        PrincipalBinding, PrincipalMapping, Principal, PrincipalIdentity, PrincipalIdentityBinding, ScopeAspect, ScopeField, ScopeValue, ScopeWrite, ScopeUnit, PrincipalSource,
        QueryAuthorization, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField,
        RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization,
    > super::WorthQueryInstalledTemporalOperation<
        WorthQueryInstalledTemporalConditionalOperation<
            Schema, ApplicationOperation, Input, D, O, F, Node, Provider,
            Clock, Source, Query, Parameters, QueryResult, Scope, Projector,
        >,
        super::super::reconstruction_authority::WorthQueryTemporalReconstructionAccess<
            Schema, PrincipalBinding, PrincipalMapping, Principal, PrincipalIdentity, PrincipalIdentityBinding, Scope,
            ScopeAspect, ScopeField, ScopeValue, ScopeWrite, ScopeUnit, PrincipalSource, QueryAuthorization,
        >,
        super::super::operation_invocation::WorthQueryTemporalOperationExecution<
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
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding: worth_query_installation::facade::ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity> + 'static,
    ScopeAspect: 'static,
    ScopeField: 'static,
    ScopeField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = ScopeValue>,
    ScopeField::Binding: worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeValue: Clone + Send + Sync + 'static,
    ScopeWrite: worth_query_installation::facade::WritePosture + 'static,
    ScopeUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    PrincipalSource: super::super::reconstruction_authority::WorthQueryTemporalPrincipalSource<Schema>,
    QueryAuthorization: super::super::WorthQueryTemporalQueryAuthorization<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>,
    Invoker: super::super::operation_invocation::WorthQueryTemporalOperationInvoker<Schema, ApplicationOperation, Input, Scope>,
    IntentEntity: 'static,
    IdentityAspect: 'static,
    IdentityField: worth_query_installation::facade::OperationReads<ApplicationOperation> + 'static,
    IdentityField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    IdentityValue: Clone + Send + 'static,
    IdentityWrite: worth_query_installation::facade::WritePosture + 'static,
    IdentityUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    RevisionAspect: 'static,
    RevisionField: worth_query_installation::facade::OperationReads<ApplicationOperation> + worth_query_installation::facade::OperationWrites<ApplicationOperation> + 'static,
    RevisionField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = RevisionValue>,
    RevisionField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<Value = RevisionValue> + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone + Send + 'static,
    RevisionWrite: worth_query_installation::facade::WritableCapability + 'static,
    RevisionEquality: 'static,
    RevisionUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    LifecycleAspect: 'static,
    LifecycleField: worth_query_installation::facade::OperationReads<ApplicationOperation> + worth_query_installation::facade::OperationWrites<ApplicationOperation> + 'static,
    LifecycleField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = LifecycleValue>,
    LifecycleField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<Value = LifecycleValue>,
    LifecycleValue: Clone + Send + Sync + 'static,
    LifecycleWrite: worth_query_installation::facade::WritableCapability + 'static,
    LifecycleEquality: 'static,
    LifecycleUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    Authorization: super::super::WorthQueryTemporalOperationAuthorization<Schema, ApplicationOperation, Input, Scope> + 'static,
{
    pub(super) fn select_product_evaluation_binding(
        &mut self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &super::super::signal_decision_reentry::WorthQueryConditionalTruthBasis,
    ) -> Result<(), super::super::installation::WorthQueryConditionalRuntimeInstallationDenial> {
        let selected_identity = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            truth.product().observation(),
        );
        if self
            .active_affinity
            .as_ref()
            .is_some_and(|active| active.identity() == &selected_identity)
        {
            return Ok(());
        }
        let active_identity = self
            .active_affinity
            .as_ref()
            .map(|active| active.identity().clone());
        if let Some(mut selected) = self.inactive_bindings.remove(&selected_identity) {
            if let Some(active_identity) = active_identity {
                self.swap_evaluation_binding(&mut selected);
                self.inactive_bindings.insert(active_identity, selected);
            } else {
                self.activate_first_evaluation_binding(selected);
            }
            return Ok(());
        }

        let lowering_anchor = Arc::clone(&self.bootstrap_lowering);
        let exact = bridge
            .admit_exact_conditional_signal_basis(&lowering_anchor, truth.signal_basis())
            .map_err(|denial| {
                super::super::installation::WorthQueryConditionalRuntimeInstallationDenial::new(
                    super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                    format!("{:?}: {}", denial.kind(), denial.detail()),
                )
            })?;
        let lowering = exact.installed_lowering();
        let predecessor = self.predecessor_binding_state(&selected_identity);
        let mut selected = self.create_evaluation_binding(
            bridge,
            runtime,
            truth,
            exact,
            &lowering,
            predecessor.as_ref(),
        )?;
        if let Some(active_identity) = active_identity {
            self.swap_evaluation_binding(&mut selected);
            self.inactive_bindings.insert(active_identity, selected);
        } else {
            self.activate_first_evaluation_binding(selected);
        }
        Ok(())
    }

    fn create_evaluation_binding(
        &mut self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &super::super::signal_decision_reentry::WorthQueryConditionalTruthBasis,
        exact: BridgeConditionalSignalBasisBinding,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
        predecessor: Option<&WorthQueryPredecessorEvaluationState>,
    ) -> Result<WorthQueryInactiveTemporalEvaluationBinding<Clock, Input>, super::super::installation::WorthQueryConditionalRuntimeInstallationDenial> {
        let evaluation_ordinal = self.next_evaluation_binding_ordinal;
        self.next_evaluation_binding_ordinal = evaluation_ordinal.checked_add(1).ok_or_else(|| {
            super::super::installation::WorthQueryConditionalRuntimeInstallationDenial::new(
                super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::RetentionIdentityExhausted,
                "conditional evaluation binding identity space is exhausted",
            )
        })?;
        let clock = self.definition.binding.clocked_node();
        let bounds = self.definition.binding.bounds();
        let managed_clock = bridge
            .install_managed_clock(worth_runtime_bridge::facade::BridgeManagedClockInstallationParts {
                lowering,
                binding_identity: Arc::from(format!("{}:evaluation={evaluation_ordinal}", self.runtime_binding_identity)),
                source_identity: Arc::from(clock.source_identity().as_str()),
                timeline_identity: Arc::from(clock.timeline_identity().as_str()),
                maximum_active_intents: bounds.maximum_reconstruction_rows(),
                maximum_due_wakes_per_observation: bounds.maximum_due_wakes_per_observation(),
            })
            .map_err(|denial| {
                super::super::installation::WorthQueryConditionalRuntimeInstallationDenial::new(
                    super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                    denial.detail(),
                )
            })?;
        let reconstruction = super::super::temporal_reconstruction::reconstruct_temporal_intents(
            runtime,
            &self.definition.binding,
            &self.definition.reconstruction,
            self.definition.execution.identity_field,
            truth.product(),
        )?;
        let mut intents = reconstruction.intents;
        super::super::temporal_reconstruction::reconcile_temporal_intents(
            bridge,
            &managed_clock,
            &mut intents,
        )?;
        let maximum_watch_records = bridge
            .conditional_evaluation_budget()
            .maximum_retained_slots
            .checked_mul(bounds.maximum_reconstruction_rows())
            .ok_or_else(|| {
                super::super::installation::WorthQueryConditionalRuntimeInstallationDenial::new(
                    super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::RetentionCapacityExhausted,
                    "conditional commit watch capacity overflowed",
                )
            })?;
        let commit_watch = super::commit_watch::WorthQueryConditionalCommitWatchSet::successor(
            predecessor.map(|state| &state.commit_watch),
            &intents,
            lowering,
            maximum_watch_records,
        )
        .map_err(|detail| {
            super::super::installation::WorthQueryConditionalRuntimeInstallationDenial::new(
                super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::RetentionCapacityExhausted,
                detail,
            )
        })?;
        Ok(WorthQueryInactiveTemporalEvaluationBinding {
            affinity: super::evaluation_affinity::WorthQueryConditionalEvaluationAffinity::new(
                truth.product(),
                exact,
            ),
            managed_clock,
            retained_wakes: Vec::new(),
            pending_direct_delivery: super::direct_delivery::empty(),
            reconstructed_intents: intents,
            reconstruction_work: reconstruction.work,
            authoritative_commit_cursor: predecessor
                .map(|state| state.cursor)
                .unwrap_or(self.authoritative_commit_cursor),
            commit_watch,
        })
    }

    fn predecessor_binding_state(
        &self,
        selected: &crate::basis::WorthQueryProductBranchReadIdentity,
    ) -> Option<WorthQueryPredecessorEvaluationState> {
        let active = self.active_affinity.as_ref().and_then(|affinity| {
            affinity
                .is_predecessor_of(selected)
                .then_some((
                    affinity.identity(),
                    WorthQueryPredecessorEvaluationState {
                        cursor: self.authoritative_commit_cursor,
                        commit_watch: self.commit_watch.clone(),
                    },
                ))
        });
        self.inactive_bindings
            .values()
            .filter(|binding| binding.affinity.is_predecessor_of(selected))
            .map(|binding| {
                (
                    binding.affinity.identity(),
                    WorthQueryPredecessorEvaluationState {
                        cursor: binding.authoritative_commit_cursor,
                        commit_watch: binding.commit_watch.clone(),
                    },
                )
            })
            .chain(active)
            .max_by(|left, right| left.0.cmp(right.0))
            .map(|(_, state)| state)
    }
}

impl<Binding, Reconstruction, Execution, Clock, Input>
    super::WorthQueryInstalledTemporalOperation<Binding, Reconstruction, Execution, Clock, Input>
{
    pub(super) fn active_lowering(&self) -> &Arc<BridgeInstalledConditionalLowering> {
        self.active_affinity
            .as_ref()
            .map(|affinity| affinity.signal_basis().installed_lowering_ref())
            .unwrap_or(&self.bootstrap_lowering)
    }

    pub(super) fn swap_evaluation_binding(
        &mut self,
        binding: &mut WorthQueryInactiveTemporalEvaluationBinding<Clock, Input>,
    ) {
        let active = self
            .active_affinity
            .as_mut()
            .expect("only a selected evaluation binding can be cached");
        std::mem::swap(active, &mut binding.affinity);
        std::mem::swap(
            self.managed_clock
                .as_mut()
                .expect("selected evaluation binding retains a managed clock"),
            &mut binding.managed_clock,
        );
        std::mem::swap(&mut self.retained_wakes, &mut binding.retained_wakes);
        std::mem::swap(
            &mut self.pending_direct_delivery,
            &mut binding.pending_direct_delivery,
        );
        std::mem::swap(
            &mut self.reconstructed_intents,
            &mut binding.reconstructed_intents,
        );
        std::mem::swap(
            &mut self.reconstruction_work,
            &mut binding.reconstruction_work,
        );
        std::mem::swap(
            &mut self.authoritative_commit_cursor,
            &mut binding.authoritative_commit_cursor,
        );
        std::mem::swap(&mut self.commit_watch, &mut binding.commit_watch);
    }

    pub(super) fn activate_first_evaluation_binding(
        &mut self,
        binding: WorthQueryInactiveTemporalEvaluationBinding<Clock, Input>,
    ) {
        let WorthQueryInactiveTemporalEvaluationBinding {
            affinity,
            managed_clock,
            retained_wakes,
            pending_direct_delivery,
            reconstructed_intents,
            reconstruction_work,
            authoritative_commit_cursor,
            commit_watch,
        } = binding;
        self.active_affinity = Some(affinity);
        self.managed_clock = Some(managed_clock);
        self.retained_wakes = retained_wakes;
        self.pending_direct_delivery = pending_direct_delivery;
        self.reconstructed_intents = reconstructed_intents;
        self.reconstruction_work = reconstruction_work;
        self.authoritative_commit_cursor = authoritative_commit_cursor;
        self.commit_watch = commit_watch;
    }
}

struct WorthQueryPredecessorEvaluationState {
    cursor: u64,
    commit_watch: super::commit_watch::WorthQueryConditionalCommitWatchSet,
}
