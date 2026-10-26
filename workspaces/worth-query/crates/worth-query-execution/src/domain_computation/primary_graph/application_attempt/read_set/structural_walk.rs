//! Seal consumes the exact original issued value; it never replays a descriptor.
use super::super::{
    fact::WorthQueryApplicationObservedFact as Fact, read_scope::WorthQueryApplicationReadScope,
    snapshot_lease::WorthQueryApplicationSnapshotLease,
};
use super::*;
#[cfg(test)]
mod tests;
use crate::domain_computation::primary_graph::invariant_projection::{
    ExpectedFactKeys, StructuralWalkKey, StructuralWalkObservation,
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
    key: &StructuralWalkKey,
    position: Option<InstalledScopePosition>,
) -> Result<(InstalledScopePosition, Fact), WorthQueryApplicationAttemptDenial> {
    let mismatch = || {
        denial(
            WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
            key.relation(),
        )
    };
    let body = expected
        .structural_walk(
            key,
            admission.admission_identity(),
            &admission.retain_graph_contract(),
            lease.snapshot(),
        )
        .ok_or_else(mismatch)?;
    let descriptor = super::super::WorthQueryApplicationFactKey::StructuralWalk(key.clone());
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
    {
        return Err(mismatch());
    }
    let result = execution.run_native_with_checkpoint(|context| {
        for visit in body.native().visits() {
            StructuralWalkObservation::checkpoint(context).map_err(|cause| {
                WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, key.relation())
            })?;
            if !scope.admits(visit.entity.entity_id) {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                    key.relation(),
                ));
            }
        }
        Ok((
            position,
            Fact::StructuralWalk {
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
