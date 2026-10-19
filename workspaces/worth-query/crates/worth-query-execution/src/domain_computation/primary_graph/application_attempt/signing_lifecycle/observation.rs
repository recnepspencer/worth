//! Bounded current membership and immutable footprint observation retain native revisions.
use super::super::{
    fact::{observe_adjacency_checked, AdjacencyObservationDenial},
    WorthQueryApplicationAdjacencyDirection as Direction,
};
use super::{
    attempt, capacity, refused, resource, unavailable, EntityId, Fact, Kind, KindId, Lease,
    Outcome, QueryMemoryReservation, QueryRequestExecution, Rule,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationSigningSubjectProvenance as Provenance,
    ApplicationSigningSubjectProvenanceBinding as Binding,
};
use worth_relational::facade::{
    runtime::{
        AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection as NativeDirection,
        RelationalRuntime,
    },
    snapshots::SnapshotHandle,
};

pub(super) fn adjacency(
    runtime: &RelationalRuntime,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    anchor: EntityId,
    relation: KindId,
    maximum: usize,
    facts: &mut Vec<Fact>,
) -> Result<Vec<super::super::fact::WorthQueryApplicationObservedRelation>, Outcome> {
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(unavailable)?;
    let native_revision = view
        .bounded_adjacency_structural_revision(
            request,
            anchor,
            relation,
            NativeDirection::Outgoing,
            1,
        )
        .map_err(|cause| match cause {
            AdjacencyStructuralRevisionDenial::Execution(cause) => attempt(
                super::super::WorthQueryApplicationAttemptDenial::from(cause),
            ),
            _ => unavailable(),
        })?
        .revision();
    let edges = observe_adjacency_checked(
        runtime,
        request,
        snapshot,
        relation,
        anchor,
        Direction::Outgoing,
        maximum.checked_mul(2).ok_or_else(capacity)?,
    )
    .map_err(|cause| match cause {
        AdjacencyObservationDenial::Execution(cause) => attempt(
            super::super::WorthQueryApplicationAttemptDenial::from(cause),
        ),
        AdjacencyObservationDenial::WorkBudgetExceeded => {
            refused(Kind::SigningLifecycleEvidenceCapacity {
                maximum_evidence: maximum,
            })
        }
        AdjacencyObservationDenial::Interrupted(event) => {
            attempt(super::super::WorthQueryApplicationAttemptDenial::new(
                match event.interruption() {
                    worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
                        super::super::WorthQueryApplicationAttemptDenialKind::Cancelled
                    }
                    worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => {
                        super::super::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
                    }
                },
                "signing lifecycle history",
            ))
        }
        AdjacencyObservationDenial::SnapshotUnavailable => unavailable(),
    })?;
    if edges.len() > maximum {
        return Err(refused(Kind::SigningLifecycleEvidenceCapacity {
            maximum_evidence: maximum,
        }));
    }
    facts.push(Fact::SourceAdjacencyRevision { relation_kind: relation, anchor, direction: NativeDirection::Outgoing,
        native_revision, comparison_work_limit: 1,
        endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty() });
    Ok(edges)
}

pub(super) fn footprint(
    rule: &Rule,
    evidence: EntityId,
    lease: &Lease,
    runtime: &RelationalRuntime,
    execution: &QueryRequestExecution<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    facts: &mut Vec<Fact>,
) -> Result<(Provenance, QueryMemoryReservation), Outcome> {
    let field = rule.provenance();
    let locator = lease
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .ok_or_else(unavailable)?;
    let kind = lease
        .layout
        .entity_kind(rule.evidence_entity())
        .ok_or_else(unavailable)?;
    let observed = super::super::observation::observe_field_value_borrowed(
        runtime,
        request,
        lease.snapshot(),
        evidence,
        kind,
        locator,
        |value| {
            let value = value.ok_or_else(unavailable)?;
            let bytes = value
                .owned_allocation_capacity_bytes()
                .checked_mul(8)
                .ok_or_else(capacity)?;
            let memory = execution
                .reserve(u64::try_from(bytes).map_err(|_| capacity())?)
                .map_err(resource)?;
            let value = Binding::decode(value).map_err(|_| unavailable())?;
            Ok::<_, Outcome>((value, memory))
        },
    )
    .map_err(|cause| {
        attempt(super::super::WorthQueryApplicationAttemptDenial::from(
            cause,
        ))
    })?
    .ok_or_else(unavailable)??;
    let locator = worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    );
    let native_revision = runtime
        .read_truth()
        .project_snapshot(lease.snapshot())
        .ok_or_else(unavailable)?
        .entity_field_revision(request, evidence, &locator)
        .map_err(|cause| {
            attempt(super::super::WorthQueryApplicationAttemptDenial::from(
                cause,
            ))
        })?;
    if native_revision.is_none() {
        return Err(unavailable());
    }
    facts.push(Fact::SourceEntity {
        entity_id: evidence,
    });
    facts.push(Fact::SourceFieldRevision {
        entity_id: evidence,
        locator,
        native_revision,
    });
    Ok(observed)
}
