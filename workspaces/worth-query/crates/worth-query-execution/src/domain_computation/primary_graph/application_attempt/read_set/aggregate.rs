//! Seal consumes the original issued aggregate; a descriptive selector is insufficient.
#[cfg(test)]
mod tests;
use super::super::{
    fact::WorthQueryApplicationObservedFact as Fact, read_scope::WorthQueryApplicationReadScope,
    snapshot_lease::WorthQueryApplicationSnapshotLease,
};
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::{AggregateKey, AggregateObservation},
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
    key: &AggregateKey,
    position: Option<InstalledScopePosition>,
) -> Result<(InstalledScopePosition, Fact), WorthQueryApplicationAttemptDenial> {
    let mismatch = || {
        denial(
            WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
            key.relation(),
        )
    };
    let body = expected
        .aggregate(
            key,
            admission.admission_identity(),
            &admission.retain_graph_contract(),
            lease.snapshot(),
        )
        .ok_or_else(mismatch)?;
    let descriptor = super::super::WorthQueryApplicationFactKey::Aggregate(key.clone());
    let position = position
        .or_else(|| {
            admission
                .allowed_graph_contract()
                .graph_reads()
                .roles()
                .iter()
                .flat_map(|role| role.read_scopes())
                .find(|s| {
                    super::observation_admission::graph_read_scope_matches_key(s, &descriptor)
                })
                .and_then(|s| {
                    InstalledScopePosition::from_installed(admission.allowed_graph_contract(), s)
                })
        })
        .ok_or_else(mismatch)?;
    if !position
        .resolve(admission.allowed_graph_contract())
        .is_some_and(|s| super::observation_admission::graph_read_scope_matches_key(s, &descriptor))
    {
        return Err(mismatch());
    }
    let layout = lease.layout.relation(key.relation()).ok_or_else(mismatch)?;
    if layout.kind != key.relation_kind()
        || layout.from != key.entity_kind()
        || layout.to != key.entity_kind()
        || !layout.orbit
        || body.native().definition() != key.definition()
    {
        return Err(mismatch());
    }
    let result = execution.run_native_with_checkpoint(|context| {
        for member in body.native().members() {
            AggregateObservation::checkpoint(context).map_err(|cause| {
                WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, key.relation())
            })?;
            if !scope.admits(member) {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                    key.relation(),
                ));
            }
        }
        Ok((
            position,
            Fact::Aggregate {
                observation: std::sync::Arc::clone(body),
            },
        ))
    });
    match result {
        Ok((value, _)) => value,
        Err(stop) => Err(WorthQueryApplicationAttemptDenial::relational_read_stopped(
            stop.into(),
            key.relation(),
        )),
    }
}
