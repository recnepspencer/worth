//! A selector seals only its original issued body and exact installed Components scope.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        fact::WorthQueryApplicationObservedFact as Fact,
        read_scope::WorthQueryApplicationReadScope,
        snapshot_lease::WorthQueryApplicationSnapshotLease,
    },
    component_selection::{ComponentSelectionKey, ComponentSelectionSubject},
    invariant_projection::ExpectedFactKeys,
};
pub(super) fn observe<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    lease: &WorthQueryApplicationSnapshotLease,
    scope: &WorthQueryApplicationReadScope,
    expected: &ExpectedFactKeys,
    key: &ComponentSelectionKey,
    position: Option<InstalledScopePosition>,
) -> Result<(InstalledScopePosition, Fact), WorthQueryApplicationAttemptDenial> {
    let mismatch = || {
        denial(
            WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
            "component selection",
        )
    };
    let body = expected
        .component(
            key,
            admission.admission_identity(),
            &admission.retain_graph_contract(),
            lease.snapshot(),
        )
        .ok_or_else(mismatch)?;
    let operation_binding = body.operation_binding().ok_or_else(mismatch)?;
    if key.root() != admission.scope_entity_id()
        || operation_binding != admission.binding_identity()
    {
        return Err(mismatch());
    }
    let descriptor = super::super::WorthQueryApplicationFactKey::ComponentSelection(key.clone());
    let position = position
        .or_else(|| {
            admission
                .allowed_graph_contract()
                .graph_reads()
                .roles()
                .iter()
                .flat_map(|role| role.read_scopes())
                .find(|installed| {
                    super::observation_admission::graph_read_scope_matches_key(
                        installed,
                        &descriptor,
                    )
                })
                .and_then(|installed| {
                    InstalledScopePosition::from_installed(
                        admission.allowed_graph_contract(),
                        installed,
                    )
                })
        })
        .ok_or_else(mismatch)?;
    if !position
        .resolve(admission.allowed_graph_contract())
        .is_some_and(|installed| body.installed_scope_matches(installed))
    {
        return Err(mismatch());
    }
    if !scope.admits(key.root())
        || match key.subject() {
            ComponentSelectionSubject::Of(member) => !scope.admits(member),
            ComponentSelectionSubject::DerivedOf(member) => !scope.admits(member.carrier()),
            ComponentSelectionSubject::Census => false,
        }
    {
        return Err(mismatch());
    }
    for facts in body.facts() {
        for entity in &facts.entities {
            execution.consult().map_err(|stop| {
                WorthQueryApplicationAttemptDenial::relational_read_stopped(
                    stop.into(),
                    "component selection",
                )
            })?;
            if !scope.admits(*entity) {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                    "component selection",
                ));
            }
        }
    }
    Ok((
        position,
        Fact::ComponentSelection {
            observation: std::sync::Arc::clone(body),
        },
    ))
}
