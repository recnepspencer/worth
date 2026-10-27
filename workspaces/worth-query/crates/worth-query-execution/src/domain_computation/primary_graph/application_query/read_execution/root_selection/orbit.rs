//! A complete single orbit retains Native anchor-forward order without path unions.
use super::super::interruption::NativeReadInterruption;
use super::path_union::traversal_denial;
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::AggregateQuery,
    application_query::observed_source::DeclaredOrbitSelection,
    structural_walk::{StructuralWalkDependencies, StructuralWalkObservation, StructuralWalkQuery},
    WorthQueryPrimaryGraphLayout,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_query::RelationDirection;
use worth_relational::facade::{
    runtime::{PinnedStructuralWalkDenialKind as WalkDenial, RelationalPinnedRead},
    transactions::RelationalExecutionDenialCause as Cause,
};

pub(super) fn select(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPinnedRead,
    graph: &WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    work: &mut RootSelectionWork<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let meaning = plan
        .contract
        .orbit()
        .expect("the selected mode is an orbit");
    // Disclosure of identity, length and all ordered membership is one slot.
    // An omission refuses before any Native aggregate selection takes place.
    if !plan.governance.is_disclosed(&meaning.slot_key()) {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            meaning.output_name(),
        ));
    }
    let relation = graph
        .relation(meaning.relation())
        .ok_or_else(|| traversal_denial(meaning.relation()))?;
    if !relation.orbit || relation.from != relation.to {
        return Err(traversal_denial(meaning.relation()));
    }
    let subject = meaning.relation();
    let interruption = NativeReadInterruption::new(work.interruption, subject);
    let control = || interruption.checkpoint();
    let result = execution.run_native_with_checkpoint(|context| {
        work.checkpoint(subject)?;
        let aggregate = super::aggregate::read(
            execution, runtime, work,
            AggregateQuery {
                subject,
                entity_kind: relation.from,
                selection: crate::domain_computation::primary_graph::aggregate_observation::AggregateSelection::OrbitRelation(relation.kind),
                member: plan.root,
                maximum_work: None,
                control: None,
            },
            &interruption,
        )?;
        let walk = StructuralWalkObservation::read(
            runtime,
            execution,
            StructuralWalkQuery {
                relation: subject,
                roots: &[aggregate.native().anchor()],
                entity_kind: relation.from,
                relation_kind: relation.kind,
                direction: RelationDirection::Forward,
                maximum_work: None,
                control: Some(&control),
            },
            context,
        );
        let (reads, units) = match &walk {
            Ok(body) => (body.native().reads(), body.native().work_units()),
            Err(stop) => (stop.reads, stop.work),
        };
        let charged = work.charge_walk(reads, units, subject);
        if let Some(stop) = interruption.denial() {
            return Err(stop);
        }
        charged?;
        let walk = walk.map_err(|stop| match stop.kind {
            WalkDenial::Read(cause) => super::super::denial::relational_read_denial(cause, subject),
            WalkDenial::WorkLimitExceeded | WalkDenial::DistanceOverflow => read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ),
            _ => traversal_denial(subject),
        })?;
        if walk.native().visits().len() != aggregate.native().len()
            || walk.native().relations().len() != aggregate.native().len()
        {
            return Err(traversal_denial(subject));
        }
        let mut source = RootPathSourceBuilder::default();
        source.observe_entity(plan.root);
        source.set_walks(
            StructuralWalkDependencies::append(None, &walk, execution, context)
                .map_err(|cause| super::super::denial::relational_read_denial(cause, subject))?,
        );
        source.set_orbit(DeclaredOrbitSelection {
            meaning: Arc::clone(meaning),
            aggregate: Arc::clone(&aggregate),
            binding: plan.binding.clone(),
        });
        let source = source.finish()?;
        let count = walk.native().visits().len();
        let mut candidates = Vec::new();
        candidates.try_reserve_exact(count).map_err(|_| {
            super::super::denial::relational_read_denial(Cause::AllocationFailed, subject)
        })?;
        let mut sources = BTreeMap::new();
        for visit in walk.native().visits() {
            work.checkpoint(subject)?;
            StructuralWalkObservation::checkpoint(context)
                .map_err(|cause| super::super::denial::relational_read_denial(cause, subject))?;
            let entity = visit.entity.entity_id;
            if !aggregate.native().contains(entity) {
                return Err(traversal_denial(subject));
            }
            candidates.push(entity);
            sources.insert(entity, Arc::clone(&source));
        }
        work.checkpoint(subject)?;
        Ok(BoundedRootSelection {
            candidates,
            selected_predicate_source: None,
            root_path_source: Some(sources),
            result_set_source: Some(source),
            examined_candidates: count,
            predicate_work_units: 0,
            work_units: work.work_units,
            predicate_index_generation: None,
            reads: work.reads,
            root_path_steps: None,
        })
    });
    // The outer execution handoff can itself refuse a nested callback stop.
    // Keep the first typed Query denial alive across that handoff.
    if let Some(stop) = interruption.denial() {
        return Err(stop);
    }
    work.checkpoint(subject)?;
    match result {
        Ok((value, _)) => value,
        Err(stop) => Err(super::super::denial::relational_read_denial(
            stop.into(),
            subject,
        )),
    }
}
