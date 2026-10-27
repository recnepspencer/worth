//! Complete Native closure retains the actual body and predecessor provenance.
use super::super::{
    read_observation::WorthQueryInvariantProjectionWork,
    schema_layout::WorthQueryPrimaryRelationLayout,
    structural_walk::{
        memory, StructuralWalkDependencies, StructuralWalkObservation, StructuralWalkQuery,
    },
};
use super::*;
use std::{cell::RefCell, sync::Arc};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
pub(super) fn advance<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
    layout: &WorthQueryPrimaryRelationLayout,
    located: PathStep<'_>,
    frontier: &Frontier,
    ports: &mut RootPathReadPorts<'_, E>,
    set_source: &mut Option<SourceFactsBuilder>,
) -> Result<Frontier, RootPathReadDenial<E>> {
    let subject = located.step.relation();
    if layout.from != layout.to {
        return Err(RootPathReadDenial::traversal(subject));
    }
    let checkpoint = ports.checkpoint;
    let original_stop = RefCell::new(None);
    let control = || match checkpoint(subject) {
        Ok(()) => Ok(()),
        Err(stop) => {
            if original_stop.borrow().is_none() {
                *original_stop.borrow_mut() = Some(stop);
            }
            Err(Cause::NestedStopped)
        }
    };
    let result = basis.execution.run_native_with_checkpoint(|context| {
        let bytes = frontier
            .len()
            .checked_mul(size_of::<EntityId>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| RootPathReadDenial::native(subject, Cause::ChargedBytesOverflow))?;
        let _roots_memory = memory::reserve(basis.execution, context.has_accounting(), bytes)
            .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(frontier.len())
            .map_err(|_| RootPathReadDenial::native(subject, Cause::AllocationFailed))?;
        for root in frontier.keys() {
            ports.check(subject)?;
            StructuralWalkObservation::checkpoint(context)
                .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
            roots.push(*root);
        }
        let observed = StructuralWalkObservation::read(
            basis.runtime,
            basis.execution,
            StructuralWalkQuery {
                relation: subject,
                roots: &roots,
                entity_kind: layout.from,
                relation_kind: layout.kind,
                direction: located.direction,
                maximum_work: None,
                control: Some(&control),
            },
            context,
        );
        let (reads, units) = match &observed {
            Ok(body) => (body.native().reads(), body.native().work_units()),
            Err(stop) => (stop.reads, stop.work),
        };
        // Acquired Native work is reported before either the original caller's
        // stop or any later adapter/physical denial can return.
        let charged = ports.notify(RootPathReadObservation::Walk {
            reads,
            units,
            subject,
        });
        if let Some(stop) = original_stop.borrow_mut().take() {
            return Err(RootPathReadDenial::Adapter(stop));
        }
        charged?;
        let body = observed.map_err(|stop| RootPathReadDenial::Walk {
            subject: subject.to_owned(),
            kind: stop.kind,
        })?;
        ports.check(subject)?;
        let mut step_reads = WorthQueryInvariantProjectionWork::default();
        step_reads.record_adjacencies(reads.adjacency_lists, reads.relation_records);
        step_reads.record_endpoints(reads.entity_records);
        ports.notify(located.completed(step_reads))?;
        let prior = frontier
            .first_key_value()
            .and_then(|(_, source)| source.walks());
        let dependencies =
            StructuralWalkDependencies::append(prior, &body, basis.execution, context)
                .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
        if let Some(source) = set_source {
            source
                .record_walk(&body, basis.execution, context)
                .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
            ports.notify(RootPathReadObservation::SourceCopy { units: 1, subject })?;
        }
        let mut next = BTreeMap::new();
        for visit in body.native().visits() {
            ports.check(subject)?;
            StructuralWalkObservation::checkpoint(context)
                .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
            let predecessor = frontier
                .get(&visit.root)
                .ok_or_else(|| RootPathReadDenial::traversal(subject))?;
            let same_prefix = match (prior, predecessor.walks()) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            };
            if !same_prefix {
                return Err(RootPathReadDenial::traversal(subject));
            }
            ports.notify(RootPathReadObservation::SourceCopy {
                units: predecessor.source_count() + 1,
                subject,
            })?;
            let mut source = predecessor.clone();
            source.observe_entity(visit.entity.entity_id);
            source.set_walks(Arc::clone(&dependencies));
            next.insert(visit.entity.entity_id, source);
        }
        ports.check(subject)?;
        Ok(next)
    });
    let result = match result {
        Ok((value, _)) => value,
        Err(stop) => Err(RootPathReadDenial::native(subject, stop.into())),
    };
    ports.check(subject)?;
    result
}
