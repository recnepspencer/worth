//! Full Parent admission precedes any Source, provider, or retained-root lookup.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::{refresh_governed_authorization, validate_before_callback},
        one_shot::{
            outcome::material::{retain_scope, MaterialCompletion},
            plan_admission::validate_one_shot_plan,
        },
        WorthQueryApplicationBasisSelectionIdentity, WorthQueryObservedScopeSelector,
    },
    WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;

pub(super) struct ParentContext<'a, S> {
    pub(super) association: WorthQueryInstalledManagedIndexParent,
    pub(super) selected: WorthQuerySelectedProductOperation<'a, S>,
    pub(super) selector: Arc<WorthQueryObservedScopeSelector>,
}

pub(super) fn prepare<'a, S: ApplicationSchema, B: ApplicationQueryBinding<S>>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
    plan: &mut BoundPlan<'_, S, B>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<ParentContext<'a, S>, ReceiverCause> {
    if !std::ptr::eq(binding.query(), plan.query) {
        return Err(ReceiverCause::Refused(Refusal::ForeignBinding));
    }
    if !plan.controls.is_current_product_one_shot() || plan.retain_expression_context {
        return Err(ReceiverCause::Refused(Refusal::UnsupportedMode));
    }
    validate_one_shot_plan(execution, application, plan).map_err(ReceiverCause::Read)?;
    refresh_governed_authorization(execution, application, plan).map_err(|cause| {
        ReceiverCause::Read(one_shot::map_authorized_read_denial(
            cause,
            plan.query.name(),
        ))
    })?;
    validate_before_callback(execution, application, plan).map_err(|cause| {
        ReceiverCause::Read(one_shot::map_authorized_read_denial(
            cause,
            plan.query.name(),
        ))
    })?;
    let storage = MaterialCompletion::new(execution, allocation);
    let association = application
        .installed_schema()
        .installed_managed_index_parent(binding, &mut |work, bytes| {
            storage.work(work)?;
            allocation.claim(execution, bytes)
        })
        .map_err(ReceiverCause::Installation)?
        .ok_or(ReceiverCause::Refused(Refusal::NotIndexParent))?;
    let basis = plan
        .graph_work
        .query_basis()
        .ok_or(ReceiverCause::Refused(Refusal::ForeignContext))?;
    let WorthQueryApplicationBasisSelectionIdentity::Product(product) = basis.selection() else {
        return Err(ReceiverCause::Refused(Refusal::UnsupportedMode));
    };
    // The occurrence token is Copy. Query-owned retained-basis copies are
    // admitted at their actual selected-observation owner before allocation.
    let selected = application.on_branch(product.product_branch())
        .select_for_managed_index(execution, |observed| {
            let bytes = super::super::super::WorthQueryApplicationBasisIdentity::retained_basis_clone_heap_bytes(
                observed.basis().relational_basis(),
            ).ok_or(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
            storage.work(bytes)?;
            allocation.claim(execution, bytes).map_err(ReceiverCause::Execution)
        })?;
    if !plan.graph_work.admits_managed_product(
        selected.product().read_lease_ref(),
        &mut |units| storage.work(units),
        || worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow,
    )? {
        return Err(ReceiverCause::Refused(Refusal::ForeignContext));
    }
    let selector =
        retain_scope(execution, plan.scope, plan.query.name()).map_err(ReceiverCause::Read)?;
    Ok(ParentContext {
        association,
        selected,
        selector,
    })
}
