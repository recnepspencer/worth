//! Actual named-derived acquisition retains complete support once, even when empty.
use super::super::derived_source::{DerivedSourceObservation, DerivedSourceQuery};
use super::super::read_observation::WorthQueryInvariantProjectionWork;
use super::*;
use std::cell::RefCell;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
pub(super) fn advance<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
    located: PathStep<'_>,
    frontier: &mut Frontier,
    ports: &mut RootPathReadPorts<'_, E>,
    selected: &mut RootPathSelectionBuilder,
) -> Result<Frontier, RootPathReadDenial<E>> {
    let subject = located.step.relation();
    let layout = basis
        .graph
        .derived_relation(subject)
        .filter(|layout| {
            layout.from.entity() == Some(located.step.from())
                && layout.to.entity() == Some(located.step.to())
                && layout.carriers.is_empty()
        })
        .ok_or_else(|| RootPathReadDenial::traversal(subject))?;
    selected
        .activate_source_allocation(basis.execution, frontier)
        .map_err(|cause| RootPathReadDenial::source(subject, cause))?;
    let checkpoint = ports.checkpoint;
    let first_stop = RefCell::new(None);
    let control = || match checkpoint(subject) {
        Ok(()) => Ok(()),
        Err(stop) => {
            if first_stop.borrow().is_none() {
                *first_stop.borrow_mut() = Some(stop);
            }
            Err(Cause::NestedStopped)
        }
    };
    let result = basis.execution.run_native_with_checkpoint(|context| {
        let mut bodies = Vec::new();
        if let Some(allocation) = &selected.allocation {
            allocation
                .claim_slice::<Arc<DerivedSourceObservation>>(basis.execution, frontier.len())
                .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
        }
        bodies
            .try_reserve_exact(frontier.len())
            .map_err(|_| RootPathReadDenial::native(subject, Cause::AllocationFailed))?;
        let mut next = BTreeMap::new();
        let mut step_reads = WorthQueryInvariantProjectionWork::default();
        for (root, mut predecessor) in std::mem::take(frontier) {
            ports.check(subject)?;
            let observed = DerivedSourceObservation::read(
                basis.runtime,
                basis.execution,
                DerivedSourceQuery {
                    relation: subject,
                    definition: &layout.handle,
                    root,
                    direction: located.direction,
                    control: Some(&control),
                },
                context,
            );
            let (reads, units) = match &observed {
                Ok(body) => (body.native().data().reads(), body.native().work_units()),
                Err(stop) => (stop.reads, stop.work),
            };
            // Counters enter the concrete caller before any adapter or Native
            // refusal can discard a partial acquisition or completed body.
            let charged = ports.notify(RootPathReadObservation::Derived {
                reads,
                units,
                subject,
            });
            if first_stop.borrow().is_some() {
                return Err(RootPathReadDenial::native(subject, Cause::NestedStopped));
            }
            charged?;
            let body = observed.map_err(|stop| RootPathReadDenial::Derived {
                subject: subject.into(),
                kind: stop.kind,
            })?;
            step_reads.record_adjacencies(reads.adjacency_lists, reads.relation_records);
            step_reads.record_endpoints(reads.entity_records);
            predecessor
                .record_derived(&body, basis.execution, context, Some(&control))
                .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
            for member in body.native().data().endpoints() {
                ports.check(subject)?;
                DerivedSourceObservation::checkpoint(context)
                    .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
                let entity = member
                    .entity()
                    .ok_or_else(|| RootPathReadDenial::traversal(subject))?;
                if next.contains_key(&entity) {
                    continue;
                }
                selected
                    .admit_frontier_entry(basis.execution, next.len())
                    .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
                ports.notify(RootPathReadObservation::SourceCopy {
                    units: predecessor.source_count() + 1,
                    subject,
                })?;
                let mut source = predecessor
                    .copy_for_step(basis.execution)
                    .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
                source
                    .observe_entity(basis.execution, entity)
                    .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
                next.insert(entity, source);
            }
            bodies.push(body);
        }
        if let Some(source) = &mut selected.set_source {
            source
                .record_derived_batch(&bodies, basis.execution, context, Some(&control))
                .map_err(|cause| RootPathReadDenial::native(subject, cause))?;
            ports.notify(RootPathReadObservation::SourceCopy {
                units: bodies.len(),
                subject,
            })?;
        }
        ports.notify(located.completed(step_reads))?;
        ports.check(subject)?;
        Ok(next)
    });
    let result = match result {
        Ok((value, _)) => value,
        Err(stop) => Err(RootPathReadDenial::native(subject, stop.into())),
    };
    if let Some(stop) = first_stop.borrow_mut().take() {
        return Err(RootPathReadDenial::Adapter(stop));
    }
    ports.check(subject)?;
    result
}
