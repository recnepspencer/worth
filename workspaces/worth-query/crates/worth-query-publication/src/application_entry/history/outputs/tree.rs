//! One original tree reaches complete E; retry never substitutes the live tip.
use super::{retained::*, *};
use crate::application_entry::demand::WorthQueryApplicationProgramDemandHandle;
use crate::application_entry::mutation::program_output_continuation::{
    ProgramOutputContinuationFactory, ProgramOutputHistoryFrame,
};
use crate::application_entry::{
    WorthQueryApplicationProgramOutputHandle, WorthQueryApplicationProgramOutputProgress,
    WorthQueryRequiredOutputRecoveryPosture,
};
use std::sync::Arc;
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryCompletedHistoryOutputs, WorthQueryHistoryOutputNavigationAdvance as Step,
};

pub(super) enum TreeCompletion {
    Single(WorthQueryCompletedHistoryOutputs),
    Forest(Arc<worth_query_execution::publication_boundary::document_history::WorthQueryRetainedHistoryOutputTree>),
}
#[derive(Clone, Copy)]
pub(super) enum TreeCompletionMode {
    Single,
    Forest,
}

impl<S, P, R> WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    pub(super) fn progress(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: [u8; 32],
        step: Step<'_, '_>,
    ) -> Result<Option<WorthQueryCompletedHistoryOutputs>, WorthQueryHistoryOutputDenial> {
        self.progress_tree(phase, request, key, step, TreeCompletionMode::Single)?
            .map(|completion| match completion {
                TreeCompletion::Single(outputs) => Ok(outputs),
                TreeCompletion::Forest(_) => Err(WorthQueryHistoryOutputDenial::ForeignSource),
            })
            .transpose()
    }

    pub(super) fn progress_forest(
        &self, phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>, key: [u8; 32], step: Step<'_, '_>,
    ) -> Result<Option<Arc<worth_query_execution::publication_boundary::document_history::WorthQueryRetainedHistoryOutputTree>>, WorthQueryHistoryOutputDenial>{
        self.progress_tree(phase, request, key, step, TreeCompletionMode::Forest)?
            .map(|completion| match completion {
                TreeCompletion::Forest(tree) => Ok(tree),
                TreeCompletion::Single(_) => Err(WorthQueryHistoryOutputDenial::ForeignSource),
            })
            .transpose()
    }

    pub(super) fn progress_tree(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: [u8; 32],
        step: Step<'_, '_>,
        completion: TreeCompletionMode,
    ) -> Result<Option<TreeCompletion>, WorthQueryHistoryOutputDenial> {
        use WorthQueryHistoryOutputDenial as Denial;
        if matches!(step, Step::Selected(_)) {
            if let Some(slot) = self.existing_slot(&key)? {
                let mut state = slot.try_state()?;
                state.selected = true;
                let disposed = state.dispose(phase, &self.application).is_ok();
                drop(state);
                if disposed {
                    self.remove_slot(&key, &slot)?;
                }
            }
            return Ok(None);
        }
        // Native has already admitted H READ, metadata, NAV and complete source C.
        let slot = self.slot(phase, key)?;
        let mut state = slot.try_state()?;
        let (branch, original, authority, selecting) = match step {
            Step::Pending => {
                return if state.terminal {
                    Err(Denial::NeedsDisposal)
                } else {
                    Ok(None)
                }
            }
            Step::Selected(_) => unreachable!("selected replays never allocate another slot"),
            Step::Outputs {
                branch,
                source,
                authority,
            } => (branch, source, authority, false),
            Step::Selecting {
                branch,
                source,
                authority,
            } => (branch, source, authority, true),
        };
        if state.terminal {
            return Err(Denial::NeedsDisposal);
        }
        if let Some((actual, source)) = &state.original {
            if *actual != branch || !Arc::ptr_eq(source, &original) {
                return Err(Denial::ForeignSource);
            }
        } else {
            state.original = Some((branch, Arc::clone(&original)));
        }
        let frame = ProgramOutputHistoryFrame { authority, branch };
        if state.tree.is_none() {
            if selecting {
                return Err(Denial::ForeignSource);
            }
            let root_demand = self.demand.clone();
            let tree_demand = self.demand.clone();
            let (admitted, source) = if self.profile.uses_owned_model_graph() {
                let read = request
                    .application
                    .execute_owned_history_root_output_query::<Source<S, R>>(
                        phase,
                        authority,
                        request.principal,
                    )
                    .map_err(Denial::Source)?;
                self.application
                    .admit_history_program_root_output::<R>(
                        phase,
                        &program_publication_access(),
                        read,
                        self.controls
                            .resolve(self.application.runtime().output_demand_resource_profile()),
                    )
                    .map_err(Denial::Root)?
            } else {
                request.query(self.demand.source_intent()).with_admitted_query(phase,
                |runtime, binding, external, plan| {
                    let read = runtime.execute_history_root_output_query(phase, authority, binding, external, plan)
                        .map_err(crate::application_entry::WorthQueryApplicationRequestQueryDenial::HistoryOutput)?;
                    Ok(self.application.admit_history_program_root_output::<R>(
                        phase, &program_publication_access(), read,
                        self.controls.resolve(self.application.runtime().output_demand_resource_profile())))
                }).map_err(Denial::Query)?.map_err(Denial::Root)?
            };
            let root = WorthQueryApplicationProgramDemandHandle::new_history(
                admitted,
                root_demand,
                Arc::clone(&original),
            );
            state.source = Some(source);
            state.tree = Some(WorthQueryApplicationProgramOutputHandle::new_initial(
                crate::application_entry::WorthQueryApplicationReadObservation::new(original),
                root,
                tree_demand,
                self.controls,
            ));
        }
        let has_settlement = state.settlement.is_some();
        let tree = state.tree.as_mut().expect("the original tree is retained");
        if selecting || has_settlement {
            tree.authorize_history_in_advancement(phase, &self.application, request, frame)
                .map_err(Denial::Preparation)?;
        } else {
            match tree.advance_history_in_advancement(phase, &self.application, request, frame) {
                Ok(
                    WorthQueryApplicationProgramOutputProgress::Pending
                    | WorthQueryApplicationProgramOutputProgress::Progressed,
                ) => return Ok(None),
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(settlement)) => {
                    state.settlement = Some(settlement)
                }
                Err(mut cause) => {
                    state.read_cleanup = cause.take_read_cleanup();
                    state.managed_cleanup = cause
                        .take_managed_cleanup()
                        .map(|cleanup| cleanup.into_parts());
                    state.terminal = state.read_cleanup.is_some()
                        || state.managed_cleanup.is_some()
                        || cause.recovery_posture()
                            == WorthQueryRequiredOutputRecoveryPosture::Terminal;
                    return Err(Denial::Preparation(cause));
                }
            }
        }
        if selecting {
            return Ok(None);
        }
        // Close interests, keep the original readonly subjects for selection retries.
        state
            .tree
            .as_mut()
            .expect("retained tree")
            .drain_read_request(phase, &self.application, branch)
            .map_err(Denial::Close)?;
        if matches!(completion, TreeCompletionMode::Forest) {
            if state.forest_tree.is_none() {
                let SlotState {
                    source,
                    settlement,
                    forest_tree,
                    ..
                } = &mut *state;
                let (root, dependents, dependencies) = settlement
                    .as_mut()
                    .ok_or(Denial::ForeignSource)?
                    .original_tree_parts();
                let retained = self
                    .application
                    .retain_original_history_output_tree::<R>(
                        phase,
                        &program_publication_access(),
                        source.as_ref().ok_or(Denial::ForeignSource)?,
                        root,
                        dependents,
                        dependencies,
                    )
                    .map_err(Denial::GraphQualification)?;
                *forest_tree = Some(retained);
            }
            return Ok(state
                .forest_tree
                .as_ref()
                .map(|tree| TreeCompletion::Forest(Arc::clone(tree))));
        }
        let settlement = state.settlement.as_ref().ok_or(Denial::ForeignSource)?;
        if self.profile.uses_owned_model_graph() {
            return self
                .application
                .qualify_original_graph_history_outputs::<R>(
                    phase,
                    &program_publication_access(),
                    authority,
                    state.source.as_ref().ok_or(Denial::ForeignSource)?,
                    settlement.root_settlement().retained_settlement(),
                    settlement.retained_settlements().skip(1),
                    settlement.dependencies.as_ref(),
                )
                .map(|outputs| Some(TreeCompletion::Single(outputs)))
                .map_err(Denial::GraphQualification);
        }
        self.application
            .qualify_original_history_outputs::<R>(
                phase,
                &program_publication_access(),
                request.scope,
                state.source.as_ref().ok_or(Denial::ForeignSource)?,
                settlement.root_settlement().retained_settlement(),
                settlement.retained_settlements().skip(1),
                settlement.dependencies.as_ref(),
            )
            .map(|outputs| Some(TreeCompletion::Single(outputs)))
            .map_err(Denial::Qualification)
    }
}
