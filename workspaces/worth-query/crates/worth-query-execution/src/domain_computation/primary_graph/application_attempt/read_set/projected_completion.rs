use worth_execution::ExecutionAllocationPolicy;
use worth_query_installation::facade::{ApplicationRelationRef, OperationReads};

use super::super::fact::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};
use super::super::read_phase::WorthQueryProjectedApplicationMutation;
use super::{denial, WorthQueryApplicationReadAttempt, WorthQueryCompleteApplicationReadSet};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEntityIdentity,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationReadAttempt<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    /// Completes exactly the sealed projection's dependencies at its pinned
    /// snapshot. A qualified native run carries unchanged facts under their
    /// original custody; other paths observe every dependency again. Callers
    /// cannot add, remove, or replace a dependency key.
    pub fn complete_projected_dependencies(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        let phase_request = match phase.request_for_source(self.lease.handle().relational_runtime_instance_id()) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "application-read-phase")),
        };
        let execution = &phase_request;
        self.complete_projected_dependencies_with_request(execution, allocation_policy)
    }

    pub(in crate::domain_computation) fn complete_projected_dependencies_with_request(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        self.check_completion_live()?;
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let expected = self.expected_facts.as_ref().ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                self.admission.operation(),
            )
        })?;
        let native = expected
            .attribution()
            .native_refresh(
                &self.computation_reads.1,
                &self.admission.retain_graph_contract(),
                self.admission.scope_entity_id(),
                self.lease.snapshot(),
                super::super::retained_decision_facts::StorageControl::new(
                    allocation_policy,
                    Some(self.admission.publication_request()),
                ),
            )
            .map_err(|cause| cause.into_attempt_denial(self.admission.operation()))?;
        if let Some(native) = native {
            for key in expected.refresh_keys() {
                self.admission
                    .validate_current_authority()
                    .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
                allocation_policy.check_live().map_err(|cause| {
                    WorthQueryApplicationAttemptDenial::allocation_denied(
                        self.admission.operation(),
                        cause,
                    )
                })?;
                let scope = native.position(key).ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                        self.admission.operation(),
                    )
                })?;
                let fact = self.observe_scoped_projected_fact(execution, key, scope)?;
                self.installed_read_scopes.insert(key.clone(), scope);
                self.facts.insert(key.clone(), fact);
            }
            self.native_completion = Some(native);
            return self.complete(allocation_policy);
        }
        for key in expected.iter() {
            self.admission
                .validate_current_authority()
                .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
            allocation_policy.check_live().map_err(|denial| {
                WorthQueryApplicationAttemptDenial::allocation_denied(
                    self.admission.operation(),
                    denial,
                )
            })?;
            let (read_scope, fact) = self.observe_projected_fact(execution, key)?;
            self.installed_read_scopes.insert(key.clone(), read_scope);
            self.facts.insert(key.clone(), fact);
        }
        self.complete(allocation_policy)
    }

    fn observe_projected_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &WorthQueryApplicationFactKey,
    ) -> Result<
        (
            super::InstalledScopePosition,
            WorthQueryApplicationObservedFact,
        ),
        WorthQueryApplicationAttemptDenial,
    > {
        if let WorthQueryApplicationFactKey::SourcePublication(read) = key {
            return self.lease.handle().with_runtime(execution, |_runtime| {
                if !read.admits(self.admission.admission_identity()) {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                        self.admission.operation(),
                    ));
                }
                // This exact native observation was issued by the same reader
                // at this pinned root and original admission. Publication
                // revalidates its native fact and Query authorization together;
                // no second query or copied permission is admitted here.
                Ok((
                    super::InstalledScopePosition::SourceQuery,
                    WorthQueryApplicationObservedFact::SourcePublication {
                        observation: std::sync::Arc::clone(self.expected_facts.as_ref()
                            .and_then(|expected|expected.source_publication(read))
                            .ok_or_else(||denial(WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,self.admission.operation()))?),
                    },
                ))
            });
        }
        if !key_entities_are_in_scope(key, &self.read_scope) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                self.admission.operation(),
            ));
        }
        let read_scope = self.projected_read_scope(key)?;
        Ok((
            read_scope,
            self.observe_scoped_projected_fact(execution, key, read_scope)?,
        ))
    }

    fn observe_scoped_projected_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &WorthQueryApplicationFactKey,
        scope: super::InstalledScopePosition,
    ) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
        if !key_entities_are_in_scope(key, &self.read_scope) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                self.admission.operation(),
            ));
        }
        if !scope
            .resolve(self.admission.allowed_graph_contract())
            .is_some_and(|scope| {
                super::observation_admission::graph_read_scope_matches_key(scope, key)
            })
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                self.admission.operation(),
            ));
        }
        let fact = self.lease.handle().with_runtime(execution, |runtime| {
            super::fact_observation::observe_fact(
                execution,
                runtime,
                self.lease.snapshot(),
                &self.layout,
                key,
            )
        })?;
        Ok(fact)
    }

    fn projected_read_scope(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> Result<super::InstalledScopePosition, WorthQueryApplicationAttemptDenial> {
        self.admission
            .allowed_graph_contract()
            .graph_reads()
            .roles()
            .iter()
            .flat_map(|role| role.read_scopes())
            .find(|scope| super::observation_admission::graph_read_scope_matches_key(scope, key))
            .and_then(|scope| {
                super::InstalledScopePosition::from_installed(
                    self.admission.allowed_graph_contract(),
                    scope,
                )
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    self.admission.operation(),
                )
            })
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub fn projected_relation<Relation, From, To>(
        &self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEntityIdentity<Schema, From>,
        to: &WorthQueryApplicationEntityIdentity<Schema, To>,
    ) -> Result<
        super::WorthQueryObservedApplicationRelation<Schema, Relation, From, To>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        let layout = self.lease.layout.relation(relation.name()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                relation.name(),
            )
        })?;
        if from.runtime_authority() != self.admission.runtime_authority()
            || to.runtime_authority() != self.admission.runtime_authority()
            || from.binding_identity() != self.admission.binding_identity()
            || to.binding_identity() != self.admission.binding_identity()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                relation.name(),
            ));
        }
        self.relation_observation(
            relation.name(),
            layout.kind,
            from.entity_id(),
            to.entity_id(),
        )
    }
}

fn key_entities_are_in_scope(
    key: &WorthQueryApplicationFactKey,
    scope: &super::super::read_scope::WorthQueryApplicationReadScope,
) -> bool {
    match key {
        WorthQueryApplicationFactKey::SourcePublication(_) => false,
        WorthQueryApplicationFactKey::Entity { entity_id, .. }
        | WorthQueryApplicationFactKey::AspectPresence { entity_id, .. }
        | WorthQueryApplicationFactKey::Field { entity_id, .. } => scope.admits(*entity_id),
        WorthQueryApplicationFactKey::Relation { from, to, .. } => {
            scope.admits(*from) && scope.admits(*to)
        }
        WorthQueryApplicationFactKey::Adjacency { anchor, .. } => scope.admits(*anchor),
    }
}
