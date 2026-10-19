use worth_execution::ExecutionAllocationPolicy;
use worth_query_installation::facade::{
    ApplicationRelationRef, OperationReads, WorthQueryOperationGraphReadScope,
};

use super::super::fact::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};
use super::super::read_phase::WorthQueryProjectedApplicationMutation;
use super::{denial, WorthQueryApplicationReadAttempt, WorthQueryCompleteApplicationReadSet};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEntityIdentity,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationReadAttempt<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    /// Re-observes exactly the decision dependencies carried by the sealed
    /// operation projection. No caller can add, remove, or replace a fact key.
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
        let expected = self.expected_facts.as_ref().ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                self.admission.operation(),
            )
        })?;
        for key in expected {
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
            WorthQueryOperationGraphReadScope,
            WorthQueryApplicationObservedFact,
        ),
        WorthQueryApplicationAttemptDenial,
    > {
        observe_projection_dependency(
            execution,
            &self.admission,
            &self.lease,
            &self.read_scope,
            key,
        )
    }
}

pub(super) fn observe_projection_dependency<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    lease: &super::super::snapshot_lease::WorthQueryApplicationSnapshotLease,
    read_scope: &super::super::read_scope::WorthQueryApplicationReadScope,
    key: &WorthQueryApplicationFactKey,
) -> Result<
    (
        WorthQueryOperationGraphReadScope,
        WorthQueryApplicationObservedFact,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    if !key_entities_are_in_scope(key, read_scope) {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
            admission.operation(),
        ));
    }
    let read_scope = admission
        .allowed_graph_contract()
        .graph_reads()
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .find(|scope| super::observation_admission::graph_read_scope_matches_key(scope, key))
        .cloned()
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                admission.operation(),
            )
        })?;
    let fact = lease.handle().with_runtime(execution, |runtime| {
        super::fact_observation::observe_fact(
            execution,
            runtime,
            lease.snapshot(),
            &lease.layout,
            key,
        )
    })?;
    Ok((read_scope, fact))
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
        WorthQueryApplicationFactKey::Entity { entity_id, .. }
        | WorthQueryApplicationFactKey::AspectPresence { entity_id, .. }
        | WorthQueryApplicationFactKey::Field { entity_id, .. } => scope.admits(*entity_id),
        WorthQueryApplicationFactKey::Relation { from, to, .. } => {
            scope.admits(*from) && scope.admits(*to)
        }
        WorthQueryApplicationFactKey::Adjacency { anchor, .. } => scope.admits(*anchor),
    }
}
