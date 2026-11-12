use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, WorthQueryOperationGraphReadContract,
    WorthQueryOperationGraphReadScope,
};

use super::super::fact::WorthQueryApplicationFactKey;
use super::WorthQueryApplicationReadAttempt;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEntityIdentity,
};

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'_, '_, Schema, Operation, Input, Scope, Phase>
{
    pub(super) fn admit_target(
        &self,
        target: &ApplicationOperationDecisionReadTarget,
    ) -> Result<super::InstalledScopePosition, WorthQueryApplicationAttemptDenial> {
        crate::domain_computation::application_contract_admission::installed_read_scope_for_target(
            self.admission.allowed_graph_contract().graph_reads(),
            target,
        )
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

    pub(super) fn validate_identity_authority<Entity>(
        &self,
        entity: &str,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if identity.runtime_authority() != self.admission.runtime_authority()
            || identity.binding_identity() != self.admission.binding_identity()
            || identity.entity_name() != entity
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::StaleEntityIdentity,
                entity,
            ));
        }
        if !self.read_scope.admits(identity.entity_id()) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                entity,
            ));
        }
        Ok(())
    }

    pub(super) fn validate_identity_freshness<Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.lease
            .handle()
            .with_runtime(execution, |runtime| {
                self.entity_resolution
                    .at_snapshot(
                        runtime,
                        self.lease.snapshot(),
                        super::super::super::WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .and_then(|truth| truth.validate_entity_freshness(execution, identity))
            })
            .map_err(|refusal| {
                match refusal.kind() {
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Stopped(cause) => WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, entity),
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(_) => denial(WorthQueryApplicationAttemptDenialKind::StaleEntityIdentity, entity),
                }
                })
    }

    pub(super) fn field_layout(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<worth_foundational::facade::AspectFieldLocator, WorthQueryApplicationAttemptDenial>
    {
        self.layout
            .field_locator(entity, aspect, field)
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    field,
                )
            })
    }
}

pub(super) fn graph_read_scope_matches_key(
    scope: &WorthQueryOperationGraphReadScope,
    key: &WorthQueryApplicationFactKey,
) -> bool {
    match (scope, key) {
        (
            WorthQueryOperationGraphReadScope::DerivedSource(scope),
            WorthQueryApplicationFactKey::ComponentSelection(key),
        ) => key.matches_derived_scope(scope),
        (
            WorthQueryOperationGraphReadScope::Components(scope),
            WorthQueryApplicationFactKey::ComponentSelection(key),
        ) => key.matches_scope(scope),
        (
            WorthQueryOperationGraphReadScope::Relation(scope),
            WorthQueryApplicationFactKey::Aggregate(key),
        ) => key.orbit_relation().is_some() && scope.relation() == key.subject(),
        (
            WorthQueryOperationGraphReadScope::Relation(scope),
            WorthQueryApplicationFactKey::StructuralWalk(key),
        ) => scope.relation() == key.relation(),
        (
            WorthQueryOperationGraphReadScope::Entity(scope),
            WorthQueryApplicationFactKey::Entity { entity, .. },
        ) => scope.semantic_key() == entity,
        (
            WorthQueryOperationGraphReadScope::NativeProjection(scope),
            WorthQueryApplicationFactKey::Field {
                entity, locator, ..
            }
            | WorthQueryApplicationFactKey::AspectPresence {
                entity, locator, ..
            },
        ) => {
            let [field] = locator.field_path().fields() else {
                return false;
            };
            scope.entity().semantic_key() == entity
                && scope.aspect() == locator.aspect().aspect_key()
                && scope.admits_field(field)
        }
        (
            WorthQueryOperationGraphReadScope::Relation(scope),
            WorthQueryApplicationFactKey::Relation { relation, .. }
            | WorthQueryApplicationFactKey::Adjacency { relation, .. },
        ) => scope.relation() == relation,
        _ => false,
    }
}

pub(super) fn graph_reads_exactly_cover_fact_keys<'key>(
    contract: &WorthQueryOperationGraphReadContract,
    mut keys: impl Clone + Iterator<Item = &'key WorthQueryApplicationFactKey>,
) -> bool {
    for scope in contract.roles().iter().flat_map(|role| role.read_scopes()) {
        let actual = keys
            .clone()
            .filter(|key| !matches!(key, WorthQueryApplicationFactKey::AspectPresence { .. }))
            .filter(|key| graph_read_scope_matches_key(scope, key))
            .count();
        let covered = match scope {
            WorthQueryOperationGraphReadScope::Components(_)
            | WorthQueryOperationGraphReadScope::DerivedSource(_) => actual > 0,
            WorthQueryOperationGraphReadScope::Entity(_)
            | WorthQueryOperationGraphReadScope::Relation(_) => actual == 1,
            WorthQueryOperationGraphReadScope::NativeProjection(scope) => scope
                .atomic_field_count()
                .is_some_and(|expected| actual == expected),
        };
        if !covered {
            return false;
        }
    }
    // Overlapping shared/inline declarations form a union. Each observed key
    // still needs genuine installed coverage, and each declared leaf is complete.
    keys.all(|key| {
        contract
            .roles()
            .iter()
            .flat_map(|role| role.read_scopes())
            .any(|scope| graph_read_scope_matches_key(scope, key))
    })
}
fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
