//! One body per cohort, one full census body, and roots in original Native order.
use super::*;
use acquisition::AcquiredComponents;
pub(super) fn complete(
    mut selected: AcquiredComponents,
    meaning: &ApplicationQueryComponentMeaning,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    work: &mut RootSelectionWork<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    selected
        .cohorts
        .sort_by_key(|cohort| cohort.aggregate.native().identity());
    let count = selected
        .cohorts
        .iter()
        .try_fold(0usize, |count, cohort| {
            count.checked_add(cohort.aggregate.native().len())
        })
        .ok_or_else(|| path_union::traversal_denial(plan.name))?;
    if count > plan.maximum_result_count {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded,
            plan.name,
        ));
    }
    let is_census = meaning.selection() == ApplicationQueryComponentSelection::Census;
    let mut of_source = None;
    let meaning = Arc::new(meaning.clone());
    let mut roots = Vec::new();
    roots.try_reserve_exact(count).map_err(|_| {
        super::super::super::denial::relational_read_denial(Cause::AllocationFailed, plan.name)
    })?;
    let mut row_sources = BTreeMap::new();
    let mut all_bodies = Vec::new();
    let mut all_facts = Vec::new();
    all_facts.extend(selected.census);
    for cohort in selected.cohorts {
        work.checkpoint(plan.name)?;
        let mut facts = cohort.visibility;
        facts.push(cohort.coupling);
        if is_census {
            for fact in &facts {
                all_facts.push(fact.copy_for_observer(execution).map_err(|cause| {
                    super::super::super::denial::relational_read_denial(cause, plan.name)
                })?);
            }
        }
        let subject = if meaning.selection() == ApplicationQueryComponentSelection::OfScopeMember {
            ComponentSelectionSubject::Of(plan.root)
        } else {
            ComponentSelectionSubject::Of(cohort.aggregate.native().anchor())
        };
        let body = make_body(
            execution,
            graph,
            plan,
            subject,
            vec![Arc::clone(&cohort.aggregate)],
            facts,
            work,
        )?;
        let source = make_source(execution, plan, &meaning, body)?;
        if !is_census {
            of_source = Some(Arc::clone(&source));
        }
        for member in cohort.aggregate.native().members() {
            work.checkpoint(plan.name)?;
            roots.push(member);
            row_sources.insert(member, Arc::clone(&source));
        }
        all_bodies.push(cohort.aggregate);
    }
    let subject = if meaning.selection() == ApplicationQueryComponentSelection::OfScopeMember {
        ComponentSelectionSubject::Of(plan.root)
    } else {
        ComponentSelectionSubject::Census
    };
    let result_set_source = if is_census {
        let full = make_body(execution, graph, plan, subject, all_bodies, all_facts, work)?;
        Some(make_source(execution, plan, &meaning, full)?)
    } else {
        of_source
    };
    Ok(BoundedRootSelection {
        candidates: roots,
        selected_predicate_source: None,
        root_path_source: Some(row_sources),
        result_set_source,
        examined_candidates: work.predicate_records_examined,
        predicate_work_units: work.predicate_work_units,
        work_units: work.work_units,
        predicate_index_generation: None,
        reads: work.reads,
        root_path_steps: work.take_steps(),
    })
}
fn make_body(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    subject: ComponentSelectionSubject,
    aggregates: Vec<Arc<AggregateObservation>>,
    facts: Vec<SourceFacts>,
    work: &mut RootSelectionWork<'_>,
) -> Result<Arc<ComponentSelection>, WorthQueryApplicationReadExecutionDenial> {
    let interruption = NativeReadInterruption::new(work.interruption, plan.name);
    let result = execution.run_native_with_checkpoint(|context| {
        ComponentSelection::from_query(
            plan.contract,
            plan.binding,
            graph,
            plan.root,
            plan.snapshot.clone(),
            subject,
            aggregates,
            facts,
            || {
                AggregateObservation::checkpoint(context)?;
                interruption.checkpoint()
            },
        )
    });
    if let Some(stop) = interruption.denial() {
        return Err(stop);
    }
    work.checkpoint(plan.name)?;
    let body = match result {
        Ok((value, _)) => value,
        Err(stop) => Err(stop.into()),
    }
    .map_err(|cause| super::super::super::denial::relational_read_denial(cause, plan.name))?
    .ok_or_else(|| path_union::traversal_denial(plan.name))?;
    Ok(Arc::new(body))
}
fn make_source(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: &ReadPlan<'_>,
    meaning: &Arc<ApplicationQueryComponentMeaning>,
    body: Arc<ComponentSelection>,
) -> Result<Arc<WorthQueryObservedRootSelection>, WorthQueryApplicationReadExecutionDenial> {
    let mut facts = SourceFactsBuilder::default();
    facts
        .observe_entity(execution, plan.root)
        .map_err(|cause| super::super::super::denial::relational_read_denial(cause, plan.name))?;
    let facts = facts
        .finish(execution)
        .map_err(|cause| super::super::super::denial::relational_read_denial(cause, plan.name))?;
    Ok(Arc::new(WorthQueryObservedRootSelection::with_components(
        facts,
        DeclaredComponentSelection {
            meaning: Arc::clone(meaning),
            body,
            binding: plan.binding.clone(),
        },
    )))
}
