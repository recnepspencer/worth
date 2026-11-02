//! Typed inverse execution shares the actual forward provider and installed entry.
use super::super::{
    invocation::{controlled, decoded_provider, provider_result},
    SourceImageView, WorthQueryManagedIndexPredicateProvider as Provider,
    WorthQueryManagedIndexSourceChange as Change, WorthQueryManagedIndexSourceRows as Rows,
    WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Denial,
};
use super::{
    probe::{storage, RetainedPredicateProbe},
    tree::PredicateTree,
    WatchSlot,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationValueBindingCatalog as Bindings,
    WorthQueryInstalledManagedIndex as Installed, WorthQueryInstalledManagedIndexParent as Parent,
};

pub(in crate::domain_computation::primary_graph::application_query) trait PredicateFactory:
    Send + Sync
{
    fn admits(&self, parent: &Parent) -> bool;
    fn open(&self, execution: &ExecutionRequest<'_, '_>) -> Result<Arc<dyn PredicateRoot>, Denial>;
}
pub(in crate::domain_computation::primary_graph::application_query) trait PredicateRoot:
    Send + Sync
{
    fn len(&self) -> usize;
    fn insert(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        slot: WatchSlot,
        probe: Arc<RetainedPredicateProbe>,
    ) -> Result<(), Denial>;
    fn remove(&self, slot: WatchSlot);
    fn invalidate_scope(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        budget: &worth_relational::facade::runtime::CustomInvariantReadBudget<'_>,
        receive: &mut dyn FnMut(
            WatchSlot,
            &super::super::WorthQueryManagedIndexControl<'_>,
        ) -> Result<(), Denial>,
    ) -> Result<(), Denial>;
    fn route(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        before: SourceImageView<'_>,
        after: SourceImageView<'_>,
        changes: &[Change],
        receive: &mut dyn FnMut(WatchSlot, &crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexControl<'_>) -> Result<(), Denial>,
    ) -> Result<(), Denial>;
    fn dispose(&self);
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) struct TypedPredicateFactory<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: Provider<S, I>,
> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) installed:
        Installed<S, I>,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) provider:
        Arc<P>,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) refinement:
        Option<super::super::refinement::RefinementCallbacks<S, I, P>>,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) bindings:
        Bindings,
}
struct TypedPredicateRoot<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: Provider<S, I>,
> {
    installed: Installed<S, I>,
    provider: Arc<P>,
    refinement: Option<super::super::refinement::RefinementCallbacks<S, I, P>>,
    bindings: Bindings,
    tree: Mutex<PredicateTree<Arc<RetainedPredicateProbe>, (), P::PredicateSummary>>,
    _allocation: Arc<SourceFactsAllocation>,
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>, P: Provider<S, I>> PredicateFactory
    for TypedPredicateFactory<S, I, P>
{
    fn admits(&self, parent: &Parent) -> bool {
        (parent.access_contract().meaning().match_posture() != worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined || self.refinement.is_some())
            && self.installed.admits_association(parent)
            && parent
                .access_contract()
                .meaning()
                .predicate_profile()
                .is_ok()
    }
    fn open(&self, execution: &ExecutionRequest<'_, '_>) -> Result<Arc<dyn PredicateRoot>, Denial> {
        let allocation = SourceFactsAllocation::open(execution).map_err(storage)?;
        allocation
            .claim_arc::<TypedPredicateRoot<S, I, P>>(execution)
            .map_err(storage)?;
        Ok(Arc::new(TypedPredicateRoot {
            installed: self.installed.clone(),
            provider: self.provider.clone(),
            bindings: self.bindings.clone(),
            refinement: self.refinement,
            tree: Mutex::new(PredicateTree::default()),
            _allocation: allocation,
        }))
    }
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>, P: Provider<S, I>> PredicateRoot
    for TypedPredicateRoot<S, I, P>
{
    fn len(&self) -> usize {
        self.tree
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
    fn insert(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        slot: WatchSlot,
        probe: Arc<RetainedPredicateProbe>,
    ) -> Result<(), Denial> {
        if !self.installed.admits_association(&probe.parent) {
            return Err(Stop::ForeignCoverage.into());
        }
        // This outer share survives every refusal inside the gate. No failed
        // insertion can run the last raw-parameter/grant destructor there.
        let retained = Arc::clone(&probe);
        let mut attached = false;
        let result = decoded_provider(execution, &self.bindings, |control, session| {
            let own = decode::<S, I>(&probe, control, session)?;
            provider_result(
                execution,
                control,
                self.provider.validate_probe(&own, control),
            )?;
            if probe.parent.access_contract().meaning().match_posture() == worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined {
                let refine = self.refinement.as_ref().ok_or_else(|| control.refuse(Stop::RefinementUnavailable))?;
                provider_result(execution, control, (refine.validate)(&self.provider, &own, control))?;
            }
            let summary = provider_result(
                execution,
                control,
                self.provider.predicate_summary(&own, control),
            )?;
            self.tree
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(
                    execution,
                    slot,
                    Arc::clone(&probe),
                    (),
                    summary,
                    control,
                    |_, other| {
                        let other = decode::<S, I>(other, control, session)?;
                        provider_result(
                            execution,
                            control,
                            self.provider.compare_probes(&own, &other, control),
                        )
                    },
                )?;
            attached = true;
            Ok(())
        });
        // The outer decoder/control postlude may stop after successful attach.
        // Undo only that actual new record, outside its gate, preserving the stop.
        if result.is_err() && attached {
            self.remove(slot);
        }
        drop(retained);
        result
    }
    fn remove(&self, slot: WatchSlot) {
        let retired = self
            .tree
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(slot);
        drop(retired);
    }
    fn route(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        before: SourceImageView<'_>,
        after: SourceImageView<'_>,
        changes: &[Change],
        receive: &mut dyn FnMut(WatchSlot, &crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexControl<'_>) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        decoded_provider(execution, &self.bindings, |control, session| {
            let source_control = after
                .source_budget
                .map(|budget| control.under_source(budget));
            let control = source_control.as_ref().unwrap_or(control);
            let before = Rows::<S, I>::new(before, session, control, *execution);
            let after = Rows::<S, I>::new(after, session, control, *execution);
            let tree = self
                .tree
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for change in changes {
                for (rows, source) in [(&before, change.before()), (&after, change.after())] {
                    let Some(source) = source else {
                        continue;
                    };
                    tree.visit(
                        control,
                        &mut |summary, extrema| {
                            let a = decode::<S, I>(extrema[0], control, session)?;
                            let b = decode::<S, I>(extrema[1], control, session)?;
                            provider_result(
                                execution,
                                control,
                                self.provider.predicate_region_may_match(
                                    summary,
                                    [&a, &b],
                                    rows.get(source)?,
                                    control,
                                ),
                            )
                        },
                        &mut |slot, probe, _| {
                            let refined = probe.parent.access_contract().meaning().match_posture() == worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined;
                            let probe = decode::<S, I>(probe, control, session)?;
                            let candidate = provider_result(
                                execution,
                                control,
                                self.provider
                                    .matches_source(rows.get(source)?, &probe, control),
                            )?;
                            if candidate {
                                let matches = if refined {
                                    let refine = self.refinement.as_ref().ok_or_else(|| control.refuse(Stop::RefinementUnavailable))?;
                                    provider_result(execution, control, (refine.evaluate)(&self.provider, rows.get(source)?, &probe, control))?
                                } else { true };
                                if matches { receive(slot, control)?; }
                            }
                            Ok(())
                        },
                    )?;
                }
            }
            Ok(())
        })
    }
    fn invalidate_scope(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        budget: &worth_relational::facade::runtime::CustomInvariantReadBudget<'_>,
        receive: &mut dyn FnMut(
            WatchSlot,
            &super::super::WorthQueryManagedIndexControl<'_>,
        ) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        controlled(execution, |control| {
            let control = control.under_source(budget);
            let tree = self
                .tree
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // Every record in THIS exact Source/World/scope is an affected output.
            for slot in tree.slots() {
                control.checkpoint(1)?;
                receive(slot, &control)?;
            }
            Ok(())
        })
    }
    fn dispose(&self) {
        let retired = std::mem::take(
            &mut *self
                .tree
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        drop(retired);
    }
}
fn decode<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    probe: &RetainedPredicateProbe,
    control: &super::super::WorthQueryManagedIndexControl<'_>,
    session: &crate::domain_computation::primary_graph::value_decode::ValueDecodeSession<'_>,
) -> Result<super::super::WorthQueryManagedIndexProbeValue<S, I>, Denial> {
    session
        .decode_managed_probe::<I::ProbeBinding>(probe.raw())
        .ok_or_else(|| {
            control
                .refuse(Stop::ValueDecode(
                    session
                        .denial()
                        .expect("managed probe decode latches its original refusal"),
                ))
                .into()
        })
}
