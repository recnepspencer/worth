//! Capture consumes the actual admitted lease and its complete candidate custody.
use super::super::{
    invocation::controlled, selection::SelectionDenial as Denial,
    source::service::ManagedSourceRoot,
};
use super::*;
use crate::domain_computation::{
    primary_graph::{
        access_strategy::{
            coverage::ReceivedSelection, realized_footprint::CapturedManagedInfluence,
        },
        application_query::one_shot::outcome::material::MaterialCompletion,
    },
    provider_session::{WorthQueryManagedGraphWorkSession, WorthQueryManagedIndexSelectionProof},
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::{
    application_query::WorthQueryAdmittedApplicationQueryParameters as Parameters,
    authenticated_principal::WorthQueryRequestScope,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent as Parent;

pub(in crate::domain_computation::primary_graph::application_query::managed_index) struct CaptureInput<
    'a,
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    pub parent: &'a WorthQueryManagedGraphWorkSession,
    pub request: &'a WorthQueryRequestScope,
    pub association: &'a Parent,
    pub source: &'a Arc<WorthQueryCompletedSourceRead>,
    pub root: &'a ManagedSourceRoot<S, I>,
    pub observation: &'a Arc<ManagedIndexSourceObservation>,
    pub parameters: &'a Parameters,
    pub governance: &'a crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    pub probe: &'a AspectValue,
    pub selected: &'a ReceivedSelection,
    pub allocation: &'a Arc<SourceFactsAllocation>,
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn capture<
    S,
    I,
>(
    execution: &ExecutionRequest<'_, '_>,
    proof: &WorthQueryManagedIndexSelectionProof<'_, '_, '_>,
    input: CaptureInput<'_, S, I>,
) -> Result<Option<Arc<CapturedManagedInfluence>>, Denial>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
{
    let CaptureInput {
        parent,
        request,
        association,
        source,
        root,
        observation,
        parameters: actual_parameters,
        governance,
        probe,
        selected,
        allocation,
    } = input;
    // Only the installed complete full-projection contract can close all tracked
    // fields, root predicates, nested endpoints and ordering within this Source.
    if association
        .access_contract()
        .meaning()
        .predicate_profile()
        .is_err()
    {
        return Ok(None);
    }
    let Some(coverage) = selected.coverage() else {
        return Ok(None);
    };
    proof.consult().map_err(Denial::Session)?;
    if !proof.admits(parent, request, association, source)
        || !coverage.admits(root.adapter.coverage_authority())
        || !root.installed.admits_association(association)
    {
        return Err(Denial::ForeignSelection);
    }
    let parameters = RetainedPredicateProbe::capture(execution, association, actual_parameters)
        .map_err(Denial::Work)?;
    if !parameters.matches_probe_borrow(actual_parameters, probe) {
        return Err(Denial::ForeignSelection);
    }
    let audience = governance.retain_managed_influence(execution, parent, association, actual_parameters, allocation)
        .map_err(|cause| match cause {
            crate::domain_computation::authorization::application_disclosure::WorthQueryRetainedQueryInfluenceDenial::ForeignAdmission => Denial::ForeignSelection,
            crate::domain_computation::authorization::application_disclosure::WorthQueryRetainedQueryInfluenceDenial::Execution(cause) => Denial::Execution(cause),
        })?;
    let candidates = controlled(execution, |control| {
        CapturedCandidateRows::capture(execution, &root.rows, selected, control)
    })
    .map_err(Denial::Work)?;
    let checked = CheckedManagedFootprintCapture {
        frame: worth_proof::BorrowedRealizedFootprintFrame::new(
            association,
            root,
            parameters.parameters(),
            parent,
        ),
        coverage,
    };
    let witness = root
        .adapter
        .verify_realized(&checked)
        .map_err(|cause| Denial::Work(cause.into()))?;
    let material = MaterialCompletion::new(execution, allocation);
    material.arc::<CapturedIndexInfluence<S, I>>()?;
    material.arc::<crate::domain_computation::authorization::application_disclosure::WorthQueryRetainedQueryInfluence>()?;
    material.arc::<CapturedManagedInfluence>()?;
    let support = Arc::new(CapturedIndexInfluence {
        adapter: Arc::clone(&root.adapter),
        original: Arc::clone(observation),
        scope: source.scope(),
        parameters,
        audience: Arc::new(audience),
        candidates,
        _coverage: Arc::clone(coverage),
        _admitted_source: Arc::clone(source),
        _allocation: Arc::clone(allocation),
    });
    let retained = CapturedManagedInfluence::new(
        witness,
        root.adapter.realized_authority(),
        support,
        Arc::clone(allocation),
    )
    .map_err(|_| Denial::ForeignSelection)?;
    Ok(Some(Arc::new(retained)))
}
